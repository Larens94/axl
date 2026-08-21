use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use axl_core::{Value, InterpreterConfig, Tool};

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(8000);
    let source_path = std::env::args().nth(2)
        .unwrap_or_else(|| "examples/netflix/streaming_home.axl".into());

    // Build frontend
    let source = std::fs::read_to_string(&source_path).expect("cannot read source");
    let program = axl_core::parse_compact(&source).expect("parse error");
    axl_core::validate(&program).expect("validation error");
    let output_dir = std::path::PathBuf::from("build/netflix");
    axl_core::build_web(&program, &output_dir).expect("build error");

    // Build backend agents
    let backend_source = std::fs::read_to_string("examples/netflix/backend.axl")
        .unwrap_or_else(|_| "2;12|\"Netflix backend\"".into());
    let backend_program = axl_core::parse_compact(&backend_source).unwrap_or_else(|_| {
        axl_core::Program { instructions: vec![axl_core::Instruction::Emit(axl_core::Expression::Literal(Value::String("backend init".into())))] }
    });

    // Create tools for the agents
    let catalog = build_catalog();
    let catalog_for_search = catalog.clone();
    let mut search_tool = Tool::new("search_catalog", Box::new(move |args: &[Value]| {
        let query = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("query must be string".into()),
        };
        let results: Vec<&(String, String, String, i32)> = catalog_for_search.iter()
            .filter(|(title, _, _, _)| title.to_lowercase().contains(&query.to_lowercase()))
            .collect();
        if results.is_empty() {
            Ok(Value::String(format!("No results for '{query}'")))
        } else {
            let titles: Vec<String> = results.iter().map(|(t, _, _, _)| format!("\"{t}\"")).collect();
            Ok(Value::String(format!("Found {} results: {}", results.len(), titles.join(", "))))
        }
    }));
    search_tool.effect = "read".into();

    let catalog_for_recs = catalog.clone();
    let mut recommend_tool = Tool::new("get_recommendations", Box::new(move |args: &[Value]| {
        let genre = match args.get(1) {
            Some(Value::String(s)) => s.clone(),
            _ => "all".to_string(),
        };
        let filtered: Vec<&(String, String, String, i32)> = catalog_for_recs.iter()
            .filter(|(_, g, _, _)| genre == "all" || g.to_lowercase().contains(&genre.to_lowercase()))
            .take(5)
            .collect();
        let titles: Vec<String> = filtered.iter().map(|(t, _, _, _)| format!("\"{t}\"")).collect();
        Ok(Value::String(format!("Top {} recommendations: {}", titles.len(), titles.join(", "))))
    }));
    recommend_tool.effect = "read".into();

    let users = build_users();
    let users_for_profile = users.clone();
    let mut profile_tool = Tool::new("get_profile", Box::new(move |args: &[Value]| {
        let uid = match &args[0] {
            Value::Int(n) => n,
            _ => return Err("user_id must be int".into()),
        };
        match users_for_profile.get(&uid) {
            Some(name) => Ok(Value::String(format!("User {uid}: {name}"))),
            None => Ok(Value::String(format!("User {uid}: guest"))),
        }
    }));
    profile_tool.effect = "read".into();

    let mut watchlist_tool = Tool::new("manage_watchlist", Box::new(move |args: &[Value]| {
        let action = match &args[0] {
            Value::String(s) => s.clone(),
            _ => "list".to_string(),
        };
        let content_id = match args.get(1) {
            Some(Value::Int(n)) => *n,
            _ => 0,
        };
        Ok(Value::String(format!("Watchlist {action}: content {content_id}")))
    }));
    watchlist_tool.effect = "write".into();

    // Initialize backend agents
    let memory_store: std::sync::Arc<std::sync::Mutex<dyn axl_core::MemoryStore>> =
        std::sync::Arc::new(std::sync::Mutex::new(axl_core::InMemoryStore::new()));
    let config = InterpreterConfig {
        max_steps: 1000,
        scope: "netflix:session".into(),
        ..Default::default()
    };
    let _ = axl_core::run_program(
        &backend_program,
        vec![search_tool, recommend_tool, profile_tool, watchlist_tool],
        memory_store,
        config,
        None,
    );

    let listener = TcpListener::bind(("127.0.0.1", port)).expect("cannot bind");
    println!("=== Netflix AXL Server ===");
    println!("Frontend: http://localhost:{port}");
    println!("API endpoints:");
    println!("  GET /api/search?q=...");
    println!("  GET /api/recommendations?genre=...");
    println!("  GET /api/profile?id=1");
    println!("  GET /api/catalog");
    println!("Backend agents: initialized with {} titles", catalog.len());

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            handle_request(&mut stream, &output_dir);
        }
    }
}

