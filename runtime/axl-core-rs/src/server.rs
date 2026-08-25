use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::primitives::db::SharedConnection;
use rusqlite::Connection;

type HandlerFn = Box<dyn Fn(&str, &str, &str, &HashMap<String,String>) -> (u16, String, String) + Send + Sync>;

pub struct AxlServer {
    pub addr: String,
    pub static_dir: String,
    routes: Vec<(String, String, Arc<HandlerFn>)>,
    db_path: String,
    db_conn: Option<SharedConnection>,
}

impl AxlServer {
    pub fn new(addr: &str, static_dir: &str, db_path: &str) -> Self {
        Self {
            addr: addr.to_string(),
            static_dir: static_dir.to_string(),
            routes: Vec::new(),
            db_path: db_path.to_string(),
            db_conn: None,
        }
    }

    /// Create a server that uses an existing shared database connection.
    /// The AXL program owns the schema and seed data; the server only queries.
    pub fn with_connection(addr: &str, static_dir: &str, conn: SharedConnection) -> Self {
        Self {
            addr: addr.to_string(),
            static_dir: static_dir.to_string(),
            routes: Vec::new(),
            db_path: String::new(),
            db_conn: Some(conn),
        }
    }

    pub fn init_database(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| format!("db open: {e}"))?;
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS customers (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT NOT NULL, company TEXT, phone TEXT, status TEXT DEFAULT 'active');
            CREATE TABLE IF NOT EXISTS leads (id INTEGER PRIMARY KEY AUTOINCREMENT, company TEXT NOT NULL, contact TEXT NOT NULL, email TEXT, source TEXT, status TEXT DEFAULT 'warm', value INTEGER DEFAULT 0, score INTEGER DEFAULT 50);
            CREATE TABLE IF NOT EXISTS deals (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, customer TEXT, value INTEGER DEFAULT 0, stage TEXT DEFAULT 'discovery', probability INTEGER DEFAULT 50);
            CREATE TABLE IF NOT EXISTS activities (id INTEGER PRIMARY KEY AUTOINCREMENT, date TEXT, type TEXT, related TEXT, description TEXT);
        ").map_err(|e| format!("db create tables: {e}"))?;

        // Seed data only if tables are empty
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
        if count == 0 {
            conn.execute_batch("
                INSERT INTO customers (name, email, company, phone, status) VALUES
                    ('Alice Johnson', 'alice@acme.com', 'Acme Corp', '+1-555-0101', 'active'),
                    ('Bob Smith', 'bob@startup.io', 'Startup Inc', '+1-555-0102', 'active'),
                    ('Charlie Brown', 'charlie@enterprise.com', 'Enterprise Ltd', '+1-555-0103', 'inactive'),
                    ('Diana Prince', 'diana@tech.co', 'Tech Co', '+1-555-0104', 'active'),
                    ('Eve Wilson', 'eve@design.io', 'Design Studio', '+1-555-0105', 'lead');
                INSERT INTO leads (company, contact, email, source, status, value, score) VALUES
                    ('Acme Corp', 'Alice Johnson', 'alice@acme.com', 'Website', 'hot', 75000, 85),
                    ('Startup Inc', 'Bob Smith', 'bob@startup.io', 'Referral', 'warm', 25000, 65),
                    ('Tech Co', 'Diana Prince', 'diana@tech.co', 'LinkedIn', 'hot', 120000, 90),
                    ('Design Studio', 'Eve Wilson', 'eve@design.io', 'Cold Call', 'cold', 15000, 30);
                INSERT INTO deals (name, customer, value, stage, probability) VALUES
                    ('Acme Renewal', 'Acme Corp', 75000, 'proposal', 80),
                    ('Startup MVP', 'Startup Inc', 25000, 'negotiation', 60),
                    ('Tech Integration', 'Tech Co', 120000, 'discovery', 40);
                INSERT INTO activities (date, type, related, description) VALUES
                    ('2026-01-15', 'Call', 'Acme Corp', 'Discussed Q1 contract renewal'),
                    ('2026-01-14', 'Email', 'Startup Inc', 'Sent product demo video'),
                    ('2026-01-13', 'Meeting', 'Tech Co', 'Technical requirements review'),
                    ('2026-01-12', 'Note', 'Enterprise Ltd', 'Follow up needed next week'),
                    ('2026-01-11', 'Task', 'Design Studio', 'Prepare proposal document');
            ").map_err(|e| format!("db seed: {e}"))?;
            println!("Database seeded with sample data");
        }
        Ok(())
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: HandlerFn) {
        self.routes.push((
            method.to_uppercase(),
            path.to_string(),
            Arc::new(handler),
        ));
    }

