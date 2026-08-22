use crate::ir::Value;

pub fn watch_create(args: &[Value]) -> Result<Value, String> {
    let _path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or(".");
    Ok(Value::String("watcher_1".into()))
}

pub fn watch_add(args: &[Value]) -> Result<Value, String> {
    let _watcher = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("*");
    Ok(Value::Bool(true))
}

pub fn watch_remove(args: &[Value]) -> Result<Value, String> {
    let _watcher = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("*");
    Ok(Value::Bool(true))
}

pub fn watch_poll(args: &[Value]) -> Result<Value, String> {
    let _watcher = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::List(vec![]))
}

pub fn watch_close(args: &[Value]) -> Result<Value, String> {
    let _watcher = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}
