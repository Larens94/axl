use crate::ir::Value;

pub fn metric_counter(args: &[Value]) -> Result<Value, String> {
    let _name = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _value = args.get(1).and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(1);
    Ok(Value::Bool(true))
}

pub fn metric_gauge(args: &[Value]) -> Result<Value, String> {
    let _name = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _value = args.get(1).and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(0);
    Ok(Value::Bool(true))
}

pub fn metric_histogram(args: &[Value]) -> Result<Value, String> {
    let _name = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _value = args.get(1).and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(0);
    Ok(Value::Bool(true))
}

pub fn metric_timer_start(args: &[Value]) -> Result<Value, String> {
    let _name = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("timer_1".into()))
}

pub fn metric_timer_stop(args: &[Value]) -> Result<Value, String> {
    let _timer = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Int(0))
}
