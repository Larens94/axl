use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::ir::Value;

// Thread-safe storage for routes, handlers, and state
lazy_static::lazy_static! {
    static ref ROUTES: Arc<Mutex<Vec<(String, String, String)>>> = Arc::new(Mutex::new(Vec::new())); // (method, path, handler_id)
    static ref STATIC_DIR: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref STATE: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref REQUEST_BODY: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref REQUEST_METHOD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref REQUEST_PATH_STR: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref REQUEST_QUERY: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref REQUEST_HEADERS: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn parse_request(stream: &mut TcpStream) -> (String, String, HashMap<String, String>, String, HashMap<String, String>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap_or(0);

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET").to_string();
    let full_path = parts.get(1).unwrap_or(&"/").to_string();

    // Parse path and query string
    let (path, query_string) = if let Some(qpos) = full_path.find('?') {
        (full_path[..qpos].to_string(), full_path[qpos+1..].to_string())
    } else {
        (full_path.clone(), String::new())
    };

    // Parse query parameters
    let mut query_params = HashMap::new();
    for pair in query_string.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            query_params.insert(k.to_string(), v.to_string());
        }
    }

    // Parse headers
    let mut headers = HashMap::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap_or(0);
        let line = line.trim().to_string();
        if line.is_empty() { break; }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            if key == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers.insert(key, val);
        }
    }

    // Read body
    let mut body = String::new();
    if content_length > 0 {
        let mut buffer = vec![0u8; content_length];
        reader.read_exact(&mut buffer).unwrap_or(());
        body = String::from_utf8_lossy(&buffer).to_string();
    }

    // Store in global state for request_* primitives
    *REQUEST_METHOD.lock().unwrap() = method.clone();
    *REQUEST_PATH_STR.lock().unwrap() = path.clone();
    *REQUEST_QUERY.lock().unwrap() = query_params.clone();
    *REQUEST_BODY.lock().unwrap() = body.clone();
    *REQUEST_HEADERS.lock().unwrap() = headers.clone();

    (method, path, query_params, body, headers)
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

fn render_json_value(val: &Value) -> String {
    match val {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
        Value::List(items) => {
            let rendered: Vec<String> = items.iter().map(render_json_value).collect();
            format!("[{}]", rendered.join(","))
        }
        Value::Map(entries) => {
            let rendered: Vec<String> = entries.iter().map(|(k, v)| {
                let key = match k {
                    Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
                    _ => format!("\"{}\"", render_json_value(k)),
                };
                format!("{}:{}", key, render_json_value(v))
            }).collect();
            format!("{{{}}}", rendered.join(","))
        }
        Value::Embedding(e) => {
            let rendered: Vec<String> = e.iter().map(|n| n.to_string()).collect();
            format!("[{}]", rendered.join(","))
        }
        Value::AgentRef(r) => format!("\"<agent:{}>\"", r),
    }
}

fn match_route(method: &str, path: &str) -> Option<String> {
    let routes = ROUTES.lock().unwrap();
    for (route_method, route_path, handler) in routes.iter() {
        if route_method == method || route_method == "*" {
            if route_path == path {
                return Some(handler.clone());
            }
        }
    }
    None
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
    let handler = args.get(3).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("default");
    ROUTES.lock().unwrap().push((method.to_uppercase(), path.to_string(), handler.to_string()));
    Ok(Value::Map(vec![
        (Value::String("method".into()), Value::String(method.into())),
        (Value::String("path".into()), Value::String(path.into())),
        (Value::String("handler".into()), Value::String(handler.into())),
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
    let routes_count = ROUTES.lock().unwrap().len();
    if routes_count > 0 {
        println!("Registered routes: {routes_count}");
    }

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let (method, path, _query, _body, _headers) = parse_request(&mut stream);

                // Handle CORS preflight
                if method == "OPTIONS" {
                    send_response(&mut stream, 204, "text/plain", "");
                    continue;
                }

                // Check dynamic routes first
                if let Some(_handler) = match_route(&method, &path) {
                    // For now, return a stub response — full handler dispatch requires interpreter integration
                    let body = format!("{{\"handler\":\"{}\",\"method\":\"{}\",\"path\":\"{}\"}}", _handler, method, path);
                    send_response(&mut stream, 200, "application/json", &body);
                    continue;
                }

                // Serve static files
                if !static_dir.is_empty() && (method == "GET" || method == "HEAD") {
                    let file_path = if path == "/" {
                        format!("{}/index.html", static_dir.trim_end_matches('/'))
                    } else {
                        format!("{}/{}", static_dir.trim_end_matches('/'), path.trim_start_matches('/'))
                    };
                    if let Ok(content) = fs::read(&file_path) {
                        let ct = match file_path.rsplit('.').next() {
                            Some("html") => "text/html; charset=utf-8",
                            Some("css") => "text/css; charset=utf-8",
                            Some("js") => "text/javascript; charset=utf-8",
                            Some("json") => "application/json",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("svg") => "image/svg+xml",
                            Some("ico") => "image/x-icon",
                            Some("woff") => "font/woff",
                            Some("woff2") => "font/woff2",
                            _ => "application/octet-stream",
                        };
                        let content_str = String::from_utf8_lossy(&content).to_string();
                        send_response(&mut stream, 200, ct, &content_str);
                        continue;
                    }
                }

                // Default: return method/path info as JSON
                let body = format!("{{\"method\":\"{}\",\"path\":\"{}\"}}", method, path);
                send_response(&mut stream, 200, "application/json", &body);
            }
            Err(_) => break,
        }
    }

    Ok(Value::String(format!("server stopped on {addr}")))
}

