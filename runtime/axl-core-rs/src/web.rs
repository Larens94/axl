use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use crate::ir::Value;
use crate::primitives;

/// HTTP Request
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// HTTP Response
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn ok() -> Self {
        Self { status: 200, headers: HashMap::new(), body: vec![] }
    }

    pub fn json(value: &Value) -> Self {
        let body = serde_json::to_string_pretty(&value_to_json(value)).unwrap_or_default();
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "application/json; charset=utf-8".into());
        headers.insert("Access-Control-Allow-Origin".into(), "*".into());
        Self { status: 200, headers, body: body.into_bytes() }
    }

    pub fn html(content: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "text/html; charset=utf-8".into());
        Self { status: 200, headers, body: content.as_bytes().to_vec() }
    }

    pub fn text(content: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "text/plain; charset=utf-8".into());
        Self { status: 200, headers, body: content.as_bytes().to_vec() }
    }

    pub fn not_found() -> Self {
        Self::text("Not Found").with_status(404)
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

/// Route handler function
pub type RouteHandler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

/// AXL Web Server
pub struct AxlServer {
    routes: Vec<(String, String, Box<dyn Fn(&Request) -> Response + Send + Sync>)>,
    static_dir: Option<PathBuf>,
    port: u16,
}

impl AxlServer {
    pub fn new(port: u16) -> Self {
        Self { routes: Vec::new(), static_dir: None, port }
    }

    pub fn static_files(mut self, dir: &str) -> Self {
        self.static_dir = Some(PathBuf::from(dir));
        self
    }

    pub fn get(mut self, path: &str, handler: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(("GET".into(), path.into(), Box::new(handler)));
        self
    }

    pub fn post(mut self, path: &str, handler: impl Fn(&Request) -> Response + Send + Sync + 'static) -> Self {
        self.routes.push(("POST".into(), path.into(), Box::new(handler)));
        self
    }

    /// API endpoint that calls a native primitive
    pub fn primitive_endpoint(self, path: &str, primitive_name: &str) -> Self {
        let name = primitive_name.to_string();
        self.post(path, move |req| {
            let args = parse_json_args(&req.body);
            match primitives::call_primitive(&name, &args) {
                Ok(result) => Response::json(&result),
                Err(e) => Response::json(&Value::Map(vec![
                    (Value::String("error".into()), Value::String(e.to_string())),
                ])).with_status(500),
            }
        })
    }

    /// API endpoint that calls a native primitive via GET with query params
    pub fn primitive_get_endpoint(self, path: &str, primitive_name: &str) -> Self {
        let name = primitive_name.to_string();
        self.get(path, move |req| {
            let args: Vec<Value> = req.query.values()
                .map(|v| Value::String(v.clone()))
                .collect();
            match primitives::call_primitive(&name, &args) {
                Ok(result) => Response::json(&result),
                Err(e) => Response::json(&Value::Map(vec![
                    (Value::String("error".into()), Value::String(e.to_string())),
                ])).with_status(500),
            }
        })
    }

    pub fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(("127.0.0.1", self.port))?;
        println!("AXL server: http://localhost:{}", self.port);

        let routes: Arc<Vec<_>> = Arc::new(self.routes);
        let static_dir = self.static_dir.map(|d| Arc::new(d));

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let routes = routes.clone();
                let static_dir = static_dir.clone();
                handle_connection(&mut stream, &routes, static_dir.as_deref());
            }
        }
        Ok(())
    }
}

fn handle_connection(stream: &mut TcpStream, routes: &[(String, String, RouteHandler)], static_dir: Option<&PathBuf>) {
    let mut request = [0u8; 8192];
    let size = stream.read(&mut request).unwrap_or(0);
    let first_line = String::from_utf8_lossy(&request[..size]);
    let lines: Vec<&str> = first_line.lines().collect();
    let request_line = lines.first().unwrap_or(&"");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET");
    let target = parts.get(1).copied().unwrap_or("/");

    let (path, query) = parse_url(target);

    let body_start = first_line.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = String::from_utf8_lossy(&request[body_start..size]).to_string();

    let headers: HashMap<String, String> = lines[1..].iter()
        .filter_map(|line| line.split_once(": "))
        .map(|(k, v)| (k.to_lowercase(), v.to_string()))
        .collect();

    let req = Request { method: method.to_string(), path: path.clone(), query, headers, body };

    // Check routes
    let mut response = None;
    for (route_method, route_path, handler) in routes {
        if route_method == method && (route_path == &path || route_path == "*") {
            response = Some(handler(&req));
            break;
        }
    }

    // Check static files
    if response.is_none() && method == "GET" {
        if let Some(dir) = static_dir {
            let file_path = dir.join(path.trim_start_matches('/'));
            if let Ok(content) = std::fs::read(&file_path) {
                let ct = match file_path.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("css") => "text/css; charset=utf-8",
                    Some("js") => "text/javascript; charset=utf-8",
                    Some("json") => "application/json; charset=utf-8",
                    _ => "application/octet-stream",
                };
                let mut headers = HashMap::new();
                headers.insert("Content-Type".into(), ct.into());
                response = Some(Response { status: 200, headers, body: content });
            }
        }
    }

    let resp = response.unwrap_or_else(Response::not_found);
    send_response(stream, &resp);
}

fn send_response(stream: &mut TcpStream, response: &Response) {
    let status_text = match response.status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let mut header = format!("HTTP/1.1 {} {status_text}\r\n", response.status);
    for (k, v) in &response.headers {
        header.push_str(&format!("{k}: {v}\r\n"));
    }
    header.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    header.push_str("Connection: close\r\n\r\n");
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&response.body);
}

fn parse_url(url: &str) -> (String, HashMap<String, String>) {
    let mut parts = url.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let mut query = HashMap::new();
    if let Some(qs) = parts.next() {
        for param in qs.split('&') {
            if let Some((k, v)) = param.split_once('=') {
                query.insert(k.to_string(), v.to_string());
            }
        }
    }
    (path, query)
}

fn parse_json_args(body: &str) -> Vec<Value> {
    if body.is_empty() { return vec![]; }
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Array(arr)) => {
            arr.iter().map(|v| json_to_value(v)).collect()
        }
        Ok(serde_json::Value::Object(obj)) => {
            obj.values().map(|v| json_to_value(v)).collect()
        }
        _ => vec![Value::String(body.to_string())],
    }
}

fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => Value::Int(n.as_i64().unwrap_or(0)),
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            Value::Map(obj.iter().map(|(k, v)| (Value::String(k.clone()), json_to_value(v))).collect())
        }
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::json!(b),
        Value::Int(n) => serde_json::json!(n),
        Value::String(s) => serde_json::json!(s),
        Value::List(items) => serde_json::Value::Array(items.iter().map(value_to_json).collect()),
        Value::Map(entries) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in entries {
                if let Value::String(key) = k {
                    obj.insert(key.clone(), value_to_json(v));
                }
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::json!(format!("{value:?}")),
    }
}

/// Convenience macro for creating AXL web apps
#[macro_export]
macro_rules! axl_app {
    ($port:expr, $static_dir:expr, $($method:ident $path:expr => $handler:expr),* $(,)?) => {{
        let mut server = $crate::web::AxlServer::new($port).static_files($static_dir);
        $(
            server = server.$method($path, $handler);
        )*
        server.serve()
    }};
}
