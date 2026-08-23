use std::collections::HashMap;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::ir::Value;
use rusqlite::Connection;

type HandlerFn = Box<dyn Fn(&str, &str, &str, &HashMap<String,String>) -> (u16, String, String) + Send + Sync>;

pub struct AxlServer {
    pub addr: String,
    pub static_dir: String,
    routes: Vec<(String, String, Arc<HandlerFn>)>,
    db_path: String,
}

impl AxlServer {
    pub fn new(addr: &str, static_dir: &str, db_path: &str) -> Self {
        Self {
            addr: addr.to_string(),
            static_dir: static_dir.to_string(),
            routes: Vec::new(),
            db_path: db_path.to_string(),
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
        let db = self.db_path.clone();

        // GET /api/customers
        let db_clone = db.clone();
        self.add_route("GET", "/api/customers", Box::new(move |_, _, _, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let mut stmt = conn.prepare("SELECT id, name, email, company, phone, status FROM customers").unwrap();
            let mut customers = Vec::new();
            let mut rows = stmt.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let email: String = row.get(2).unwrap();
                let company: String = row.get(3).unwrap();
                let phone: String = row.get(4).unwrap();
                let status: String = row.get(5).unwrap();
                customers.push(format!("{{\"id\":{},\"name\":\"{}\",\"email\":\"{}\",\"company\":\"{}\",\"phone\":\"{}\",\"status\":\"{}\"}}", id, name, email, company, phone, status));
            }
            let json = format!("[{}]", customers.join(","));
            (200, "application/json".to_string(), json)
        }));

