use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axl_core::primitives;

struct Agent { id: String, name: String, capabilities: Vec<String>, status: String, programs: i64 }
#[derive(Clone)]
struct ExecRecord { agent: String, status: String, output: String, duration_ms: f64 }
#[derive(Clone)]
struct Message { from: String, to: String, topic: String, payload: String }
#[derive(Clone)]
struct Knowledge { key: String, value: String }
struct State { agents: HashMap<String, Agent>, execs: Vec<ExecRecord>, msgs: Vec<Message>, knowledge: Vec<Knowledge>, metrics: HashMap<String, i64> }

fn now() -> f64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64() }

fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(9000);
    let out = std::path::PathBuf::from("build/agent-platform");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("index.html"), HTML).unwrap();

    let state = Arc::new(Mutex::new(State {
        agents: HashMap::new(), execs: Vec::new(), msgs: Vec::new(),
        knowledge: Vec::new(), metrics: HashMap::new(),
    }));
    {
        let mut s = state.lock().unwrap();
        s.agents.insert("1".into(), Agent { id:"1".into(), name:"Research Agent".into(), capabilities:vec!["search".into(),"analyze".into()], status:"idle".into(), programs:5 });
        s.agents.insert("2".into(), Agent { id:"2".into(), name:"Analysis Agent".into(), capabilities:vec!["classify".into(),"extract".into()], status:"running".into(), programs:3 });
        s.agents.insert("3".into(), Agent { id:"3".into(), name:"Memory Agent".into(), capabilities:vec!["remember".into(),"recall".into()], status:"idle".into(), programs:8 });
        s.metrics.insert("agents".into(), 3); s.metrics.insert("executions".into(), 16);
        s.metrics.insert("messages".into(), 42); s.metrics.insert("knowledge".into(), 25);
    }

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    println!("=== AXL Agent Platform === http://localhost:{port}");
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream { handle(&mut stream, &out, &state); }
    }
}

