use crate::ir::Value;
use super::PrimitiveError;

pub fn env_get(args: &[Value]) -> Result<Value, PrimitiveError> {
    let name = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("env_get requires name:string".into()))?;
    match std::env::var(name) {
        Ok(val) => Ok(Value::String(val)),
        Err(_) => Ok(Value::Null),
    }
}

pub fn env_set(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Err(PrimitiveError("env_set not available (unsafe forbidden)".into()))
}

pub fn env_list(_args: &[Value]) -> Result<Value, PrimitiveError> {
    let entries: Vec<(Value, Value)> = std::env::vars()
        .map(|(k, v)| (Value::String(k), Value::String(v)))
        .collect();
    Ok(Value::Map(entries))
}

pub fn time_now(_args: &[Value]) -> Result<Value, PrimitiveError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    Ok(Value::Int(t.as_millis() as i64))
}

pub fn time_format(args: &[Value]) -> Result<Value, PrimitiveError> {
    let ts = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None })
        .ok_or_else(|| PrimitiveError("time_format requires timestamp:int".into()))?;
    let _format = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .unwrap_or("%Y-%m-%d %H:%M:%S");
    use std::time::UNIX_EPOCH;
    let secs = (ts / 1000) as u64;
    let dt = UNIX_EPOCH + std::time::Duration::from_secs(secs);
    let Ok(sys_time) = dt.duration_since(UNIX_EPOCH) else {
        return Ok(Value::String(format!("timestamp:{ts}")));
    };
    let secs = sys_time.as_secs();
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let month = ((days % 365) / 30 + 1).min(12);
    let day = (days % 365) % 30 + 1;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    Ok(Value::String(format!("{year:04}-{month:02}-{day:02} {hours:02}:{minutes:02}:{seconds:02}")))
}

pub fn time_sleep(args: &[Value]) -> Result<Value, PrimitiveError> {
    let ms = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None })
        .ok_or_else(|| PrimitiveError("time_sleep requires ms:int".into()))?;
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    Ok(Value::Bool(true))
}

pub fn path_join(args: &[Value]) -> Result<Value, PrimitiveError> {
    let parts: Vec<&str> = args.iter().filter_map(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).collect();
    let joined = parts.join("/");
    Ok(Value::String(joined))
}

pub fn path_absolute(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("path_absolute requires path:string".into()))?;
    let abs = std::fs::canonicalize(path).map_err(|e| PrimitiveError(format!("path_absolute: {e}")))?;
    Ok(Value::String(abs.to_string_lossy().to_string()))
}

pub fn path_parent(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("path_parent requires path:string".into()))?;
    let p = std::path::Path::new(path);
    match p.parent() {
        Some(parent) => Ok(Value::String(parent.to_string_lossy().to_string())),
        None => Ok(Value::Null),
    }
}

pub fn path_filename(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("path_filename requires path:string".into()))?;
    let p = std::path::Path::new(path);
    match p.file_name() {
        Some(name) => Ok(Value::String(name.to_string_lossy().to_string())),
        None => Ok(Value::Null),
    }
}

pub fn path_extension(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("path_extension requires path:string".into()))?;
    let p = std::path::Path::new(path);
    match p.extension() {
        Some(ext) => Ok(Value::String(ext.to_string_lossy().to_string())),
        None => Ok(Value::Null),
    }
}

pub fn path_exists(args: &[Value]) -> Result<Value, PrimitiveError> {
    let path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("path_exists requires path:string".into()))?;
    Ok(Value::Bool(std::path::Path::new(path).exists()))
}

pub fn temp_dir(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::String(std::env::temp_dir().to_string_lossy().to_string()))
}

pub fn temp_file(_args: &[Value]) -> Result<Value, PrimitiveError> {
    let name = format!("axl_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
    let path = std::env::temp_dir().join(name);
    Ok(Value::String(path.to_string_lossy().to_string()))
}

pub fn sys_hostname(_args: &[Value]) -> Result<Value, PrimitiveError> {
    let hostname = hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_else(|_| "unknown".into());
    Ok(Value::String(hostname))
}

pub fn sys_os(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::String(std::env::consts::OS.to_string()))
}

pub fn sys_arch(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::String(std::env::consts::ARCH.to_string()))
}

pub fn process_run(args: &[Value]) -> Result<Value, PrimitiveError> {
    let cmd = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("process_run requires cmd:string".into()))?;
    let output = std::process::Command::new(cmd)
        .output()
        .map_err(|e| PrimitiveError(format!("process_run: {e}")))?;
    Ok(Value::Map(vec![
        (Value::String("stdout".into()), Value::String(String::from_utf8_lossy(&output.stdout).to_string())),
        (Value::String("stderr".into()), Value::String(String::from_utf8_lossy(&output.stderr).to_string())),
        (Value::String("status".into()), Value::Int(output.status.code().unwrap_or(-1) as i64)),
    ]))
}

pub fn process_output(args: &[Value]) -> Result<Value, PrimitiveError> {
    let cmd = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("process_output requires cmd:string".into()))?;
    let output = std::process::Command::new(cmd)
        .output()
        .map_err(|e| PrimitiveError(format!("process_output: {e}")))?;
    Ok(Value::String(String::from_utf8_lossy(&output.stdout).to_string()))
}
