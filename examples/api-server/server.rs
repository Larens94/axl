use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axl_core::{Value, primitives};

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(8080);

    // Build frontend
    let source = std::fs::read_to_string("examples/api-server/api-server.axl").unwrap();
    let program = axl_core::parse_compact(&source).unwrap();
    axl_core::validate(&program).unwrap();
    let output_dir = std::path::PathBuf::from("build/api-server");
    axl_core::build_web(&program, &output_dir).unwrap();

    // Shared state
    let db: Arc<Mutex<HashMap<i64, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    let cache: Arc<Mutex<HashMap<String, (Value, f64)>>> = Arc::new(Mutex::new(HashMap::new()));
    let rate_limits: Arc<Mutex<HashMap<String, Vec<f64>>>> = Arc::new(Mutex::new(HashMap::new()));
    let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Seed data
    {
        let mut db = db.lock().unwrap();
        db.insert(1, Value::Map(vec![
            (Value::String("id".into()), Value::Int(1)),
            (Value::String("name".into()), Value::String("Alice".into())),
            (Value::String("email".into()), Value::String("alice@example.com".into())),
            (Value::String("role".into()), Value::String("admin".into())),
        ]));
        db.insert(2, Value::Map(vec![
            (Value::String("id".into()), Value::Int(2)),
            (Value::String("name".into()), Value::String("Bob".into())),
            (Value::String("email".into()), Value::String("bob@example.com".into())),
            (Value::String("role".into()), Value::String("user".into())),
        ]));
    }

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    println!("=== AXL API Server ===");
    println!("Frontend: http://localhost:{port}");
    println!("API: GET/POST/PUT/DELETE /api/users");
    println!("Health: GET /api/health");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let db = db.clone();
            let cache = cache.clone();
            let rate_limits = rate_limits.clone();
            let logs = logs.clone();
            handle_request(&mut stream, &output_dir, &db, &cache, &rate_limits, &logs);
        }
    }
}

