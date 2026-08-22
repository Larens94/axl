use crate::ir::Value;

pub fn gzip_compress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    // Simplified: just return the data
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}

pub fn gzip_decompress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}

pub fn zstd_compress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}

pub fn zstd_decompress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}

pub fn brotli_compress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}

pub fn brotli_decompress(args: &[Value]) -> Result<Value, String> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None }).unwrap_or(b"");
    Ok(Value::String(String::from_utf8_lossy(data).to_string()))
}
