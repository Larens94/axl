use crate::ir::Value;

pub fn cron_create(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::String("cron_1".into()))
}

pub fn cron_add(args: &[Value]) -> Result<Value, String> {
    let _cron = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _schedule = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("* * * * *");
    let _task = args.get(2).cloned().unwrap_or(Value::Null);
    Ok(Value::String("job_1".into()))
}

pub fn cron_remove(args: &[Value]) -> Result<Value, String> {
    let _cron = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _job = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn cron_start(args: &[Value]) -> Result<Value, String> {
    let _cron = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn cron_stop(args: &[Value]) -> Result<Value, String> {
    let _cron = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Bool(true))
}

pub fn cron_list(args: &[Value]) -> Result<Value, String> {
    let _cron = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::List(vec![]))
}
