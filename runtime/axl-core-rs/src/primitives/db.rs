use crate::ir::Value;

pub fn db_connect(args: &[Value]) -> Result<Value, String> {
    let path = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or(":memory:");
    Ok(Value::String(format!("db_{path}")))
}

pub fn db_execute(args: &[Value]) -> Result<Value, String> {
    let sql = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("affected".into()), Value::Int(0)),
        (Value::String("sql".into()), Value::String(sql.into())),
    ]))
}

pub fn db_query(args: &[Value]) -> Result<Value, String> {
    let sql = args.get(1).and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    Ok(Value::List(vec![
        Value::Map(vec![
            (Value::String("query".into()), Value::String(sql.into())),
            (Value::String("rows".into()), Value::Int(0)),
        ])
    ]))
}

pub fn db_begin(args: &[Value]) -> Result<Value, String> { Ok(Value::Bool(true)) }
pub fn db_commit(args: &[Value]) -> Result<Value, String> { Ok(Value::Bool(true)) }
pub fn db_rollback(args: &[Value]) -> Result<Value, String> { Ok(Value::Bool(true)) }

pub fn db_tables(args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(vec![
        Value::String("users".into()),
        Value::String("todos".into()),
    ]))
}
