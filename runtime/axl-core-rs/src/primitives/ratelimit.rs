use crate::ir::Value;

pub fn ratelimit_create(args: &[Value]) -> Result<Value, String> {
    let _max = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(100);
    let _window = args.get(1).and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(60000);
    Ok(Value::String("ratelimit_1".into()))
}

pub fn ratelimit_check(args: &[Value]) -> Result<Value, String> {
    let _rl = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _key = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn ratelimit_reset(args: &[Value]) -> Result<Value, String> {
    let _rl = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}
