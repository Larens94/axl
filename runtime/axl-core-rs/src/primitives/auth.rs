use crate::ir::Value;

pub fn auth_hash_password(args: &[Value]) -> Result<Value, String> {
    let password = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let hash = format!("{:x}", md5::compute(password.as_bytes()));
    Ok(Value::String(hash))
}

pub fn auth_verify_password(args: &[Value]) -> Result<Value, String> {
    let password = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let hash = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let computed = format!("{:x}", md5::compute(password.as_bytes()));
    Ok(Value::Bool(computed == hash))
}

pub fn auth_jwt_create(args: &[Value]) -> Result<Value, String> {
    let _payload = args.first().cloned().unwrap_or(Value::Null);
    let _secret = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("jwt_token_placeholder".into()))
}

pub fn auth_jwt_verify(args: &[Value]) -> Result<Value, String> {
    let _token = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _secret = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn auth_jwt_decode(args: &[Value]) -> Result<Value, String> {
    let _token = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![]))
}

pub fn session_create(args: &[Value]) -> Result<Value, String> {
    let _user_data = args.first().cloned().unwrap_or(Value::Null);
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(Value::String(format!("session_{}", t.as_nanos())))
}

pub fn session_get(args: &[Value]) -> Result<Value, String> {
    let _session_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![]))
}

pub fn session_destroy(args: &[Value]) -> Result<Value, String> {
    let _session_id = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}
