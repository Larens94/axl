use crate::ir::Value;
use super::PrimitiveError;

fn int_arg(args: &[Value], idx: usize) -> Result<i64, PrimitiveError> {
    args.get(idx).and_then(|v| match v { Value::Int(n) => Some(*n), _ => None })
        .ok_or_else(|| PrimitiveError(format!("argument {idx} must be int")))
}

pub fn math_add(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)? + int_arg(args, 1)?))
}

pub fn math_sub(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)? - int_arg(args, 1)?))
}

pub fn math_mul(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)? * int_arg(args, 1)?))
}

pub fn math_div(args: &[Value]) -> Result<Value, PrimitiveError> {
    let b = int_arg(args, 1)?;
    if b == 0 { return Err(PrimitiveError("division by zero".into())); }
    Ok(Value::Int(int_arg(args, 0)? / b))
}

pub fn math_mod(args: &[Value]) -> Result<Value, PrimitiveError> {
    let b = int_arg(args, 1)?;
    if b == 0 { return Err(PrimitiveError("modulo by zero".into())); }
    Ok(Value::Int(int_arg(args, 0)? % b))
}

pub fn math_pow(args: &[Value]) -> Result<Value, PrimitiveError> {
    let base = int_arg(args, 0)?;
    let exp = int_arg(args, 1)?;
    Ok(Value::Int(base.pow(exp as u32)))
}

pub fn math_abs(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)?.abs()))
}

pub fn math_min(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)?.min(int_arg(args, 1)?)))
}

pub fn math_max(args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Int(int_arg(args, 0)?.max(int_arg(args, 1)?)))
}

pub fn math_clamp(args: &[Value]) -> Result<Value, PrimitiveError> {
    let val = int_arg(args, 0)?;
    let min = int_arg(args, 1)?;
    let max = int_arg(args, 2)?;
    Ok(Value::Int(val.max(min).min(max)))
}

pub fn math_random(_args: &[Value]) -> Result<Value, PrimitiveError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(Value::Int((t.as_nanos() % 1000000) as i64))
}

pub fn math_random_range(args: &[Value]) -> Result<Value, PrimitiveError> {
    let min = int_arg(args, 0)?;
    let max = int_arg(args, 1)?;
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let range = (max - min).abs() as u64 + 1;
    Ok(Value::Int(min + (t.as_nanos() % range as u128) as i64))
}

pub fn math_sum_list(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("math_sum requires list".into()))?;
    let sum: i64 = list.iter().filter_map(|v| match v { Value::Int(n) => Some(*n), _ => None }).sum();
    Ok(Value::Int(sum))
}

pub fn math_average(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("math_average requires list".into()))?;
    let nums: Vec<i64> = list.iter().filter_map(|v| match v { Value::Int(n) => Some(*n), _ => None }).collect();
    if nums.is_empty() { return Err(PrimitiveError("math_average: empty list".into())); }
    Ok(Value::Int(nums.iter().sum::<i64>() / nums.len() as i64))
}