fn handle_request(stream: &mut TcpStream, root: &Path) {
    let mut request = [0u8; 8192];
    let size = stream.read(&mut request).unwrap_or(0);
    let first_line = String::from_utf8_lossy(&request[..size]);
    let target = first_line.lines().next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");

    let (path, query) = parse_url(target);

    match path.as_str() {
        "/" | "/index.html" => serve_file(stream, root, "index.html", "text/html; charset=utf-8"),
        "/ax-ui.css" => serve_file(stream, root, "ax-ui.css", "text/css; charset=utf-8"),
        "/ax-ui.js" => serve_file(stream, root, "ax-ui.js", "text/javascript; charset=utf-8"),
        "/api/search" => {
            let q = query.get("q").map(|s| s.as_str()).unwrap_or("");
            serve_json(stream, &search_content(q));
        }
        "/api/recommendations" => {
            let genre = query.get("genre").map(|s| s.as_str()).unwrap_or("all");
            serve_json(stream, &get_recommendations(genre));
        }
        "/api/profile" => {
            let id = query.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
            serve_json(stream, &get_profile(id));
        }
        "/api/catalog" => serve_json(stream, &get_full_catalog()),
        _ => respond(stream, "404 Not Found", "text/plain", b"Not found"),
    }
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

fn serve_file(stream: &mut TcpStream, root: &Path, name: &str, content_type: &str) {
    match std::fs::read(root.join(name)) {
        Ok(body) => respond(stream, "200 OK", content_type, &body),
        Err(_) => respond(stream, "404 Not Found", "text/plain", b"File not found"),
    }
}

fn serve_json(stream: &mut TcpStream, value: &serde_json::Value) {
    let body = serde_json::to_vec_pretty(value).unwrap_or_default();
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

type CatalogEntry = (String, String, String, i32);

fn build_catalog() -> Vec<CatalogEntry> {
    vec![
        ("Stranger Things".into(), "Sci-Fi Horror".into(), "When a young boy vanishes, a small town uncovers a mystery.".into(), 9),
        ("The Witcher".into(), "Fantasy Action".into(), "Geralt of Rivia hunts monsters.".into(), 8),
        ("Squid Game".into(), "Thriller Drama".into(), "Hundreds of cash-strapped players accept a strange invitation.".into(), 10),
        ("Bridgerton".into(), "Romance Period".into(), "The powerful Bridgerton family navigates Regency London.".into(), 7),
        ("Wednesday".into(), "Comedy Mystery".into(), "Wednesday Addams investigates a murder spree.".into(), 8),
        ("Dahmer".into(), "Crime Drama".into(), "Jeffrey Dahmer's story told from his victims' perspectives.".into(), 7),
        ("All Quiet on the Western Front".into(), "War Drama".into(), "A young German soldier on the Western Front.".into(), 10),
        ("Glass Onion".into(), "Mystery Comedy".into(), "A new adventure for Detective Blanc.".into(), 8),
        ("Extraction 2".into(), "Action".into(), "Tyler Rake returns for another mission.".into(), 8),
        ("The Old Guard".into(), "Action".into(), "Immortal warriors fight for justice.".into(), 7),
        ("Red Notice".into(), "Action Comedy".into(), "An FBI profiler partners with an art thief.".into(), 6),
        ("Black Mirror".into(), "Sci-Fi Thriller".into(), "Technology gone wrong in near-future stories.".into(), 9),
        ("Love Death Robots".into(), "Animated Sci-Fi".into(), "A animated anthology of sci-fi stories.".into(), 8),
        ("Altered Carbon".into(), "Cyberpunk Noir".into(), "In a future where consciousness can be transferred.".into(), 8),
        ("The Crown".into(), "Historical Drama".into(), "The reign of Queen Elizabeth II.".into(), 9),
        ("Money Heist".into(), "Crime Thriller".into(), "A criminal mastermind leads a heist.".into(), 9),
        ("Dark".into(), "Sci-Fi Mystery".into(), "A time-travel mystery in a German town.".into(), 10),
        ("Narcos".into(), "Crime Drama".into(), "The rise and fall of Pablo Escobar.".into(), 8),
        ("The Queen's Gambit".into(), "Drama".into(), "A chess prodigy's journey to the top.".into(), 9),
        ("Ozark".into(), "Crime Drama".into(), "A financial advisor drags his family into a money-laundering scheme.".into(), 9),
    ]
}

fn build_users() -> HashMap<i64, String> {
    let mut m = HashMap::new();
    m.insert(1, "Fabrizio".to_string());
    m.insert(2, "Guest User".to_string());
    m
}

fn search_content(query: &str) -> serde_json::Value {
    let catalog = build_catalog();
    let results: Vec<&CatalogEntry> = catalog.iter()
        .filter(|(title, genre, desc, _)|
            title.to_lowercase().contains(&query.to_lowercase()) ||
            genre.to_lowercase().contains(&query.to_lowercase()) ||
            desc.to_lowercase().contains(&query.to_lowercase())
        )
        .collect();

    let items: Vec<serde_json::Value> = results.into_iter().map(|(title, genre, desc, rating)| {
        serde_json::json!({
            "title": title,
            "genre": genre,
            "description": desc,
            "rating": rating,
            "year": 2023
        })
    }).collect();

    serde_json::json!({
        "query": query,
        "count": items.len(),
        "results": items
    })
}

fn get_recommendations(genre: &str) -> serde_json::Value {
    let catalog = build_catalog();
    let filtered: Vec<serde_json::Value> = catalog.iter()
        .filter(|(_, g, _, _)| genre == "all" || g.to_lowercase().contains(&genre.to_lowercase()))
        .take(10)
        .map(|(title, genre, desc, rating)| {
            serde_json::json!({
                "title": title,
                "genre": genre,
                "description": desc,
                "rating": rating
            })
        })
        .collect();

    serde_json::json!({
        "genre": genre,
        "count": filtered.len(),
        "recommendations": filtered
    })
}

fn get_profile(id: i64) -> serde_json::Value {
    let users = build_users();
    let name = users.get(&id).cloned().unwrap_or_else(|| "Guest".to_string());
    serde_json::json!({
        "id": id,
        "name": name,
        "plan": "Premium",
        "profiles": ["Fabrizio", "Guest"],
        "watchlist_count": 12,
        "watching_count": 3
    })
}

fn get_full_catalog() -> serde_json::Value {
    let catalog = build_catalog();
    let items: Vec<serde_json::Value> = catalog.iter().map(|(title, genre, desc, rating)| {
        serde_json::json!({
            "title": title,
            "genre": genre,
            "description": desc,
            "rating": rating
        })
    }).collect();
    serde_json::json!({
        "total": items.len(),
        "catalog": items
    })
}
