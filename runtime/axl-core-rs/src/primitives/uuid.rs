use crate::ir::Value;

pub fn uuid_v4(_args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = t.as_nanos();
    let hex: String = format!("{:032x}", nanos as u128);
    let uuid = format!("{}-{}-4{}-{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[16..20], &hex[20..32]);
    Ok(Value::String(uuid))
}

pub fn uuid_v5(args: &[Value]) -> Result<Value, String> {
    let namespace = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
    let name = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let combined = format!("{namespace}{name}{:?}", t.as_nanos());
    let hex: String = format!("{:032x}", combined.len() as u128);
    let uuid = format!("{}-{}-5{}-{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[16..20], &hex[20..32]);
    Ok(Value::String(uuid))
}

pub fn uuid_parse(args: &[Value]) -> Result<Value, String> {
    let uuid = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(uuid.len() == 36 && uuid.chars().nth(8) == Some('-')))
}

pub fn uuid_validate(args: &[Value]) -> Result<Value, String> {
    uuid_parse(args)
}
