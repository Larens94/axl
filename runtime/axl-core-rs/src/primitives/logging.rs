use crate::ir::Value;

pub fn log_info(args: &[Value]) -> Result<Value, String> {
    let msg = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    eprintln!("[INFO] {msg}");
    Ok(Value::Bool(true))
}

pub fn log_warn(args: &[Value]) -> Result<Value, String> {
    let msg = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    eprintln!("[WARN] {msg}");
    Ok(Value::Bool(true))
}

pub fn log_error(args: &[Value]) -> Result<Value, String> {
    let msg = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    eprintln!("[ERROR] {msg}");
    Ok(Value::Bool(true))
}

pub fn log_debug(args: &[Value]) -> Result<Value, String> {
    let msg = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    eprintln!("[DEBUG] {msg}");
    Ok(Value::Bool(true))
}

pub fn log_set_level(args: &[Value]) -> Result<Value, String> {
    let _level = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("info");
    Ok(Value::Bool(true))
}

pub fn log_set_file(args: &[Value]) -> Result<Value, String> {
    let _path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn log_json(args: &[Value]) -> Result<Value, String> {
    let event = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let data = args.get(1).cloned().unwrap_or(Value::Null);
    eprintln!("[JSON] event={event} data={data:?}");
    Ok(Value::Bool(true))
}