fn handle(stream: &mut TcpStream, root: &Path, state: &Arc<Mutex<State>>) {
    let mut req = [0u8; 8192];
    let size = stream.read(&mut req).unwrap_or(0);
    let first = String::from_utf8_lossy(&req[..size]);
    let parts: Vec<&str> = first.lines().next().unwrap_or("").split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET");
    let target = parts.get(1).copied().unwrap_or("/");
    let (path, query) = parse_url(target);
    let body_start = first.find("\r\n\r\n").map(|i| i+4).unwrap_or(0);
    let body = String::from_utf8_lossy(&req[body_start..size]).to_string();

    // Static files
    if method == "GET" && !path.starts_with("/api/") {
        let fp = root.join(path.trim_start_matches('/'));
        if let Ok(content) = std::fs::read(&fp) {
            let ct = match fp.extension().and_then(|e| e.to_str()) {
                Some("css") => "text/css; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                _ => "text/html; charset=utf-8",
            };
            return send(stream, 200, ct, &content);
        }
    }

    let resp: (u16, serde_json::Value) = match (method, path.as_str()) {
        ("GET", "/api/agents") => {
            let s = state.lock().unwrap();
            let list: Vec<serde_json::Value> = s.agents.values().map(|a| serde_json::json!({
                "id": a.id, "name": a.name, "status": a.status,
                "capabilities": a.capabilities, "programs": a.programs
            })).collect();
            ok(serde_json::json!({"agents": list, "count": list.len()}))
        }
        ("POST", "/api/agents") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let id = format!("{}", now() as i64);
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed").to_string();
            let caps: Vec<String> = p.get("capabilities").and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            state.lock().unwrap().agents.insert(id.clone(), Agent { id: id.clone(), name: name.clone(), capabilities: caps.clone(), status: "idle".into(), programs: 0 });
            ok(serde_json::json!({"id": id, "name": name, "capabilities": caps}))
        }
        ("DELETE", _) if path.starts_with("/api/agents/") => {
            let id = path.trim_start_matches("/api/agents/").to_string();
            if state.lock().unwrap().agents.remove(&id).is_some() { ok(serde_json::json!({"deleted": id})) }
            else { err(404, "agent not found") }
        }
        ("POST", "/api/execute") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let agent_id = p.get("agent_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let src = p.get("program").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let prog = match axl_core::parse_compact(&src) { Ok(p) => p, Err(e) => return send_json(stream, 400, &serde_json::json!({"error": format!("{e}")})), };
            if let Err(e) = axl_core::validate(&prog) { return send_json(stream, 400, &serde_json::json!({"error": format!("{e}")})); }
            let mem: std::sync::Arc<std::sync::Mutex<dyn axl_core::MemoryStore>> = Arc::new(Mutex::new(axl_core::InMemoryStore::new()));
            let cfg = axl_core::InterpreterConfig { max_steps: 10000, scope: format!("agent:{agent_id}"), ..Default::default() };
            let t0 = now();
            let res = axl_core::run_program(&prog, vec![], mem, cfg, None);
            let dur = (now() - t0) * 1000.0;
            let (st, out): (String, String) = match res { Ok(r) => ("success".into(), r.output.iter().map(|v| axl_core::render_value(v).unwrap_or_default()).collect::<Vec<_>>().join("\n")), Err(e) => ("error".into(), e.to_string()) };
            let mut s = state.lock().unwrap(); s.execs.push(ExecRecord { agent: agent_id, status: st.clone(), output: out.clone(), duration_ms: dur });
            *s.metrics.entry("executions".into()).or_insert(0) += 1;
            return send_json(stream, 200, &serde_json::json!({"status": st, "output": out, "duration_ms": dur}));
        }
        ("GET", "/api/executions") => {
            let s = state.lock().unwrap();
            let list: Vec<serde_json::Value> = s.execs.iter().rev().take(10).map(|e| serde_json::json!({"agent": e.agent, "status": e.status, "output": e.output, "duration_ms": e.duration_ms})).collect();
            ok(serde_json::json!({"executions": list, "total": s.execs.len()}))
        }
        ("POST", "/api/messages") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let m = Message { from: p.get("from").and_then(|v| v.as_str()).unwrap_or("").into(), to: p.get("to").and_then(|v| v.as_str()).unwrap_or("*").into(), topic: p.get("topic").and_then(|v| v.as_str()).unwrap_or("general").into(), payload: p.get("payload").and_then(|v| v.as_str()).unwrap_or("").into() };
            state.lock().unwrap().msgs.push(m.clone());
            ok(serde_json::json!({"from": m.from, "to": m.to, "topic": m.topic, "payload": m.payload}))
        }
        ("GET", "/api/messages") => {
            let s = state.lock().unwrap();
            let to = query.get("to").map(|s| s.as_str()).unwrap_or("*");
            let list: Vec<serde_json::Value> = s.msgs.iter().rev().take(50).filter(|m| to == "*" || m.to == to || m.to == "*").map(|m| serde_json::json!({"from": m.from, "to": m.to, "topic": m.topic, "payload": m.payload})).collect();
            ok(serde_json::json!({"messages": list, "count": list.len()}))
        }
        ("POST", "/api/knowledge") => {
            let p: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::json!({}));
            let k = Knowledge { key: p.get("key").and_then(|v| v.as_str()).unwrap_or("").into(), value: p.get("value").and_then(|v| v.as_str()).unwrap_or("").into() };
            state.lock().unwrap().knowledge.push(k.clone());
            *state.lock().unwrap().metrics.entry("knowledge".into()).or_insert(0) += 1;
            ok(serde_json::json!({"key": k.key, "value": k.value}))
        }
        ("GET", "/api/knowledge") => {
            let s = state.lock().unwrap();
            let list: Vec<serde_json::Value> = s.knowledge.iter().map(|k| serde_json::json!({"key": k.key, "value": k.value})).collect();
            ok(serde_json::json!({"entries": list, "count": list.len()}))
        }
        ("GET", "/api/metrics") => {
            let s = state.lock().unwrap();
            let list: Vec<serde_json::Value> = s.metrics.iter().map(|(k, v)| serde_json::json!({"name": k, "value": v})).collect();
            ok(serde_json::json!({"metrics": list, "primitives": primitives::available_primitives().len()}))
        }
        ("GET", "/api/system") => {
            let s = state.lock().unwrap();
            ok(serde_json::json!({"platform": "AXL Agent Platform", "version": "1.0", "agents": s.agents.len(), "executions": s.execs.len(), "messages": s.msgs.len(), "knowledge": s.knowledge.len()}))
        }
        _ => err(404, "not found"),
    };
    send_json(stream, resp.0, &resp.1);
}

