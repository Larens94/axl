use crate::ir::Value;
use super::PrimitiveError;

pub fn text_upper(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_upper requires text:string".into()))?;
    Ok(Value::String(s.to_uppercase()))
}

pub fn text_lower(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_lower requires text:string".into()))?;
    Ok(Value::String(s.to_lowercase()))
}

pub fn text_trim(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_trim requires text:string".into()))?;
    Ok(Value::String(s.trim().to_string()))
}

pub fn text_replace(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_replace requires text:string".into()))?;
    let from = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_replace requires from:string".into()))?;
    let to = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_replace requires to:string".into()))?;
    Ok(Value::String(s.replace(from, to)))
}

pub fn text_split(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_split requires text:string".into()))?;
    let delim = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .unwrap_or("");
    let parts: Vec<Value> = s.split(delim).map(|p| Value::String(p.to_string())).collect();
    Ok(Value::List(parts))
}

pub fn text_join(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("text_join requires list:list".into()))?;
    let delim = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .unwrap_or("");
    let parts: Vec<String> = list.iter().filter_map(|v| match v { Value::String(s) => Some(s.clone()), _ => None }).collect();
    Ok(Value::String(parts.join(delim)))
}

pub fn text_find(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_find requires text:string".into()))?;
    let pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_find requires pattern:string".into()))?;
    let indices: Vec<Value> = s.match_indices(pattern).map(|(i, _)| Value::Int(i as i64)).collect();
    Ok(Value::List(indices))
}

pub fn text_contains(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_contains requires text:string".into()))?;
    let pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_contains requires pattern:string".into()))?;
    Ok(Value::Bool(s.contains(pattern)))
}

pub fn text_matches(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_matches requires text:string".into()))?;
    let pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_matches requires pattern:string".into()))?;
    let re = regex::Regex::new(pattern).map_err(|e| PrimitiveError(format!("text_matches: {e}")))?;
    Ok(Value::Bool(re.is_match(s)))
}

pub fn text_length(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_length requires text:string".into()))?;
    Ok(Value::Int(s.len() as i64))
}

pub fn text_reverse(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_reverse requires text:string".into()))?;
    Ok(Value::String(s.chars().rev().collect()))
}

pub fn text_lines(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_lines requires text:string".into()))?;
    let lines: Vec<Value> = s.lines().map(|l| Value::String(l.to_string())).collect();
    Ok(Value::List(lines))
}

pub fn text_extract(args: &[Value]) -> Result<Value, PrimitiveError> {
    let s = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_extract requires text:string".into()))?;
    let pattern = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("text_extract requires pattern:string".into()))?;
    let re = regex::Regex::new(pattern).map_err(|e| PrimitiveError(format!("text_extract: {e}")))?;
    let captures: Vec<Value> = re.captures_iter(s)
        .filter_map(|c| c.get(1).or_else(|| c.get(0)))
        .map(|m| Value::String(m.as_str().to_string()))
        .collect();
    Ok(Value::List(captures))
}
