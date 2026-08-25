use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ir::Value;
use rusqlite::Connection;

pub type SharedConnection = Arc<Mutex<Connection>>;

lazy_static::lazy_static! {
    static ref CONNECTIONS: Mutex<HashMap<String, SharedConnection>> = Mutex::new(HashMap::new());
}

/// Get a shared connection handle for use by other modules (e.g., HTTP server).
pub fn get_shared_connection(db_id: &str) -> Option<SharedConnection> {
    CONNECTIONS.lock().unwrap().get(db_id).cloned()
}

pub fn db_connect(args: &[Value]) -> Result<Value, String> {
    let path = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or(":memory:");

    let conn = Connection::open(path).map_err(|e| format!("db_connect: {e}"))?;
    let db_id = format!("db_{}", path.replace('/', "_").replace('.', "_").replace(':', "_").replace(' ', "_"));

    CONNECTIONS.lock().unwrap().insert(db_id.clone(), Arc::new(Mutex::new(conn)));

    Ok(Value::String(db_id))
}

pub fn db_execute(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_execute: missing db_id")?;

    let sql = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_execute: missing SQL")?;

    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_execute: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();

    conn.execute_batch(sql).map_err(|e| format!("db_execute: {e}"))?;

    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::String("ok".into())),
        (Value::String("sql".into()), Value::String(sql.into())),
    ]))
}

pub fn db_query(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_query: missing db_id")?;

    let sql = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_query: missing SQL")?;

    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_query: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();

    let mut stmt = conn.prepare(sql).map_err(|e| format!("db_query prepare: {e}"))?;
    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    
    let mut rows = Vec::new();
    let mut row_iter = stmt.query([]).map_err(|e| format!("db_query query: {e}"))?;
    
    while let Some(row) = row_iter.next().map_err(|e| format!("db_query row: {e}"))? {
        let mut map = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            let value = row.get::<_, rusqlite::types::Value>(i).map_err(|e| format!("db_query get: {e}"))?;
            let axl_value = match value {
                rusqlite::types::Value::Null => Value::Null,
                rusqlite::types::Value::Integer(n) => Value::Int(n),
                rusqlite::types::Value::Real(r) => Value::Int(r as i64),
                rusqlite::types::Value::Text(s) => Value::String(s),
                rusqlite::types::Value::Blob(b) => Value::String(format!("blob:{} bytes", b.len())),
            };
            map.push((Value::String(col.clone()), axl_value));
        }
        rows.push(Value::Map(map));
    }
    
    Ok(Value::List(rows))
}

pub fn db_begin(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_begin: missing db_id")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_begin: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();

    conn.execute_batch("BEGIN TRANSACTION").map_err(|e| format!("db_begin: {e}"))?;
    
    Ok(Value::String(format!("tx_{db_id}")))
}

pub fn db_commit(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_commit: missing db_id")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_commit: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();

    conn.execute_batch("COMMIT").map_err(|e| format!("db_commit: {e}"))?;
    
    Ok(Value::Bool(true))
}

pub fn db_rollback(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_rollback: missing db_id")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_rollback: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();

    conn.execute_batch("ROLLBACK").map_err(|e| format!("db_rollback: {e}"))?;
    
    Ok(Value::Bool(true))
}

pub fn db_tables(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_tables: missing db_id")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_tables: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").map_err(|e| format!("db_tables: {e}"))?;
    let mut tables = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| format!("db_tables: {e}"))?;
    
    while let Some(row) = rows.next().map_err(|e| format!("db_tables row: {e}"))? {
        let name: String = row.get(0).map_err(|e| format!("db_tables get: {e}"))?;
        tables.push(Value::String(name));
    }
    
    Ok(Value::List(tables))
}

pub fn db_columns(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_columns: missing db_id")?;
    
    let table = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_columns: missing table name")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_columns: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&sql).map_err(|e| format!("db_columns: {e}"))?;
    let mut columns = Vec::new();
    let mut rows = stmt.query([]).map_err(|e| format!("db_columns: {e}"))?;
    
    while let Some(row) = rows.next().map_err(|e| format!("db_columns row: {e}"))? {
        let name: String = row.get(1).map_err(|e| format!("db_columns get: {e}"))?;
        let col_type: String = row.get(2).map_err(|e| format!("db_columns get type: {e}"))?;
        columns.push(Value::Map(vec![
            (Value::String("name".into()), Value::String(name)),
            (Value::String("type".into()), Value::String(col_type)),
        ]));
    }
    
    Ok(Value::List(columns))
}