pub fn http_response(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n as u16), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status as i64)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_response_json(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n as u16), _ => None }).unwrap_or(200);
    let body = args.get(1).cloned().unwrap_or(Value::Null);
    let json_str = render_json_value(&body);
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status as i64)),
        (Value::String("body".into()), Value::String(json_str)),
        (Value::String("content_type".into()), Value::String("application/json".into())),
    ]))
}

pub fn http_response_html(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n as u16), _ => None }).unwrap_or(200);
    let body = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status as i64)),
        (Value::String("body".into()), Value::String(body.into())),
        (Value::String("content_type".into()), Value::String("text/html; charset=utf-8".into())),
    ]))
}

pub fn http_response_error(args: &[Value]) -> Result<Value, String> {
    let status = args.first().and_then(|v| match v { Value::Int(n) => Some(*n as u16), _ => None }).unwrap_or(500);
    let message = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("Internal Server Error");
    Ok(Value::Map(vec![
        (Value::String("status".into()), Value::Int(status as i64)),
        (Value::String("body".into()), Value::String(message.into())),
        (Value::String("content_type".into()), Value::String("text/plain".into())),
    ]))
}

pub fn http_request_method(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let method = REQUEST_METHOD.lock().unwrap().clone();
    Ok(Value::String(method))
}

pub fn http_request_path(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let path = REQUEST_PATH_STR.lock().unwrap().clone();
    Ok(Value::String(path))
}

pub fn http_request_query(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let query = REQUEST_QUERY.lock().unwrap();
    let entries: Vec<(Value, Value)> = query.iter()
        .map(|(k, v)| (Value::String(k.clone()), Value::String(v.clone())))
        .collect();
    Ok(Value::Map(entries))
}

pub fn http_request_body(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let body = REQUEST_BODY.lock().unwrap().clone();
    Ok(Value::String(body))
}

pub fn http_request_header(args: &[Value]) -> Result<Value, String> {
    let _req = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let name = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let headers = REQUEST_HEADERS.lock().unwrap();
    match headers.get(&name.to_lowercase()) {
        Some(v) => Ok(Value::String(v.clone())),
        None => Ok(Value::Null),
    }
}

pub fn http_server_state_get(args: &[Value]) -> Result<Value, String> {
    let key = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let state = STATE.lock().unwrap();
    Ok(state.get(key).cloned().unwrap_or(Value::Null))
}

pub fn http_server_state_set(args: &[Value]) -> Result<Value, String> {
    let key = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("");
    let value = args.get(1).cloned().unwrap_or(Value::Null);
    STATE.lock().unwrap().insert(key.to_string(), value);
    Ok(Value::Bool(true))
}

pub fn axl_server_start(args: &[Value]) -> Result<Value, String> {
    let addr = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("127.0.0.1:8080");
    let static_dir = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or("./public");
    let db_path = args.get(2).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).unwrap_or(":memory:");

    let mut server = crate::server::AxlServer::new(addr, static_dir, db_path);

    // If a 4th arg is a list of table names, add generic CRUD routes for each
    if let Some(Value::List(tables)) = args.get(3) {
        for table_val in tables {
            if let Value::String(table_name) = table_val {
                server.add_table_routes(table_name);
                println!("  API routes: /api/{}", table_name);
            }
        }
    }

    println!("AXL Server on {addr} | db={db_path} | static={static_dir}");

    server.run().map_err(|e| format!("axl_server_start: {e}"))?;
    Ok(Value::String(format!("server stopped on {addr}")))
}

/// Register generic CRUD API routes for a database table.
/// Usage: http_server_api(db_path, table_name)
/// Creates: GET /api/{table}, GET /api/{table}/:id, POST /api/{table}, PUT /api/{table}/:id, DELETE /api/{table}/:id
pub fn http_server_api(args: &[Value]) -> Result<Value, String> {
    let db_path = args.first().and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or("http_server_api: missing db_path")?;
    let table = args.get(1).and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or("http_server_api: missing table name")?;

    let db = db_path.to_string();
    let tbl = table.to_string();
    let api_prefix = format!("/api/{}", tbl);

    // GET /api/{table} — list all rows
    let db_c = db.clone();
    let tbl_c = tbl.clone();
    ROUTES.lock().unwrap().push(("GET".into(), api_prefix.clone(), format!("list_{}", tbl_c)));

    // POST /api/{table} — create new row
    let db_c = db.clone();
    let tbl_c = tbl.clone();
    ROUTES.lock().unwrap().push(("POST".into(), api_prefix.clone(), format!("create_{}", tbl_c)));

    // GET /api/{table}/:id — get by id (prefix match)
    let db_c = db.clone();
    let tbl_c = tbl.clone();
    ROUTES.lock().unwrap().push(("GET".into(), format!("{}/", api_prefix), format!("get_{}", tbl_c)));

    // PUT /api/{table}/:id — update by id
    let db_c = db.clone();
    let tbl_c = tbl.clone();
    ROUTES.lock().unwrap().push(("PUT".into(), format!("{}/", api_prefix), format!("update_{}", tbl_c)));

    // DELETE /api/{table}/:id — delete by id
    let db_c = db.clone();
    let tbl_c = tbl.clone();
    ROUTES.lock().unwrap().push(("DELETE".into(), format!("{}/", api_prefix), format!("delete_{}", tbl_c)));

    Ok(Value::Map(vec![
        (Value::String("table".into()), Value::String(tbl.into())),
        (Value::String("routes".into()), Value::Int(5)),
        (Value::String("status".into()), Value::String("registered".into())),
    ]))
}
