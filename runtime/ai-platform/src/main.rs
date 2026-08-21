use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};

use axl_core::{Value, InterpreterConfig, Tool, LlmBackend, mimo::MiMoBackend};

const MIMO_API_KEY: &str = "sk-ejmpfhhrc5eyh9n1bwp2yn0dt1vtghqclesto54fnju5my9c";

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(8000);
    let out_dir = std::path::PathBuf::from("build/ai-platform");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(out_dir.join("index.html"), HTML).unwrap();

    let llm: Arc<dyn LlmBackend> = Arc::new(MiMoBackend::new(MIMO_API_KEY.to_string()));
    println!("=== AI Content Platform (MiMo Backend) ===");
    println!("Frontend: http://localhost:{port}");
    println!("LLM: MiMo mimo-v2.5-pro");

    let root = out_dir;
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream { handle(&mut stream, &root, &llm); }
    }
}

fn handle(stream: &mut TcpStream, root: &Path, llm: &Arc<dyn LlmBackend>) {
    let mut req = [0u8; 8192];
    let size = stream.read(&mut req).unwrap_or(0);
    let first = String::from_utf8_lossy(&req[..size]);
    let parts: Vec<&str> = first.lines().next().unwrap_or("").split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET");
    let target = parts.get(1).copied().unwrap_or("/");
    let (path, query) = parse_url(target);
    let body_start = first.find("\r\n\r\n").map(|i| i+4).unwrap_or(0);
    let body = String::from_utf8_lossy(&req[body_start..size]).to_string();

    if method == "GET" && (path == "/" || path == "/index.html") {
        if let Ok(content) = std::fs::read(root.join("index.html")) {
            return send(stream, 200, "text/html; charset=utf-8", &content);
        }
    }

    let resp = match (method, path.as_str()) {
        ("GET", "/api/search") => {
            let q = query.get("q").map(|s| s.as_str()).unwrap_or("");
            let catalog = build_catalog();
            let results: Vec<serde_json::Value> = catalog.iter()
                .filter(|(t,c,d,_)| q.is_empty() || t.to_lowercase().contains(q) || c.to_lowercase().contains(q) || d.to_lowercase().contains(q))
                .map(|(t,c,d,r)| serde_json::json!({"title":t,"category":c,"description":d,"rating":r}))
                .collect();
            (200, serde_json::json!({"query":q,"count":results.len(),"results":results}))
        }
        ("GET", "/api/catalog") => {
            let catalog = build_catalog();
            let items: Vec<serde_json::Value> = catalog.iter().map(|(t,c,d,r)| serde_json::json!({"title":t,"category":c,"description":d,"rating":r})).collect();
            (200, serde_json::json!({"total":items.len(),"catalog":items}))
        }
        ("GET", "/api/agents") => (200, serde_json::json!({"agents":[{"name":"content_agent","capabilities":["search","classify","sentiment","extract"]},{"name":"reasoning_agent","capabilities":["reason","explain"]},{"name":"memory_agent","capabilities":["remember","recall"]}] })),
        ("POST", "/api/classify") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let text = p.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match llm.generate("Classify into: news, opinion, review, tutorial. Reply ONLY the category.", &[("user".into(), text.to_string())]) {
                Ok(r) => (200, serde_json::json!({"input":text,"category":r.trim(),"llm":"mimo-v2.5-pro"})),
                Err(e) => (500, serde_json::json!({"error":e.to_string()})),
            }
        }
        ("POST", "/api/sentiment") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let text = p.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match llm.generate("Analyze sentiment: positive, negative, or neutral. Reply ONLY the sentiment.", &[("user".into(), text.to_string())]) {
                Ok(r) => (200, serde_json::json!({"input":text,"sentiment":r.trim(),"llm":"mimo-v2.5-pro"})),
                Err(e) => (500, serde_json::json!({"error":e.to_string()})),
            }
        }
        ("POST", "/api/extract") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let text = p.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match llm.generate("Extract people, organizations, topics. One per line.", &[("user".into(), text.to_string())]) {
                Ok(r) => { let e: Vec<&str> = r.lines().filter(|l|!l.trim().is_empty()).collect(); (200, serde_json::json!({"input":text,"entities":e,"count":e.len(),"llm":"mimo-v2.5-pro"})) },
                Err(e) => (500, serde_json::json!({"error":e.to_string()})),
            }
        }
        ("POST", "/api/reason") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let q = p.get("query").and_then(|v| v.as_str()).unwrap_or("");
            match llm.generate("Think step by step, show reasoning, then give a clear answer.", &[("user".into(), q.to_string())]) {
                Ok(r) => (200, serde_json::json!({"query":q,"reasoning":r,"llm":"mimo-v2.5-pro"})),
                Err(e) => (500, serde_json::json!({"error":e.to_string()})),
            }
        }
        ("POST", "/api/embed") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let text = p.get("text").and_then(|v| v.as_str()).unwrap_or("");
            match llm.embed(text) {
                Ok(e) => { let p: Vec<i64> = e.iter().take(10).cloned().collect(); (200, serde_json::json!({"input":text,"dimensions":e.len(),"preview":p,"llm":"mimo-v2.5-pro"})) },
                Err(e) => (500, serde_json::json!({"error":e.to_string()})),
            }
        }
        _ => (404, serde_json::json!({"error":"not found"})),
    };
    send_json(stream, resp.0, &resp.1);
}