pub fn db_count(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_count: missing db_id")?;
    
    let table = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_count: missing table name")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_count: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let mut stmt = conn.prepare(&sql).map_err(|e| format!("db_count: {e}"))?;
    let count: i64 = stmt.query_row([], |row| row.get(0)).map_err(|e| format!("db_count: {e}"))?;
    
    Ok(Value::Int(count))
}

pub fn db_insert(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_insert: missing db_id")?;
    
    let table = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_insert: missing table name")?;
    
    let data = args.get(2).and_then(|v| match v {
        Value::Map(m) => Some(m),
        _ => None,
    }).ok_or("db_insert: missing data map")?;
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_insert: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    // Build INSERT statement from map
    let columns: Vec<String> = data.iter().filter_map(|(k, _)| {
        if let Value::String(s) = k { Some(s.clone()) } else { None }
    }).collect();
    
    let placeholders: Vec<String> = columns.iter().map(|_| "?".to_string()).collect();
    let sql = format!("INSERT INTO {table} ({}) VALUES ({})", columns.join(", "), placeholders.join(", "));
    
    let values: Vec<rusqlite::types::Value> = data.iter().filter_map(|(k, v)| {
        if let Value::String(_) = k {
            Some(match v {
                Value::Null => rusqlite::types::Value::Null,
                Value::Int(n) => rusqlite::types::Value::Integer(*n),
                Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                _ => rusqlite::types::Value::Text(format!("{v:?}")),
            })
        } else {
            None
        }
    }).collect();
    
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
    conn.execute(&sql, params_refs.as_slice()).map_err(|e| format!("db_insert: {e}"))?;
    
    let row_id: i64 = conn.last_insert_rowid();
    Ok(Value::Int(row_id))
}

pub fn db_update(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_update: missing db_id")?;
    
    let table = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_update: missing table name")?;
    
    let data = args.get(2).and_then(|v| match v {
        Value::Map(m) => Some(m),
        _ => None,
    }).ok_or("db_update: missing data map")?;
    
    let where_clause = args.get(3).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("1=1");
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_update: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    // Build SET clause
    let set_parts: Vec<String> = data.iter().filter_map(|(k, _)| {
        if let Value::String(s) = k { Some(format!("{s} = ?")) } else { None }
    }).collect();
    
    let sql = format!("UPDATE {table} SET {} WHERE {where_clause}", set_parts.join(", "));
    
    let values: Vec<rusqlite::types::Value> = data.iter().filter_map(|(k, v)| {
        if let Value::String(_) = k {
            Some(match v {
                Value::Null => rusqlite::types::Value::Null,
                Value::Int(n) => rusqlite::types::Value::Integer(*n),
                Value::String(s) => rusqlite::types::Value::Text(s.clone()),
                Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                _ => rusqlite::types::Value::Text(format!("{v:?}")),
            })
        } else {
            None
        }
    }).collect();
    
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
    let affected = conn.execute(&sql, params_refs.as_slice()).map_err(|e| format!("db_update: {e}"))?;
    
    Ok(Value::Int(affected as i64))
}

pub fn db_delete(args: &[Value]) -> Result<Value, String> {
    let db_id = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_delete: missing db_id")?;
    
    let table = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).ok_or("db_delete: missing table name")?;
    
    let where_clause = args.get(2).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("1=1");
    
    let connections = CONNECTIONS.lock().unwrap();
    let shared = connections.get(db_id).ok_or_else(|| format!("db_delete: unknown db '{db_id}'"))?;
    let conn = shared.lock().unwrap();
    
    let sql = format!("DELETE FROM {table} WHERE {where_clause}");
    conn.execute_batch(&sql).map_err(|e| format!("db_delete: {e}"))?;
    
    Ok(Value::Bool(true))
}
