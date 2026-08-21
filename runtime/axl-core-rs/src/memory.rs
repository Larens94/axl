use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ir::Value;

pub const DEFAULT_SCOPE: &str = "session:default";
pub const MAX_PERSISTED_VALUE_BYTES: usize = 2_000_000;

pub trait MemoryStore {
    fn configure_limits(&mut self, max_bytes: usize, max_nodes: usize, max_depth: usize);
    fn get(&mut self, key: &str, scope: &str) -> Result<Option<Value>, String>;
    fn set(&mut self, key: &str, value: Value, scope: &str, confidence: i32, ttl_seconds: Option<i64>, source: &str) -> Result<(), String>;
    fn delete(&mut self, key: &str, scope: &str) -> Result<bool, String>;
    fn snapshot(&mut self, scope: &str) -> Result<HashMap<String, Value>, String>;
}

pub struct InMemoryStore {
    records: HashMap<(String, String), (Value, i64, String, i32, String, Option<String>)>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self { records: HashMap::new() }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self { Self::new() }
}

impl MemoryStore for InMemoryStore {
    fn configure_limits(&mut self, _max_bytes: usize, _max_nodes: usize, _max_depth: usize) {}

    fn get(&mut self, key: &str, scope: &str) -> Result<Option<Value>, String> {
        let k = (scope.to_string(), key.to_string());
        match self.records.get(&k) {
            Some((_, _, _, _, _, expires)) if is_expired(expires) => {
                self.records.remove(&k);
                Ok(None)
            }
            Some((value, _, _, _, _, _)) => Ok(Some(value.clone())),
            None => Ok(None),
        }
    }

    fn set(&mut self, key: &str, value: Value, scope: &str, confidence: i32, ttl_seconds: Option<i64>, source: &str) -> Result<(), String> {
        let k = (scope.to_string(), key.to_string());
        let version = self.records.get(&k).map(|(_, v, _, _, _, _)| v + 1).unwrap_or(1);
        self.records.insert(k, (value, version, now_iso(), confidence, source.to_string(), compute_expiry(ttl_seconds)));
        Ok(())
    }

    fn delete(&mut self, key: &str, scope: &str) -> Result<bool, String> {
        Ok(self.records.remove(&(scope.to_string(), key.to_string())).is_some())
    }

    fn snapshot(&mut self, scope: &str) -> Result<HashMap<String, Value>, String> {
        let s = scope.to_string();
        let keys: Vec<String> = self.records.keys()
            .filter(|(rs, _)| rs == &s)
            .map(|(_, k)| k.clone())
            .collect();
        let mut result = HashMap::new();
        for k in keys {
            if let Some((value, _, _, _, _, _)) = self.records.get(&(s.clone(), k.clone())) {
                result.insert(k, value.clone());
            }
        }
        Ok(result)
    }
}

pub struct SQLiteMemoryStore {
    connection: rusqlite::Connection,
}

impl SQLiteMemoryStore {
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
        let store = Self { connection: conn };
        store.initialize()?;
        Ok(store)
    }

    fn initialize(&self) -> Result<(), String> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS memory (\
             scope TEXT NOT NULL, key TEXT NOT NULL, value_json TEXT NOT NULL, \
             version INTEGER NOT NULL, updated_at TEXT NOT NULL, \
             confidence INTEGER NOT NULL DEFAULT 100, source TEXT NOT NULL DEFAULT 'program', \
             expires_at TEXT, PRIMARY KEY(scope,key))"
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl MemoryStore for SQLiteMemoryStore {
    fn configure_limits(&mut self, _max_bytes: usize, _max_nodes: usize, _max_depth: usize) {}

    fn get(&mut self, key: &str, scope: &str) -> Result<Option<Value>, String> {
        let result: Option<(String, Option<String>)> = {
            let mut stmt = self.connection
                .prepare("SELECT value_json,expires_at FROM memory WHERE scope=? AND key=?")
                .map_err(|e| e.to_string())?;
            let mut rows = stmt.query_map(rusqlite::params![scope, key], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            }).map_err(|e| e.to_string())?;
            rows.next().and_then(|r| r.ok())
        };
        if let Some((json, expires)) = result {
            if is_expired(&expires) {
                self.connection.execute("DELETE FROM memory WHERE scope=? AND key=?", rusqlite::params![scope, key]).map_err(|e| e.to_string())?;
                return Ok(None);
            }
            let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| format!("invalid persisted memory value: {e}"))?;
            let value = Value::from_json_value(&parsed).map_err(|e| format!("invalid persisted memory value: {e}"))?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    fn set(&mut self, key: &str, value: Value, scope: &str, confidence: i32, ttl_seconds: Option<i64>, source: &str) -> Result<(), String> {
        let json = serde_json::to_string(&value.to_json_value()).map_err(|e| e.to_string())?;
        let now = now_iso();
        let expires = compute_expiry(ttl_seconds);
        self.connection.execute(
            "INSERT INTO memory(scope,key,value_json,version,updated_at,confidence,source,expires_at) \
             VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(scope,key) DO UPDATE SET \
             value_json=excluded.value_json, version=memory.version+1, \
             updated_at=excluded.updated_at, confidence=excluded.confidence, \
             source=excluded.source, expires_at=excluded.expires_at",
            rusqlite::params![scope, key, json, 1, now, confidence, source, expires],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn delete(&mut self, key: &str, scope: &str) -> Result<bool, String> {
        let affected = self.connection.execute(
            "DELETE FROM memory WHERE scope=? AND key=?", rusqlite::params![scope, key],
        ).map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    fn snapshot(&mut self, scope: &str) -> Result<HashMap<String, Value>, String> {
        let keys: Vec<String> = {
            let mut stmt = self.connection
                .prepare("SELECT key FROM memory WHERE scope=? ORDER BY key")
                .map_err(|e| e.to_string())?;
            stmt.query_map(rusqlite::params![scope], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok()).collect()
        };
        let mut result = HashMap::new();
        for k in keys {
            if let Some(v) = self.get(&k, scope)? {
                result.insert(k, v);
            }
        }
        Ok(result)
    }
}

pub fn now_iso() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:.3}Z", d.as_secs_f64())
}

fn compute_expiry(ttl_seconds: Option<i64>) -> Option<String> {
    ttl_seconds.map(|ttl| {
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        format!("{:.3}Z", (d.as_secs_f64() + ttl as f64))
    })
}

fn is_expired(expires_at: &Option<String>) -> bool {
    match expires_at {
        None => false,
        Some(exp) => now_iso() >= *exp,
    }
}