fn handle_request(
    stream: &mut TcpStream,
    root: &Path,
    db: &Arc<Mutex<HashMap<i64, Value>>>,
    cache: &Arc<Mutex<HashMap<String, (Value, f64)>>>,
    rate_limits: &Arc<Mutex<HashMap<String, Vec<f64>>>>,
    logs: &Arc<Mutex<Vec<String>>>,
) {
    use std::path::Path;

    let mut request = [0u8; 8192];
    let size = stream.read(&mut request).unwrap_or(0);
    let first_line = String::from_utf8_lossy(&request[..size]);
    let lines: Vec<&str> = first_line.lines().collect();
    let parts: Vec<&str> = lines.first().unwrap_or(&"").split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET");
    let target = parts.get(1).copied().unwrap_or("/");

    let (path, query) = parse_url(target);
    let body_start = first_line.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = String::from_utf8_lossy(&request[body_start..size]).to_string();

    // Extract client IP for rate limiting
    let client_ip = "127.0.0.1".to_string();

    // Rate limiting (100 requests per minute)
    {
        let mut rl = rate_limits.lock().unwrap();
        let now = now_f64();
        let entries = rl.entry(client_ip.clone()).or_default();
        entries.retain(|t| now - t < 60.0);
        if entries.len() >= 100 {
            send_json(stream, 429, &json!({"error": "rate limit exceeded"}));
            return;
        }
        entries.push(now);
    }

    // Log request
    {
        let mut logs = logs.lock().unwrap();
        logs.push(format!("[{}] {} {} {} bytes", now_f64(), method, path, body.len()));
    }

    // Route matching
    let response = match (method, path.as_str()) {
        // Health check
        ("GET", "/api/health") => {
            json_response(200, &json!({
                "status": "ok",
                "timestamp": now_f64() as i64,
                "version": "1.0.0"
            }))
        }

        // List users (with caching)
        ("GET", "/api/users") => {
            let cache_key = "users:list".to_string();
            let cached = {
                let cache = cache.lock().unwrap();
                cache.get(&cache_key).and_then(|(v, t)| {
                    if now_f64() - t < 5.0 { Some(v.clone()) } else { None }
                })
            };
            if let Some(cached) = cached {
                return send_json(stream, 200, &value_to_json(&cached));
            }
            let users: Vec<Value> = db.lock().unwrap().values().cloned().collect();
            let result = Value::List(users.clone());
            {
                let mut cache = cache.lock().unwrap();
                cache.insert(cache_key, (result.clone(), now_f64()));
            }
            json_response(200, &json!({
                "count": users.len(),
                "users": users.iter().map(|u| value_to_json(u)).collect::<Vec<_>>()
            }))
        }

        // Get user by ID
        ("GET", _) if path.starts_with("/api/users/") => {
            let id: i64 = path.trim_start_matches("/api/users/").parse().unwrap_or(0);
            let db = db.lock().unwrap();
            match db.get(&id) {
                Some(user) => json_response(200, &json!({"user": value_to_json(user)})),
                None => json_response(404, &json!({"error": "user not found"})),
            }
        }

        // Create user
        ("POST", "/api/users") => {
            let mut db = db.lock().unwrap();
            let next_id = db.keys().max().unwrap_or(&0) + 1;
            let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let email = parsed.get("email").and_then(|v| v.as_str()).unwrap_or("unknown");
            let role = parsed.get("role").and_then(|v| v.as_str()).unwrap_or("user");

            let user = Value::Map(vec![
                (Value::String("id".into()), Value::Int(next_id)),
                (Value::String("name".into()), Value::String(name.into())),
                (Value::String("email".into()), Value::String(email.into())),
                (Value::String("role".into()), Value::String(role.into())),
            ]);
            db.insert(next_id, user.clone());

            // Invalidate cache
            cache.lock().unwrap().remove("users:list");

            json_response(201, &json!({"user": value_to_json(&user)}))
        }

        // Update user
        ("PUT", _) if path.starts_with("/api/users/") => {
            let id: i64 = path.trim_start_matches("/api/users/").parse().unwrap_or(0);
            let mut db = db.lock().unwrap();
            if let Some(user) = db.get_mut(&id) {
                let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
                if let Value::Map(entries) = user {
                    if let Some(name) = parsed.get("name").and_then(|v| v.as_str()) {
                        entries.retain(|(k, _)| k != &Value::String("name".into()));
                        entries.push((Value::String("name".into()), Value::String(name.into())));
                    }
                    if let Some(email) = parsed.get("email").and_then(|v| v.as_str()) {
                        entries.retain(|(k, _)| k != &Value::String("email".into()));
                        entries.push((Value::String("email".into()), Value::String(email.into())));
                    }
                }
                cache.lock().unwrap().remove("users:list");
                json_response(200, &json!({"user": value_to_json(user)}))
            } else {
                json_response(404, &json!({"error": "user not found"}))
            }
        }

        // Delete user
        ("DELETE", _) if path.starts_with("/api/users/") => {
            let id: i64 = path.trim_start_matches("/api/users/").parse().unwrap_or(0);
            let mut db = db.lock().unwrap();
            if db.remove(&id).is_some() {
                cache.lock().unwrap().remove("users:list");
                json_response(200, &json!({"deleted": id}))
            } else {
                json_response(404, &json!({"error": "user not found"}))
            }
        }

        // Static files
        ("GET", _) => {
            let file_path = root.join(path.trim_start_matches('/'));
            match std::fs::read(&file_path) {
                Ok(content) => {
                    let ct = match file_path.extension().and_then(|e| e.to_str()) {
                        Some("html") => "text/html; charset=utf-8",
                        Some("css") => "text/css; charset=utf-8",
                        Some("js") => "text/javascript; charset=utf-8",
                        _ => "application/octet-stream",
                    };
                    send_response(stream, 200, ct, &content);
                    return;
                }
                Err(_) => {
                    send_response(stream, 200, "text/html; charset=utf-8", b"<!DOCTYPE html><html><body><h1>Not Found</h1></body></html>");
                    return;
                }
            }
        }

        _ => json_response(404, &json!({"error": "not found"})),
    };

    send_response(stream, response.0, &response.1, response.2.as_bytes());
}

fn json_response(status: u16, value: &serde_json::Value) -> (u16, &'static str, String) {
    (status, "application/json; charset=utf-8", serde_json::to_string(value).unwrap())
}

fn send_json(stream: &mut TcpStream, status: u16, value: &serde_json::Value) {
    let body = serde_json::to_string(value).unwrap();
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body.as_bytes());
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
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

fn now_f64() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64()
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
