use crate::ir::Value;

pub fn validate_email(args: &[Value]) -> Result<Value, String> {
    let email = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(email.contains('@') && email.contains('.')))
}

pub fn validate_url(args: &[Value]) -> Result<Value, String> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(url.starts_with("http://") || url.starts_with("https://")))
}

pub fn validate_ip(args: &[Value]) -> Result<Value, String> {
    let ip = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 { return Ok(Value::Bool(false)); }
    let valid = parts.iter().all(|p| p.parse::<u8>().is_ok());
    Ok(Value::Bool(valid))
}

pub fn validate_uuid(args: &[Value]) -> Result<Value, String> {
    let uuid = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let valid = uuid.len() == 36 && uuid.chars().nth(8) == Some('-') && uuid.chars().nth(13) == Some('-');
    Ok(Value::Bool(valid))
}

pub fn validate_json_str(args: &[Value]) -> Result<Value, String> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(serde_json::from_str::<serde_json::Value>(text).is_ok()))
}

pub fn validate_regex(args: &[Value]) -> Result<Value, String> {
    let _text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn validate_credit_card(args: &[Value]) -> Result<Value, String> {
    let _card = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(false))
}

pub fn validate_phone(args: &[Value]) -> Result<Value, String> {
    let phone = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    Ok(Value::Bool(digits.len() >= 10))
}
