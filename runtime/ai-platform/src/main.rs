use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

use axl_core::{Value, InterpreterConfig, Tool, LlmBackend, MockBackend};

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(8000);
    let source_path = std::env::args().nth(2)
        .unwrap_or_else(|| "examples/ai-platform/platform.axl".into());

    // Build frontend
    let source = std::fs::read_to_string(&source_path).expect("cannot read source");
    let program = axl_core::parse_compact(&source).expect("parse error");
    axl_core::validate(&program).expect("validation error");
    let output_dir = std::path::PathBuf::from("build/ai-platform");
    axl_core::build_web(&program, &output_dir).expect("build error");

    // Initialize LLM backend (mock for demo)
    let llm = Arc::new(MockBackend::new(vec![
        "Category: Technology | Sentiment: Positive | Entities: AXL, AI, Agents".into(),
        "The article discusses advances in AI agent programming languages. Key entities: AXL (language), OpenAI (company), Anthropic (company). Sentiment is predominantly positive.".into(),
        "Based on the analysis, this content is a technical tutorial about agent-native programming. It covers LLM primitives, semantic memory, and inter-agent communication.".into(),
    ]));

    // Create agent tools
    let search_tool = Tool::new("search_catalog", Box::new(move |args: &[Value]| {
        let query = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("query must be string".into()),
        };
        let catalog = build_content_catalog();
        let results: Vec<&(String, String, String, i32)> = catalog.iter()
            .filter(|(title, cat, desc, _)| 
                title.to_lowercase().contains(&query.to_lowercase()) ||
                cat.to_lowercase().contains(&query.to_lowercase()) ||
                desc.to_lowercase().contains(&query.to_lowercase())
            )
            .collect();
        if results.is_empty() {
            Ok(Value::String(format!("No results for '{query}'")))
        } else {
            let items: Vec<String> = results.iter().map(|(t, c, _, _)| format!("\"{t}\" [{c}]")).collect();
            Ok(Value::String(format!("Found {} items: {}", results.len(), items.join(", "))))
        }
    }));

    let llm_for_classify = llm.clone();
    let classify_tool = Tool::new("classify_content", Box::new(move |args: &[Value]| {
        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("text must be string".into()),
        };
        let system = "Classify the following text into one of: news, opinion, review, tutorial. Reply with ONLY the category.";
        let messages = vec![("user".to_string(), text)];
        let result = llm_for_classify.generate(system, &messages).map_err(|e| e.to_string())?;
        Ok(Value::String(result))
    }));

    let llm_for_sentiment = llm.clone();
    let sentiment_tool = Tool::new("analyze_sentiment", Box::new(move |args: &[Value]| {
        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("text must be string".into()),
        };
        let system = "Analyze the sentiment of the following text. Reply with: positive, negative, or neutral.";
        let messages = vec![("user".to_string(), text)];
        let result = llm_for_sentiment.generate(system, &messages).map_err(|e| e.to_string())?;
        Ok(Value::String(result))
    }));

    let llm_for_extract = llm.clone();
    let extract_tool = Tool::new("extract_entities", Box::new(move |args: &[Value]| {
        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("text must be string".into()),
        };
        let system = "Extract person names, organizations, and key topics from the text. One per line.";
        let messages = vec![("user".to_string(), text)];
        let result = llm_for_extract.generate(system, &messages).map_err(|e| e.to_string())?;
        let entities: Vec<String> = result.lines().map(String::from).filter(|l| !l.is_empty()).collect();
        Ok(Value::List(entities.into_iter().map(Value::String).collect()))
    }));

    let llm_for_reason = llm.clone();
    let reason_tool = Tool::new("reason_content", Box::new(move |args: &[Value]| {
        let query = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("query must be string".into()),
        };
        let system = "You are a thoughtful AI assistant. Think step by step and provide a clear, reasoned response.";
        let messages = vec![("user".to_string(), query)];
        let result = llm_for_reason.generate(system, &messages).map_err(|e| e.to_string())?;
        Ok(Value::String(result))
    }));

    let llm_for_embed = llm.clone();
    let embed_tool = Tool::new("embed_text", Box::new(move |args: &[Value]| {
        let text = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err("text must be string".into()),
        };
        let embedding = llm_for_embed.embed(&text).map_err(|e| e.to_string())?;
        Ok(Value::Embedding(embedding))
    }));

    // Initialize agents
    let memory_store: Arc<Mutex<dyn axl_core::MemoryStore>> = Arc::new(Mutex::new(axl_core::InMemoryStore::new()));
    let config = InterpreterConfig {
        max_steps: 5000,
        scope: "ai-platform:session".into(),
        ..Default::default()
    };
    let _ = axl_core::run_program(
        &program,
        vec![search_tool, classify_tool, sentiment_tool, extract_tool, reason_tool, embed_tool],
        memory_store,
        config,
        None,
    );

    let listener = TcpListener::bind(("127.0.0.1", port)).expect("cannot bind");
    println!("=== AI Content Platform (AXL 3.0) ===");
    println!("Frontend: http://localhost:{port}");
    println!("API endpoints:");
    println!("  POST /api/analyze      - Full content analysis");
    println!("  POST /api/classify     - Classify content");
    println!("  POST /api/sentiment    - Analyze sentiment");
    println!("  POST /api/extract      - Extract entities");
    println!("  POST /api/reason       - Chain-of-thought reasoning");
    println!("  POST /api/embed        - Generate embedding");
    println!("  GET  /api/search?q=... - Search content catalog");
    println!("  GET  /api/catalog      - Full catalog");
    println!("  GET  /api/agents       - List available agents");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            handle_request(&mut stream, &output_dir, &llm);
        }
    }
}

