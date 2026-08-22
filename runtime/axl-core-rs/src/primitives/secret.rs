use crate::ir::Value;

pub fn secret_store(args: &[Value]) -> Result<Value, String> {
    let _key = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _value = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn secret_get(args: &[Value]) -> Result<Value, String> {
    let _key = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Null)
}

pub fn secret_delete(args: &[Value]) -> Result<Value, String> {
    let _key = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn secret_list(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(vec![]))
}
