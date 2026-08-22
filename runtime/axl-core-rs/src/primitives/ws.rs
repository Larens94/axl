use crate::ir::Value;

pub fn ws_server_create(args: &[Value]) -> Result<Value, String> {
    let port = args.first().and_then(|v| match v {
        Value::Int(n) => Some(*n as u16),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }).unwrap_or(8080);
    Ok(Value::String(format!("ws_server_{port}")))
}

pub fn ws_connect(args: &[Value]) -> Result<Value, String> {
    let url = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("ws://localhost:8080");
    Ok(Value::String(format!("ws_conn_{url}")))
}

pub fn ws_send(args: &[Value]) -> Result<Value, String> {
    let _conn = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    let _data = args.get(1).cloned().unwrap_or(Value::Null);
    Ok(Value::Bool(true))
}

pub fn ws_recv(args: &[Value]) -> Result<Value, String> {
    let _conn = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("type".into()), Value::String("message".into())),
        (Value::String("data".into()), Value::String("".into())),
    ]))
}

pub fn ws_broadcast(args: &[Value]) -> Result<Value, String> {
    let _server = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    let _data = args.get(1).cloned().unwrap_or(Value::Null);
    Ok(Value::Bool(true))
}

pub fn ws_on_message(args: &[Value]) -> Result<Value, String> {
    let _server = args.first().and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    }).unwrap_or("");
    Ok(Value::Bool(true))
}