fn handle_request(stream: &mut TcpStream, root: &Path, llm: &Arc<MockBackend>) {
    let mut request = [0u8; 8192];
    let size = stream.read(&mut request).unwrap_or(0);
    let first_line = String::from_utf8_lossy(&request[..size]);
    let request_line = first_line.lines().next().unwrap_or("");
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("");
    let target = parts.get(1).copied().unwrap_or("/");

    let body_start = first_line.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = String::from_utf8_lossy(&request[body_start..size]).to_string();

    let (path, query) = parse_url(target);

    match (method, path.as_str()) {
        ("GET", "/" | "/index.html") => serve_file(stream, root, "index.html", "text/html; charset=utf-8"),
        ("GET", "/ax-ui.css") => serve_file(stream, root, "ax-ui.css", "text/css; charset=utf-8"),
        ("GET", "/ax-ui.js") => serve_file(stream, root, "ax-ui.js", "text/javascript; charset=utf-8"),
        ("GET", "/api/search") => {
            let q = query.get("q").map(|s| s.as_str()).unwrap_or("");
            serve_json(stream, &api_search(q));
        }
        ("GET", "/api/catalog") => serve_json(stream, &api_catalog()),
        ("GET", "/api/agents") => serve_json(stream, &api_agents()),
        ("POST", "/api/analyze") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_analyze(llm, &input));
        }
        ("POST", "/api/classify") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_classify(llm, &input));
        }
        ("POST", "/api/sentiment") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_sentiment(llm, &input));
        }
        ("POST", "/api/extract") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_extract(llm, &input));
        }
        ("POST", "/api/reason") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_reason(llm, &input));
        }
        ("POST", "/api/embed") => {
            let input = parse_json_body(&body);
            serve_json(stream, &api_embed(llm, &input));
        }
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

fn parse_json_body(body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(obj) = json.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
        }
    }
    map
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
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n",
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

// ============================================================================
// API Implementations
// ============================================================================

fn api_search(query: &str) -> serde_json::Value {
    let catalog = build_content_catalog();
    let results: Vec<&(String, String, String, i32)> = catalog.iter()
        .filter(|(title, cat, desc, _)|
            query.is_empty() ||
            title.to_lowercase().contains(&query.to_lowercase()) ||
            cat.to_lowercase().contains(&query.to_lowercase()) ||
            desc.to_lowercase().contains(&query.to_lowercase())
        )
        .collect();

    let items: Vec<serde_json::Value> = results.into_iter().map(|(title, cat, desc, rating)| {
        serde_json::json!({
            "title": title,
            "category": cat,
            "description": desc,
            "rating": rating
        })
    }).collect();

    serde_json::json!({
        "query": query,
        "count": items.len(),
        "results": items
    })
}

fn api_catalog() -> serde_json::Value {
    let catalog = build_content_catalog();
    let items: Vec<serde_json::Value> = catalog.iter().map(|(title, cat, desc, rating)| {
        serde_json::json!({
            "title": title,
            "category": cat,
            "description": desc,
            "rating": rating
        })
    }).collect();
    serde_json::json!({ "total": items.len(), "catalog": items })
}

fn api_agents() -> serde_json::Value {
    serde_json::json!({
        "agents": [
            {
                "name": "content_agent",
                "description": "Search and analyze content",
                "capabilities": ["search", "classify", "sentiment", "extract"],
                "tools": ["search_catalog", "classify_content", "analyze_sentiment", "extract_entities"]
            },
            {
                "name": "reasoning_agent",
                "description": "Chain-of-thought reasoning",
                "capabilities": ["reason", "explain", "summarize"],
                "tools": ["reason_content"]
            },
            {
                "name": "memory_agent",
                "description": "Semantic memory operations",
                "capabilities": ["remember", "recall", "search_similar"],
                "tools": ["embed_text"]
            }
        ]
    })
}