fn ok(v: serde_json::Value) -> (u16, serde_json::Value) { (200, v) }
fn err(s: u16, m: &str) -> (u16, serde_json::Value) { (s, serde_json::json!({"error": m})) }

fn send_json(stream: &mut TcpStream, status: u16, value: &serde_json::Value) {
    let body = serde_json::to_string(value).unwrap();
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
    if let Some(qs) = parts.next() { for p in qs.split('&') { if let Some((k, v)) = p.split_once('=') { q.insert(k.to_string(), v.to_string()); } } }
    (path, q)
}

const HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>AXL Agent Platform</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { background: #090909; color: #f5f5f1; font-family: system-ui, sans-serif; padding: 20px; }
    h1 { color: #e50914; margin-bottom: 20px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-bottom: 30px; }
    .card { background: #1a1a1a; border-radius: 8px; padding: 20px; }
    .card h2 { font-size: 18px; margin-bottom: 12px; }
    .metric { font-size: 36px; font-weight: 700; color: #46d369; }
    .label { font-size: 14px; color: #888; }
    .btn { background: #e50914; border: 0; color: white; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-size: 14px; }
    .btn:hover { background: #ff1a25; }
    textarea { width: 100%; height: 80px; background: #0d0d0d; border: 1px solid #333; color: #fff; padding: 10px; border-radius: 6px; font-family: monospace; font-size: 14px; margin: 10px 0; }
    pre { background: #0d0d0d; padding: 10px; border-radius: 6px; font-size: 12px; overflow-x: auto; margin-top: 10px; }
  </style>
</head>
<body>
  <h1>AXL Agent Platform</h1>
  <div class="grid">
    <div class="card"><h2>Agents</h2><div class="metric" id="agents">3</div><div class="label">registered</div></div>
    <div class="card"><h2>Executions</h2><div class="metric" id="execs">16</div><div class="label">programs run</div></div>
    <div class="card"><h2>Messages</h2><div class="metric" id="msgs">42</div><div class="label">sent</div></div>
    <div class="card"><h2>Knowledge</h2><div class="metric" id="know">25</div><div class="label">entries</div></div>
  </div>
  <div class="card" style="margin-bottom:20px">
    <h2>Execute AXL Program</h2>
    <textarea id="program">2;10|r|"hello",!text_upper/1|s;12|$r</textarea>
    <button class="btn" onclick="execute()">Run</button>
    <pre id="output">Output will appear here...</pre>
  </div>
  <script>
    async function execute() {
      const program = document.getElementById('program').value;
      const res = await fetch('/api/execute', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({agent_id: '1', program}) });
      const data = await res.json();
      document.getElementById('output').textContent = JSON.stringify(data, null, 2);
    }
    async function refresh() {
      const res = await fetch('/api/metrics');
      const data = await res.json();
      if (data.metrics) data.metrics.forEach(m => { const el = document.getElementById(m.name); if (el) el.textContent = m.value; });
    }
    setInterval(refresh, 5000);
  </script>
</body>
</html>"#;
