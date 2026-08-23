use crate::ir::Value;
use super::PrimitiveError;
use std::io::Read;

fn http_response_map(response: ureq::Response) -> Result<Value, PrimitiveError> {
    let status = response.status();
    let mut body = String::new();
    if let Err(e) = response.into_reader().read_to_string(&mut body) {
        return Err(PrimitiveError(format!("http read: {e}")));
    }
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status as i64)),
        (Value::String("body".into()), Value::String(body)),
    ]))
}

pub fn http_get(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_get requires url:string".into()))?;
    match ureq::get(url).call() {
        Ok(response) => http_response_map(response),
        Err(e) => Err(PrimitiveError(format!("http_get: {e}"))),
    }
}

pub fn http_post(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_post requires url:string".into()))?;
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    match ureq::post(url).set("Content-Type", "application/json").send_bytes(body.as_bytes()) {
        Ok(response) => http_response_map(response),
        Err(e) => Err(PrimitiveError(format!("http_post: {e}"))),
    }
}

pub fn http_put(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_put requires url:string".into()))?;
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    match ureq::put(url).set("Content-Type", "application/json").send_bytes(body.as_bytes()) {
        Ok(response) => http_response_map(response),
        Err(e) => Err(PrimitiveError(format!("http_put: {e}"))),
    }
}

pub fn http_delete(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_delete requires url:string".into()))?;
    match ureq::delete(url).call() {
        Ok(response) => http_response_map(response),
        Err(e) => Err(PrimitiveError(format!("http_delete: {e}"))),
    }
}

pub fn http_patch(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_patch requires url:string".into()))?;
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    match ureq::patch(url).set("Content-Type", "application/json").send_bytes(body.as_bytes()) {
        Ok(response) => http_response_map(response),
        Err(e) => Err(PrimitiveError(format!("http_patch: {e}"))),
    }
}

pub fn http_download(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_download requires url:string".into()))?;
    let path = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_download requires path:string".into()))?;
    match ureq::get(url).call() {
        Ok(response) => {
            let mut content = Vec::new();
            response.into_reader().read_to_end(&mut content).map_err(|e| PrimitiveError(format!("http_download read: {e}")))?;
            std::fs::write(path, &content).map_err(|e| PrimitiveError(format!("http_download write: {e}")))?;
            Ok(Value::Map(vec![
                (Value::String("status".into()), Value::Int(200)),
                (Value::String("bytes".into()), Value::Int(content.len() as i64)),
                (Value::String("path".into()), Value::String(path.into())),
            ]))
        }
        Err(e) => Err(PrimitiveError(format!("http_download: {e}"))),
    }
}