fn api_analyze(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "No text provided".to_string();
    let text = input.get("text").unwrap_or(&default_text);
    let system = "Analyze this content comprehensively. Provide: 1) Category, 2) Sentiment, 3) Key entities, 4) Summary.";
    let messages = vec![("user".to_string(), text.clone())];
    let result = llm.generate(system, &messages).unwrap_or_else(|_| "Analysis failed".into());
    serde_json::json!({
        "input": text,
        "analysis": result,
        "agent": "content_agent",
        "primitives_used": ["classify", "sentiment", "extract", "reason"]
    })
}

fn api_classify(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "".to_string();
    let text = input.get("text").unwrap_or(&default_text);
    let system = "Classify this content into one category: news, opinion, review, tutorial, entertainment. Reply with ONLY the category name.";
    let messages = vec![("user".to_string(), text.clone())];
    let category = llm.generate(system, &messages).unwrap_or_else(|_| "unknown".into());
    serde_json::json!({
        "input": text,
        "category": category.trim(),
        "agent": "content_agent",
        "primitive": "classify"
    })
}

fn api_sentiment(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "".to_string();
    let text = input.get("text").unwrap_or(&default_text);
    let system = "Analyze the sentiment of this text. Reply with: positive, negative, or neutral.";
    let messages = vec![("user".to_string(), text.clone())];
    let sentiment = llm.generate(system, &messages).unwrap_or_else(|_| "neutral".into());
    serde_json::json!({
        "input": text,
        "sentiment": sentiment.trim(),
        "agent": "content_agent",
        "primitive": "sentiment"
    })
}

fn api_extract(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "".to_string();
    let text = input.get("text").unwrap_or(&default_text);
    let system = "Extract key entities from this text: people names, organizations, topics. One per line.";
    let messages = vec![("user".to_string(), text.clone())];
    let result = llm.generate(system, &messages).unwrap_or_default();
    let entities: Vec<&str> = result.lines().filter(|l| !l.trim().is_empty()).collect();
    serde_json::json!({
        "input": text,
        "entities": entities,
        "count": entities.len(),
        "agent": "content_agent",
        "primitive": "extract"
    })
}

fn api_reason(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "".to_string();
    let query = input.get("query").or(input.get("text")).unwrap_or(&default_text);
    let system = "You are a thoughtful AI assistant. Think step by step, show your reasoning, then provide a clear answer.";
    let messages = vec![("user".to_string(), query.clone())];
    let reasoning = llm.generate(system, &messages).unwrap_or_else(|_| "Reasoning failed".into());
    serde_json::json!({
        "query": query,
        "reasoning": reasoning,
        "agent": "reasoning_agent",
        "primitive": "reason"
    })
}

fn api_embed(llm: &Arc<MockBackend>, input: &HashMap<String, String>) -> serde_json::Value {
    let default_text = "".to_string();
    let text = input.get("text").unwrap_or(&default_text);
    let embedding = llm.embed(text).unwrap_or_default();
    let preview: Vec<i64> = embedding.iter().take(10).cloned().collect();
    serde_json::json!({
        "input": text,
        "dimensions": embedding.len(),
        "preview": preview,
        "agent": "memory_agent",
        "primitive": "embed"
    })
}

// ============================================================================
// Content Catalog
// ============================================================================

type CatalogEntry = (String, String, String, i32);

fn build_content_catalog() -> Vec<CatalogEntry> {
    vec![
        ("Introduction to Agent Programming".into(), "tutorial".into(), "Learn how to build AI agents with AXL 3.0".into(), 10),
        ("The Future of AI Agents".into(), "opinion".into(), "Why agent-native languages are the future".into(), 9),
        ("AXL 3.0 Release Notes".into(), "news".into(), "New LLM primitives and semantic memory".into(), 9),
        ("Building a Chatbot with AXL".into(), "tutorial".into(), "Step-by-step guide to agent communication".into(), 8),
        ("AI Agent Security Best Practices".into(), "news".into(), "How to secure agent tool permissions".into(), 9),
        ("Semantic Memory in Practice".into(), "tutorial".into(), "Using recall_semantic for better search".into(), 8),
        ("Agent-to-Agent Communication".into(), "tutorial".into(), "Send, delegate, and broadcast patterns".into(), 9),
        ("LLM Primitives Deep Dive".into(), "tutorial".into(), "Understanding reason, classify, extract".into(), 10),
        ("Event-Driven Agent Architecture".into(), "opinion".into(), "Why events matter for autonomous agents".into(), 8),
        ("Memory Persistence with AXL".into(), "tutorial".into(), "Storing and retrieving semantic memories".into(), 7),
    ]
}
