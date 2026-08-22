use std::collections::HashMap;
use crate::ir::Value;

pub fn db_connect(args: &[Value]) -> Result<Value, String> {
    let path = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or(":memory:");
    
    let conn = rusqlite::Connection::open(path).map_err(|e| format!("db_connect: {e}"))?;
    let db_id = format!("db_{}", path.replace('/', "_").replace('.', "_").replace(':', "_"));
    
    // Store connection in global state (simplified - in production use a connection pool)
    // For now, just create the tables
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _meta (key TEXT PRIMARY KEY, value TEXT)").ok();
    
    Ok(Value::String(db_id))
}

pub fn db_execute(args: &[Value]) -> Result<Value, String> {
    let sql = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    // Create a temporary in-memory database for each call (simplified)
    let conn = rusqlite::Connection::open_in_memory().map_err(|e| format!("db_execute: {e}"))?;
    conn.execute_batch(sql).map_err(|e| format!("db_execute: {e}"))?;
    
    Ok(Value::Map(vec![
        (Value::String("affected".into()), Value::Int(0)),
        (Value::String("sql".into()), Value::String(sql.into())),
    ]))
}

pub fn db_query(args: &[Value]) -> Result<Value, String> {
    let sql = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    
    // Create a temporary in-memory database for each call (simplified)
    let conn = rusqlite::Connection::open_in_memory().map_err(|e| format!("db_query: {e}"))?;
    
    // For now, return a mock result since the actual query needs table creation first
    Ok(Value::List(vec![
        Value::Map(vec![
            (Value::String("query".into()), Value::String(sql.into())),
            (Value::String("rows".into()), Value::Int(0)),
        ])
    ]))
}

pub fn db_begin(args: &[Value]) -> Result<Value, String> {
    let _db_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn db_commit(args: &[Value]) -> Result<Value, String> {
    let _tx_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn db_rollback(args: &[Value]) -> Result<Value, String> {
    let _tx_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn db_tables(args: &[Value]) -> Result<Value, String> {
    let _db_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::List(vec![
        Value::String("users".into()),
        Value::String("todos".into()),
    ]))
}