        // GET /api/leads
        let db_clone = db.clone();
        self.add_route("GET", "/api/leads", Box::new(move |_, _, _, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let mut stmt = conn.prepare("SELECT id, company, contact, email, source, status, value, score FROM leads").unwrap();
            let mut leads = Vec::new();
            let mut rows = stmt.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let company: String = row.get(1).unwrap();
                let contact: String = row.get(2).unwrap();
                let email: String = row.get(3).unwrap();
                let source: String = row.get(4).unwrap();
                let status: String = row.get(5).unwrap();
                let value: i64 = row.get(6).unwrap();
                let score: i64 = row.get(7).unwrap();
                leads.push(format!("{{\"id\":{},\"company\":\"{}\",\"contact\":\"{}\",\"email\":\"{}\",\"source\":\"{}\",\"status\":\"{}\",\"value\":{},\"score\":{}}}", id, company, contact, email, source, status, value, score));
            }
            let json = format!("[{}]", leads.join(","));
            (200, "application/json".to_string(), json)
        }));

        // GET /api/deals
        let db_clone = db.clone();
        self.add_route("GET", "/api/deals", Box::new(move |_, _, _, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let mut stmt = conn.prepare("SELECT id, name, customer, value, stage, probability FROM deals").unwrap();
            let mut deals = Vec::new();
            let mut rows = stmt.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let name: String = row.get(1).unwrap();
                let customer: String = row.get(2).unwrap();
                let value: i64 = row.get(3).unwrap();
                let stage: String = row.get(4).unwrap();
                let probability: i64 = row.get(5).unwrap();
                deals.push(format!("{{\"id\":{},\"name\":\"{}\",\"customer\":\"{}\",\"value\":{},\"stage\":\"{}\",\"probability\":{}}}", id, name, customer, value, stage, probability));
            }
            let json = format!("[{}]", deals.join(","));
            (200, "application/json".to_string(), json)
        }));

        // GET /api/activities
        let db_clone = db.clone();
        self.add_route("GET", "/api/activities", Box::new(move |_, _, _, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let mut stmt = conn.prepare("SELECT id, date, type, related, description FROM activities ORDER BY date DESC").unwrap();
            let mut activities = Vec::new();
            let mut rows = stmt.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                let id: i64 = row.get(0).unwrap();
                let date: String = row.get(1).unwrap();
                let act_type: String = row.get(2).unwrap();
                let related: String = row.get(3).unwrap();
                let description: String = row.get(4).unwrap();
                activities.push(format!("{{\"id\":{},\"date\":\"{}\",\"type\":\"{}\",\"related\":\"{}\",\"desc\":\"{}\"}}", id, date, act_type, related, description));
            }
            let json = format!("[{}]", activities.join(","));
            (200, "application/json".to_string(), json)
        }));

        // GET /api/stats
        let db_clone = db.clone();
        self.add_route("GET", "/api/stats", Box::new(move |_, _, _, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let customers: i64 = conn.query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0)).unwrap_or(0);
            let leads: i64 = conn.query_row("SELECT COUNT(*) FROM leads", [], |r| r.get(0)).unwrap_or(0);
            let deals: i64 = conn.query_row("SELECT COUNT(*) FROM deals", [], |r| r.get(0)).unwrap_or(0);
            let revenue: i64 = conn.query_row("SELECT COALESCE(SUM(value),0) FROM deals", [], |r| r.get(0)).unwrap_or(0);
            let won: i64 = conn.query_row("SELECT COUNT(*) FROM deals WHERE stage='closed-won'", [], |r| r.get(0)).unwrap_or(0);
            let win_rate = if deals > 0 { (won * 100 / deals) as i64 } else { 0 };
            let json = format!("{{\"customers\":{},\"leads\":{},\"deals\":{},\"revenue\":{},\"winRate\":{}}}", customers, leads, deals, revenue, win_rate);
            (200, "application/json".to_string(), json)
        }));

        // POST /api/customers
        let db_clone = db.clone();
        self.add_route("POST", "/api/customers", Box::new(move |_, _, body, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            // Simple JSON parsing for name, email, company, phone
            let name = extract_json_string(body, "name");
            let email = extract_json_string(body, "email");
            let company = extract_json_string(body, "company");
            let phone = extract_json_string(body, "phone");
            if name.is_empty() || email.is_empty() {
                return (400, "application/json".to_string(), "{\"error\":\"name and email required\"}".to_string());
            }
            let result = conn.execute(
                "INSERT INTO customers (name, email, company, phone, status) VALUES (?1, ?2, ?3, ?4, 'active')",
                rusqlite::params![name, email, company, phone],
            );
            match result {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    let json = format!("{{\"id\":{},\"name\":\"{}\",\"email\":\"{}\",\"company\":\"{}\",\"phone\":\"{}\",\"status\":\"active\"}}", id, name, email, company, phone);
                    (201, "application/json".to_string(), json)
                }
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // PUT /api/customers/:id
        let db_clone = db.clone();
        self.add_route("PUT", "/api/customers", Box::new(move |_, path, body, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let name = extract_json_string(body, "name");
            let email = extract_json_string(body, "email");
            let company = extract_json_string(body, "company");
            let phone = extract_json_string(body, "phone");
            let status = extract_json_string(body, "status");
            let result = conn.execute(
                "UPDATE customers SET name=?1, email=?2, company=?3, phone=?4, status=?5 WHERE id=?6",
                rusqlite::params![name, email, company, phone, if status.is_empty() { "active" } else { &status }, id],
            );
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // DELETE /api/customers/:id
        let db_clone = db.clone();
        self.add_route("DELETE", "/api/customers", Box::new(move |_, path, _, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let result = conn.execute("DELETE FROM customers WHERE id=?1", rusqlite::params![id]);
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // POST /api/leads
        let db_clone = db.clone();
        self.add_route("POST", "/api/leads", Box::new(move |_, _, body, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let company = extract_json_string(body, "company");
            let contact = extract_json_string(body, "contact");
            let email = extract_json_string(body, "email");
            let source = extract_json_string(body, "source");
            let value = extract_json_number(body, "value");
            if company.is_empty() || contact.is_empty() {
                return (400, "application/json".to_string(), "{\"error\":\"company and contact required\"}".to_string());
            }
            let result = conn.execute(
                "INSERT INTO leads (company, contact, email, source, status, value, score) VALUES (?1, ?2, ?3, ?4, 'warm', ?5, 50)",
                rusqlite::params![company, contact, email, source, value],
            );
            match result {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    let json = format!("{{\"id\":{},\"company\":\"{}\",\"contact\":\"{}\",\"email\":\"{}\",\"source\":\"{}\",\"status\":\"warm\",\"value\":{},\"score\":50}}", id, company, contact, email, source, value);
                    (201, "application/json".to_string(), json)
                }
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // PUT /api/leads/:id
        let db_clone = db.clone();
        self.add_route("PUT", "/api/leads", Box::new(move |_, path, body, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let company = extract_json_string(body, "company");
            let contact = extract_json_string(body, "contact");
            let email = extract_json_string(body, "email");
            let source = extract_json_string(body, "source");
            let value = extract_json_number(body, "value");
            let status = extract_json_string(body, "status");
            let score = extract_json_number(body, "score");
            let result = conn.execute(
                "UPDATE leads SET company=?1, contact=?2, email=?3, source=?4, value=?5, status=?6, score=?7 WHERE id=?8",
                rusqlite::params![company, contact, email, source, value, if status.is_empty() { "warm" } else { &status }, score, id],
            );
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // DELETE /api/leads/:id
        let db_clone = db.clone();
        self.add_route("DELETE", "/api/leads", Box::new(move |_, path, _, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let result = conn.execute("DELETE FROM leads WHERE id=?1", rusqlite::params![id]);
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // POST /api/deals
        let db_clone = db.clone();
        self.add_route("POST", "/api/deals", Box::new(move |_, _, body, _| {
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let name = extract_json_string(body, "name");
            let customer = extract_json_string(body, "customer");
            let value = extract_json_number(body, "value");
            let stage = extract_json_string(body, "stage");
            if name.is_empty() {
                return (400, "application/json".to_string(), "{\"error\":\"name required\"}".to_string());
            }
            let stage_str = if stage.is_empty() { "discovery" } else { &stage };
            let result = conn.execute(
                "INSERT INTO deals (name, customer, value, stage, probability) VALUES (?1, ?2, ?3, ?4, 50)",
                rusqlite::params![name, customer, value, stage_str],
            );
            match result {
                Ok(_) => {
                    let id = conn.last_insert_rowid();
                    let json = format!("{{\"id\":{},\"name\":\"{}\",\"customer\":\"{}\",\"value\":{},\"stage\":\"{}\",\"probability\":50}}", id, name, customer, value, stage_str);
                    (201, "application/json".to_string(), json)
                }
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // PUT /api/deals/:id
        let db_clone = db.clone();
        self.add_route("PUT", "/api/deals", Box::new(move |_, path, body, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let name = extract_json_string(body, "name");
            let customer = extract_json_string(body, "customer");
            let value = extract_json_number(body, "value");
            let stage = extract_json_string(body, "stage");
            let probability = extract_json_number(body, "probability");
            let result = conn.execute(
                "UPDATE deals SET name=?1, customer=?2, value=?3, stage=?4, probability=?5 WHERE id=?6",
                rusqlite::params![name, customer, value, if stage.is_empty() { "discovery" } else { &stage }, probability, id],
            );
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));

        // DELETE /api/deals/:id
        let db_clone = db.clone();
        self.add_route("DELETE", "/api/deals", Box::new(move |_, path, _, _| {
            let id = path.split('/').last().unwrap_or("0").parse::<i64>().unwrap_or(0);
            if id == 0 {
                return (400, "application/json".to_string(), "{\"error\":\"invalid id\"}".to_string());
            }
            let conn = Connection::open(&db_clone).unwrap_or_else(|_| Connection::open_in_memory().unwrap());
            let result = conn.execute("DELETE FROM deals WHERE id=?1", rusqlite::params![id]);
            match result {
                Ok(_) => (200, "application/json".to_string(), "{\"success\":true}".to_string()),
                Err(e) => (500, "application/json".to_string(), format!("{{\"error\":\"{}\"}}", e)),
            }
        }));
    }

    pub fn run(&self) -> Result<(), String> {
        self.init_database()?;
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

    // Check routes
    for (route_method, route_path, handler) in routes {
        if route_method == &method || route_method == "*" {
            let matched = if route_path == &path {
                true
            } else if route_path.ends_with('/') {
                path.starts_with(route_path.as_str())
            } else {
                // Support /api/customers/:id pattern
                let prefix = format!("{}/", route_path);
                path.starts_with(&prefix)
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

fn extract_json_string(json: &str, key: &str) -> String {
    let pattern = format!("\"{}\"", key);
    if let Some(start) = json.find(&pattern) {
        let after_key = &json[start + pattern.len()..];
        if let Some(colon_pos) = after_key.find(':') {
            let after_colon = after_key[colon_pos + 1..].trim_start();
            if after_colon.starts_with('"') {
                let value_start = 1;
                if let Some(end) = after_colon[value_start..].find('"') {
                    return after_colon[value_start..value_start + end].to_string();
                }
            }
        }
    }
    String::new()
}

fn extract_json_number(json: &str, key: &str) -> i64 {
    let pattern = format!("\"{}\"", key);
    if let Some(start) = json.find(&pattern) {
        let after_key = &json[start + pattern.len()..];
        if let Some(colon_pos) = after_key.find(':') {
            let after_colon = after_key[colon_pos + 1..].trim_start();
            let num_str: String = after_colon.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().unwrap_or(0);
        }
    }
    0
}
