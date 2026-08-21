use crate::ir::Value;
use super::PrimitiveError;

pub fn hash_sha256(args: &[Value]) -> Result<Value, PrimitiveError> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None })
        .ok_or_else(|| PrimitiveError("hash_sha256 requires data:string".into()))?;
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(Value::String(format!("{result:x}")))
}

pub fn hash_blake3(args: &[Value]) -> Result<Value, PrimitiveError> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None })
        .ok_or_else(|| PrimitiveError("hash_blake3 requires data:string".into()))?;
    let hash = blake3::hash(data);
    Ok(Value::String(hash.to_hex().to_string()))
}

pub fn hash_md5(args: &[Value]) -> Result<Value, PrimitiveError> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None })
        .ok_or_else(|| PrimitiveError("hash_md5 requires data:string".into()))?;
    let result = md5::compute(data);
    Ok(Value::String(format!("{result:x}")))
}

pub fn encode_base64(args: &[Value]) -> Result<Value, PrimitiveError> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None })
        .ok_or_else(|| PrimitiveError("encode_base64 requires data:string".into()))?;
    use base64::Engine as _;
    Ok(Value::String(base64::engine::general_purpose::STANDARD.encode(data)))
}

pub fn decode_base64(args: &[Value]) -> Result<Value, PrimitiveError> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("decode_base64 requires text:string".into()))?;
    use base64::Engine as _;
    let data = base64::engine::general_purpose::STANDARD.decode(text)
        .map_err(|e| PrimitiveError(format!("decode_base64: {e}")))?;
    Ok(Value::String(String::from_utf8_lossy(&data).to_string()))
}

pub fn encode_hex(args: &[Value]) -> Result<Value, PrimitiveError> {
    let data = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_bytes()), _ => None })
        .ok_or_else(|| PrimitiveError("encode_hex requires data:string".into()))?;
    Ok(Value::String(data.iter().map(|b| format!("{b:02x}")).collect()))
}

pub fn decode_hex(args: &[Value]) -> Result<Value, PrimitiveError> {
    let text = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("decode_hex requires text:string".into()))?;
    let data: Vec<u8> = (0..text.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&text[i..i+2], 16).ok())
        .collect();
    Ok(Value::String(String::from_utf8_lossy(&data).to_string()))
}

pub fn crypto_random_bytes(args: &[Value]) -> Result<Value, PrimitiveError> {
    let n = args.first().and_then(|v| match v { Value::Int(n) => Some(*n as usize), _ => None })
        .unwrap_or(32);
    let mut bytes = vec![0u8; n];
    getrandom::getrandom(&mut bytes).map_err(|e| PrimitiveError(format!("crypto_random_bytes: {e}")))?;
    Ok(Value::String(bytes.iter().map(|b| format!("{b:02x}")).collect()))
}