fn send_json(stream: &mut TcpStream, status: u16, value: &serde_json::Value) {
    let body = serde_json::to_string_pretty(value).unwrap();
    let hdr = format!("HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(hdr.as_bytes()); let _ = stream.write_all(body.as_bytes());
}

fn send(stream: &mut TcpStream, status: u16, ct: &str, body: &[u8]) {
    let hdr = format!("HTTP/1.1 {status}\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    let _ = stream.write_all(hdr.as_bytes()); let _ = stream.write_all(body);
}

fn parse_url(url: &str) -> (String, HashMap<String, String>) {
    let mut parts = url.splitn(2, '?');
    let path = parts.next().unwrap_or("/").to_string();
    let mut q = HashMap::new();
    if let Some(qs) = parts.next() { for p in qs.split('&') { if let Some((k,v)) = p.split_once('=') { q.insert(k.to_string(), v.to_string()); } } }
    (path, q)
}

type CE = (String, String, String, i32);
fn build_catalog() -> Vec<CE> { vec![
    ("Introduction to Agent Programming".into(),"tutorial".into(),"Learn how to build AI agents with AXL 3.0".into(),10),
    ("The Future of AI Agents".into(),"opinion".into(),"Why agent-native languages are the future".into(),9),
    ("AXL 3.0 Release Notes".into(),"news".into(),"New LLM primitives and semantic memory".into(),9),
    ("Building a Chatbot with AXL".into(),"tutorial".into(),"Step-by-step guide to agent communication".into(),8),
    ("AI Agent Security Best Practices".into(),"news".into(),"How to secure agent tool permissions".into(),9),
    ("Semantic Memory in Practice".into(),"tutorial".into(),"Using recall_semantic for better search".into(),8),
    ("Agent-to-Agent Communication".into(),"tutorial".into(),"Send, delegate, and broadcast patterns".into(),9),
    ("LLM Primitives Deep Dive".into(),"tutorial".into(),"Understanding reason, classify, extract".into(),10),
    ("Event-Driven Agent Architecture".into(),"opinion".into(),"Why events matter for autonomous agents".into(),8),
    ("Memory Persistence with AXL".into(),"tutorial".into(),"Storing and retrieving semantic memories".into(),7),
]}

const HTML: &str = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>AI Platform — MiMo</title><style>*{box-sizing:border-box;margin:0;padding:0}body{background:#090909;color:#f5f5f1;font-family:system-ui,sans-serif}.hdr{background:linear-gradient(135deg,#1a1a2e,#16213e);padding:40px;text-align:center}.hdr h1{color:#e50914;font-size:36px;margin-bottom:10px}.hdr p{color:#888}.c{max-width:1200px;margin:0 auto;padding:20px}.g{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:20px;margin:20px 0}.card{background:#1a1a1a;border-radius:8px;padding:20px}.card h3{color:#e50914;margin-bottom:10px}.btn{background:#e50914;border:0;color:#fff;padding:12px 24px;border-radius:6px;cursor:pointer;font-size:14px}.btn:hover{background:#ff1a25}textarea{width:100%;height:80px;background:#0d0d0d;border:1px solid #333;color:#fff;padding:12px;border-radius:6px;font-family:monospace;font-size:14px;margin:10px 0}pre{background:#0d0d0d;padding:12px;border-radius:6px;font-size:13px;overflow:auto;margin-top:10px;max-height:300px;white-space:pre-wrap}.mimo{background:#2d1a4a;color:#a855f7;padding:4px 12px;border-radius:20px;font-size:12px;font-weight:600}</style></head><body><div class="hdr"><h1>AI Content Platform</h1><p>Powered by <span class="mimo">MiMo mimo-v2.5-pro</span></p></div><div class="c"><div class="card" style="margin-bottom:20px"><h3>Chain-of-Thought Reasoning</h3><textarea id="ri" placeholder="Ask a question...">What are the benefits of agent-native programming?</textarea><button class="btn" onclick="reason()">Reason with MiMo</button><pre id="ro">Click Reason...</pre></div><div class="g"><div class="card"><h3>Classification</h3><textarea id="ci">AXL is a new programming language for AI agents.</textarea><button class="btn" onclick="classify()">Classify</button><pre id="co">Result...</pre></div><div class="card"><h3>Sentiment</h3><textarea id="si">This AI language is absolutely amazing!</textarea><button class="btn" onclick="sentiment()">Analyze</button><pre id="so">Result...</pre></div><div class="card"><h3>Entity Extraction</h3><textarea id="ei">Elon Musk founded Tesla in Austin, Texas.</textarea><button class="btn" onclick="extract()">Extract</button><pre id="eo">Result...</pre></div></div></div><script>async function api(e,i,o){const t=document.getElementById(i).value;document.getElementById(o).textContent='Processing...';try{const r=await fetch(e,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({text:t})});const d=await r.json();document.getElementById(o).textContent=JSON.stringify(d,null,2)}catch(x){document.getElementById(o).textContent='Error: '+x.message}}async function reason(){const t=document.getElementById('ri').value;document.getElementById('ro').textContent='Reasoning...';try{const r=await fetch('/api/reason',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({query:t})});const d=await r.json();document.getElementById('ro').textContent=d.reasoning||JSON.stringify(d,null,2)}catch(x){document.getElementById('ro').textContent='Error: '+x.message}}function classify(){api('/api/classify','ci','co')}function sentiment(){api('/api/sentiment','si','so')}function extract(){api('/api/extract','ei','eo')}</script></body></html>"#;
