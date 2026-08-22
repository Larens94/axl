use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::ir::Value;

// Thread-safe storage for routes and static dir
lazy_static::lazy_static! {
    static ref ROUTES: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
    static ref STATIC_DIR: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

pub fn http_server_create(args: &[Value]) -> Result<Value, String> {
    let port = args.first().and_then(|v| match v {
        Value::Int(n) => Some(*n as u16),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }).unwrap_or(8080);
    Ok(Value::String(format!("server_{port}")))
}

pub fn http_server_route(args: &[Value]) -> Result<Value, String> {
    let _server = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let method = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("GET");
    let path = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("/");
    let route_key = format!("{} {}", method, path);
    ROUTES.lock().unwrap().push((route_key, path.to_string()));
    Ok(Value::Map(vec![
        (Value::String("method".into()), Value::String(method.into())),
        (Value::String("path".into()), Value::String(path.into())),
        (Value::String("status".into()), Value::String("registered".into())),
    ]))
}

pub fn http_server_static(args: &[Value]) -> Result<Value, String> {
    let dir = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("./public");
    *STATIC_DIR.lock().unwrap() = dir.to_string();
    Ok(Value::Map(vec![
        (Value::String("static_dir".into()), Value::String(dir.into())),
        (Value::String("status".into()), Value::String("configured".into())),
    ]))
}

pub fn http_server_listen(args: &[Value]) -> Result<Value, String> {
    let addr = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("127.0.0.1:8080");
    let static_dir = STATIC_DIR.lock().unwrap().clone();
    
    let listener = TcpListener::bind(addr).map_err(|e| format!("http_listen: {e}"))?;
    println!("AXL HTTP Server listening on {addr}");
    if !static_dir.is_empty() {
        println!("Serving static files from: {static_dir}");
    }
    
    // Serve requests (blocking)
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut request = [0u8; 8192];
                let size = stream.read(&mut request).unwrap_or(0);
                let request_str = String::from_utf8_lossy(&request[..size]);
                let first_line = request_str.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                let method = parts.first().unwrap_or(&"GET");
                let path = parts.get(1).unwrap_or(&"/");
                
                // Check for static file
                if !static_dir.is_empty() && *method == "GET" {
                    let file_path = if *path == "/" {
                        format!("{}/index.html", static_dir.trim_end_matches('/'))
                    } else {
                        format!("{}/{}", static_dir.trim_end_matches('/'), path.trim_start_matches('/'))
                    };
                    if let Ok(content) = fs::read(&file_path) {
                        let ct = match file_path.rsplit('.').next() {
                            Some("html") => "text/html; charset=utf-8",
                            Some("css") => "text/css; charset=utf-8",
                            Some("js") => "text/javascript; charset=utf-8",
                            _ => "application/octet-stream",
                        };
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            content.len()
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.write_all(&content);
                        continue;
                    }
                }
                
                // Default JSON response
                let body = format!("{{\"method\":\"{method}\",\"path\":\"{path}\"}}");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
            Err(_) => break,
        }
    }
    
    Ok(Value::String(format!("server stopped on {addr}")))
}

pub fn http_response(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_response_json(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).cloned().unwrap_or(Value::Null);
    let json_str = format!("{body:?}");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(json_str)),
        (Value::String("content_type".into()), Value::String("application/json".into())),
    ]))
}

pub fn http_response_html(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/html; charset=utf-8".into())),
    ]))
}

pub fn http_response_error(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n), _ => None }).unwrap_or(500);
    let message = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("Internal Server Error");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status)),
        (Value::String("body".into()), Value::String(message.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_request_method(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("GET".into()))
}

pub fn http_request_path(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("/");
    Ok(Value::String("/".into()))
}

pub fn http_request_query(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![]))
}

pub fn http_request_body(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::String("".into()))
}

pub fn http_request_header(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let _name = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Null)
}
