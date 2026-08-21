use crate::ir::Value;
use super::PrimitiveError;

pub fn http_get(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_get requires url:string".into()))?;
    match ureq::get(url).call() {
        Ok(response) => {
            let mut body = String::new();
            if let Err(e) = response.into_reader().read_to_string(&mut body) {
                return Err(PrimitiveError(format!("http_get read: {e}")));
            }
            Ok(Value::Map(vec![
                (Value::String("status".into()), Value::Int(200)),
                (Value::String("body".into()), Value::String(body)),
            ]))
        }
        Err(e) => Err(PrimitiveError(format!("http_get: {e}"))),
    }
}

pub fn http_post(args: &[Value]) -> Result<Value, PrimitiveError> {
    let url = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("http_post requires url:string".into()))?;
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .unwrap_or("");
    match ureq::post(url).set("Content-Type", "application/json").send_bytes(body.as_bytes()) {
        Ok(response) => {
            let mut response_body = String::new();
            if let Err(e) = response.into_reader().read_to_string(&mut response_body) {
                return Err(PrimitiveError(format!("http_post read: {e}")));
            }
            Ok(Value::Map(vec![
                (Value::String("status".into()), Value::Int(200)),
                (Value::String("body".into()), Value::String(response_body)),
            ]))
        }
        Err(e) => Err(PrimitiveError(format!("http_post: {e}"))),
    }
}
