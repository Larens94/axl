use crate::ir::Value;
use super::PrimitiveError;

pub fn file_read(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_read requires path:string".into()))?;
    let data = std::fs::read(path).map_err(|e| PrimitiveError(format!("file_read: {e}")))?;
    Ok(Value::String(String::from_utf8_lossy(&data).to_string()))
}

pub fn file_write(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_write requires path:string".into()))?;
    let content = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_write requires content:string".into()))?;
    std::fs::write(path, content).map_err(|e| PrimitiveError(format!("file_write: {e}")))?;
    Ok(Value::Bool(true))
}

pub fn file_exists(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_exists requires path:string".into()))?;
    Ok(Value::Bool(std::path::Path::new(path).exists()))
}

pub fn file_size(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_size requires path:string".into()))?;
    let meta = std::fs::metadata(path).map_err(|e| PrimitiveError(format!("file_size: {e}")))?;
    Ok(Value::Int(meta.len() as i64))
}

pub fn file_delete(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_delete requires path:string".into()))?;
    std::fs::remove_file(path).map_err(|e| PrimitiveError(format!("file_delete: {e}")))?;
    Ok(Value::Bool(true))
}

pub fn file_copy(args: &[Value]) -> Result<Value, PrimitiveError> {
    let src = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_copy requires src:string".into()))?;
    let dst = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_copy requires dst:string".into()))?;
    std::fs::copy(src, dst).map_err(|e| PrimitiveError(format!("file_copy: {e}")))?;
    Ok(Value::Bool(true))
}

pub fn file_move(args: &[Value]) -> Result<Value, PrimitiveError> {
    let src = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_move requires src:string".into()))?;
    let dst = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("file_move requires dst:string".into()))?;
    std::fs::rename(src, dst).map_err(|e| PrimitiveError(format!("file_move: {e}")))?;
    Ok(Value::Bool(true))
}

pub fn dir_create(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("dir_create requires path:string".into()))?;
    std::fs::create_dir_all(path).map_err(|e| PrimitiveError(format!("dir_create: {e}")))?;
    Ok(Value::Bool(true))
}

pub fn dir_list(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("dir_list requires path:string".into()))?;
    let entries: Vec<Value> = std::fs::read_dir(path)
        .map_err(|e| PrimitiveError(format!("dir_list: {e}")))?
        .filter_map(|e| e.ok())
        .map(|e| Value::String(e.file_name().to_string_lossy().to_string()))
        .collect();
    Ok(Value::List(entries))
}

pub fn dir_delete(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("dir_delete requires path:string".into()))?;
    std::fs::remove_dir_all(path).map_err(|e| PrimitiveError(format!("dir_delete: {e}")))?;
    Ok(Value::Bool(true))
}