    pub fn add_api_routes(&mut self) {
        // Generic CRUD routes are now added via add_table_routes()
        // This method is kept for backward compatibility
    }

    /// Get or create a SharedConnection for route handlers.
    fn get_shared_conn(&self) -> SharedConnection {
        if let Some(ref conn) = self.db_conn {
            conn.clone()
        } else {
            let conn = Connection::open(&self.db_path).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            Arc::new(Mutex::new(conn))
        }
    }

    /// Add generic CRUD routes for any database table.
    /// Creates: GET /api/{table}, GET /api/{table}/:id, POST /api/{table}, PUT /api/{table}/:id, DELETE /api/{table}/:id
    pub fn add_table_routes(&mut self, table: &str) {
        let sc = self.get_shared_conn();
        let tbl = table.to_string();
        let api_prefix = format!("/api/{}", tbl);
        let prefix = format!("{}/", api_prefix);

        // GET /api/{table} — list all rows (Refine format: {data:[...],total:N})
        let c = sc.clone(); let t = tbl.clone();
        self.add_route("GET", &api_prefix, Box::new(move |_, _, _, _| {
            let conn = c.lock().unwrap();
            let sql = format!("SELECT * FROM {}", t);
            let mut stmt = match conn.prepare(&sql) { Ok(s) => s, Err(e) => return (500, "application/json".into(), format!("{{\"error\":\"{}\"}}", e)) };
            let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
            let mut rows = Vec::new();
            let mut row_iter = match stmt.query([]) { Ok(r) => r, Err(e) => return (500, "application/json".into(), format!("{{\"error\":\"{}\"}}", e)) };
            while let Ok(Some(row)) = row_iter.next() {
                let mut map = Vec::new();
                for (i, col) in columns.iter().enumerate() {
                    let val = if let Ok(n) = row.get::<_, i64>(i) { format!("{}", n) }
                        else if let Ok(s) = row.get::<_, String>(i) { format!("\"{}\"", s.replace('"', "\\\"")) }
                        else { "null".into() };
                    map.push(format!("\"{}\":{}", col, val));
                }
                rows.push(format!("{{{}}}", map.join(",")));
            }
            let total = rows.len();
            (200, "application/json".into(), format!("{{\"data\":[{}],\"total\":{}}}", rows.join(","), total))
        }));

        // POST /api/{table} — create new row (Refine format: {data:{...}})
        let c = sc.clone(); let t = tbl.clone();
        self.add_route("POST", &api_prefix, Box::new(move |_, _, body, _| {
            let conn = c.lock().unwrap();
            let (columns, values, placeholders) = parse_json_body(body);
            if columns.is_empty() { return (400, "application/json".into(), "{\"error\":\"empty body\"}".into()); }
            let sql = format!("INSERT INTO {} ({}) VALUES ({})", t, columns.join(", "), placeholders.join(", "));
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
            match conn.execute(&sql, params_refs.as_slice()) {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    if let Some(row_json) = query_row_json(&conn, &t, id) {
                        (201, "application/json".into(), format!("{{\"data\":{}}}", row_json))
                    } else {
                        (201, "application/json".into(), format!("{{\"data\":{{\"id\":{}}}}}", id))
                    }
                }
                Err(e) => (500, "application/json".into(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // GET /api/{table}/:id — get by id (Refine format: {data:{...}})
        let c = sc.clone(); let t = tbl.clone();
        self.add_route("GET", &prefix, Box::new(move |_, path, _, _| {
            let id = extract_id(path);
            if id == 0 { return (400, "application/json".into(), "{\"error\":\"invalid id\"}".into()); }
            let conn = c.lock().unwrap();
            match query_row_json(&conn, &t, id) {
                Some(json) => (200, "application/json".into(), format!("{{\"data\":{}}}", json)),
                None => (404, "application/json".into(), "{\"error\":\"not found\"}".into()),
            }
        }));

        // PUT /api/{table}/:id — update by id (Refine format: {data:{...}})
        let c = sc.clone(); let t = tbl.clone();
        self.add_route("PUT", &prefix, Box::new(move |_, path, body, _| {
            let id = extract_id(path);
            if id == 0 { return (400, "application/json".into(), "{\"error\":\"invalid id\"}".into()); }
            let conn = c.lock().unwrap();
            let (set_parts, mut values) = parse_json_set(body);
            if set_parts.is_empty() { return (400, "application/json".into(), "{\"error\":\"empty body\"}".into()); }
            values.push(id.to_string());
            let idx = values.len();
            let sql = format!("UPDATE {} SET {} WHERE id = ?{}", t, set_parts.join(", "), idx);
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
            match conn.execute(&sql, params_refs.as_slice()) {
                Ok(_) => {
                    if let Some(row_json) = query_row_json(&conn, &t, id) {
                        (200, "application/json".into(), format!("{{\"data\":{}}}", row_json))
                    } else {
                        (200, "application/json".into(), "{\"data\":{}}".into())
                    }
                }
                Err(e) => (500, "application/json".into(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // DELETE /api/{table}/:id — delete by id (Refine format: {data:{id:N}})
        let c = sc.clone(); let t = tbl.clone();
        self.add_route("DELETE", &prefix, Box::new(move |_, path, _, _| {
            let id = extract_id(path);
            if id == 0 { return (400, "application/json".into(), "{\"error\":\"invalid id\"}".into()); }
            let conn = c.lock().unwrap();
            let sql = format!("DELETE FROM {} WHERE id = ?1", t);
            match conn.execute(&sql, rusqlite::params![id]) {
                Ok(_) => (200, "application/json".into(), format!("{{\"data\":{{\"id\":{}}}}}", id)),
                Err(e) => (500, "application/json".into(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));
    }

    pub fn run(&self) -> Result<(), String> {
        if self.db_conn.is_none() {
            self.init_database()?;
        }
        let listener = TcpListener::bind(&self.addr).map_err(|e| format!("bind: {e}"))?;
        println!("AXL Server listening on {}", self.addr);
        println!("API routes: {}", self.routes.len());
        if !self.static_dir.is_empty() {
            println!("Static files: {}", self.static_dir);
        }

        let routes: Vec<(String, String, Arc<HandlerFn>)> = self.routes.iter()
            .map(|(m, p, h)| (m.clone(), p.clone(), h.clone()))
            .collect();
        let static_dir = self.static_dir.clone();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = routes.clone();
                    let static_dir = static_dir.clone();
                    thread::spawn(move || {
                        handle_connection(stream, &routes, &static_dir);
                    });
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    /// Spawn the server in a background thread and return immediately.
    pub fn run_non_blocking(self) -> Result<(), String> {
        let addr = self.addr.clone();
        thread::spawn(move || {
            if let Err(e) = self.run() {
                eprintln!("AXL Server error: {e}");
            }
        });
        // Small delay to let the thread start
        std::thread::sleep(std::time::Duration::from_millis(50));
        println!("AXL Server started on {addr} (background)");
        Ok(())
    }
}

fn extract_id(path: &str) -> i64 {
    path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0)
}

/// Query a single row by id and return it as a JSON object string.
fn query_row_json(conn: &Connection, table: &str, id: i64) -> Option<String> {
    let sql = format!("SELECT * FROM {} WHERE id = ?1", table);
    let mut stmt = conn.prepare(&sql).ok()?;
    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    stmt.query_row(rusqlite::params![id], |row| {
        let mut map = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            let val = if let Ok(n) = row.get::<_, i64>(i) { format!("{}", n) }
                else if let Ok(s) = row.get::<_, String>(i) { format!("\"{}\"", s.replace('"', "\\\"")) }
                else { "null".into() };
            map.push(format!("\"{}\":{}", col, val));
        }
        Ok(format!("{{{}}}", map.join(",")))
    }).ok()
}

fn parse_json_body(body: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut columns = Vec::new();
    let mut values = Vec::new();
    let mut placeholders = Vec::new();
    let mut idx = 1;
    let body_trimmed = body.trim();
    if body_trimmed.starts_with('{') && body_trimmed.ends_with('}') {
        let inner = &body_trimmed[1..body_trimmed.len()-1];
        for pair in inner.split(',') {
            if let Some((key, val)) = pair.split_once(':') {
                let key = key.trim().trim_matches('"').to_string();
                let val = val.trim();
                let val_str = if val.starts_with('"') && val.ends_with('"') {
                    val[1..val.len()-1].to_string()
                } else { val.to_string() };
                columns.push(key);
                values.push(val_str);
                placeholders.push(format!("?{}", idx));
                idx += 1;
            }
        }
    }
    (columns, values, placeholders)
}

fn parse_json_set(body: &str) -> (Vec<String>, Vec<String>) {
    let mut set_parts = Vec::new();
    let mut values = Vec::new();
    let mut idx = 1;
    let body_trimmed = body.trim();
    if body_trimmed.starts_with('{') && body_trimmed.ends_with('}') {
        let inner = &body_trimmed[1..body_trimmed.len()-1];
        for pair in inner.split(',') {
            if let Some((key, val)) = pair.split_once(':') {
                let key = key.trim().trim_matches('"').to_string();
                let val = val.trim();
                let val_str = if val.starts_with('"') && val.ends_with('"') {
                    val[1..val.len()-1].to_string()
                } else { val.to_string() };
                set_parts.push(format!("{} = ?{}", key, idx));
                values.push(val_str);
                idx += 1;
            }
        }
    }
    (set_parts, values)
}

fn handle_connection(mut stream: TcpStream, routes: &[(String, String, Arc<HandlerFn>)], static_dir: &str) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap_or(0);

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET").to_string();
    let full_path = parts.get(1).unwrap_or(&"/").to_string();

    let (path, _query) = if let Some(qpos) = full_path.find('?') {
        (full_path[..qpos].to_string(), full_path[qpos+1..].to_string())
    } else {
        (full_path.clone(), String::new())
    };

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

    // CORS preflight
    if method == "OPTIONS" {
        send_response(&mut stream, 204, "text/plain", "");
        return;
    }

    // Check routes — exact match first, then prefix match
    // This ensures /api/customers matches exactly, while /api/customers/1 matches the prefix route
    for (route_method, route_path, handler) in routes {
        if route_method == &method || route_method == "*" {
            let matched = if route_path == &path {
                // Exact match
                true
            } else if route_path.ends_with('/') && path.starts_with(route_path.as_str()) && path.len() > route_path.len() {
                // Prefix match only if path is longer than route (e.g., /api/customers/1 matches /api/customers/)
                true
            } else {
                false
            };
            if matched {
                let (status, ct, response_body) = handler(&method, &path, &body, &headers);
                send_response(&mut stream, status, &ct, &response_body);
                return;
            }
        }
    }

    // Static files
    if !static_dir.is_empty() && (method == "GET" || method == "HEAD") {
        let file_path = if path == "/" {
            format!("{}/index.html", static_dir.trim_end_matches('/'))
        } else {
            format!("{}/{}", static_dir.trim_end_matches('/'), path.trim_start_matches('/'))
        };
        if let Ok(content) = std::fs::read(&file_path) {
            let ct = match file_path.rsplit('.').next() {
                Some("html") => "text/html; charset=utf-8",
                Some("css") => "text/css; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("json") => "application/json",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            let content_str = String::from_utf8_lossy(&content).to_string();
            send_response(&mut stream, 200, ct, &content_str);
            return;
        }
    }

    // SPA fallback — serve index.html for non-API, non-file routes
    if !static_dir.is_empty() && !path.starts_with("/api") {
        let index = format!("{}/index.html", static_dir.trim_end_matches('/'));
        if let Ok(content) = std::fs::read(&index) {
            let content_str = String::from_utf8_lossy(&content).to_string();
            send_response(&mut stream, 200, "text/html; charset=utf-8", &content_str);
            return;
        }
    }

    let body = format!("{{\"method\":\"{}\",\"path\":\"{}\"}}", method, path);
    send_response(&mut stream, 200, "application/json", &body);
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}


