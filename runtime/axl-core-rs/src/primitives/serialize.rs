use crate::ir::Value;
use super::PrimitiveError;

pub fn json_parse(args: &[Value]) -> Result<Value, PrimitiveError> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("json_parse requires text:string".into()))?;
    let json: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| PrimitiveError(format!("json_parse: {e}")))?;
    json_to_value(&json)
}

pub fn json_stringify(args: &[Value]) -> Result<Value, PrimitiveError> {
    let value = args.first().cloned().ok_or_else(|| PrimitiveError("json_stringify requires value".into()))?;
    let json = value_to_json(&value)?;
    Ok(Value::String(serde_json::to_string(&json).unwrap_or_default()))
}

pub fn json_validate(args: &[Value]) -> Result<Value, PrimitiveError> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("json_validate requires text:string".into()))?;
    Ok(Value::Bool(serde_json::from_str::<serde_json::Value>(text).is_ok()))
}

fn json_to_value(json: &serde_json::Value) -> Result<Value, PrimitiveError> {
    match json {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => Ok(Value::Int(n.as_i64().unwrap_or(0))),
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(|v| json_to_value(v)).collect::<Result<_, _>>()?;
            Ok(Value::List(items))
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<(Value, Value)> = obj.iter()
                .map(|(k, v)| Ok((Value::String(k.clone()), json_to_value(v)?)))
                .collect::<Result<_, PrimitiveError>>()?;
            Ok(Value::Map(entries))
        }
    }
}

fn value_to_json(value: &Value) -> Result<serde_json::Value, PrimitiveError> {
    match value {
        Value::Null => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::json!(b)),
        Value::Int(n) => Ok(serde_json::json!(n)),
        Value::String(s) => Ok(serde_json::json!(s)),
        Value::List(items) => {
            let arr: Vec<serde_json::Value> = items.iter().map(|v| value_to_json(v)).collect::<Result<_, _>>()?;
            Ok(serde_json::Value::Array(arr))
        }
        Value::Map(entries) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in entries {
                if let Value::String(key) = k {
                    obj.insert(key.clone(), value_to_json(v)?);
                }
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Ok(serde_json::json!(format!("{value:?}"))),
    }
}
