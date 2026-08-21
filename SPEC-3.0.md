# AXL 3.0 — Agent eXecution Language

## 1. Identità

AXL è un linguaggio **agent-native**: il programmeer scrive agenti, non程序. Gli agenti hanno goal, ragionano, usano tools, comunicano tra loro, e persistono memoria.

**Principi:**
- L'agente è la prima classe (non la funzione, non il programma)
- Il reasoning LLM è nativo, non un tool esterno
- Gli agenti sono event-driven, non sequenziali
- La memoria è semantica, non solo key-value
- La comunicazione è un protocollo, non una chiamata

## 2. Pipeline

```
AXL Source 3.0
→ parser deterministico
→ AX-IR 2.0 (agent-centric)
→ type-check
→ AX-MIR 2.0
→ runtime
  ├─ LLM backend (reason, generate, parse)
  ├─ Tool runtime (discovery, execution)
  ├─ Agent runtime (messaging, scheduling)
  ├─ Memory runtime (semantic, persistent)
  └─ Bridge (native, WASM, VM)
```

## 3. Source Format

Supporta due frontend:
- **Compact** (RPN, opcode numerici) — per generazione automatica
- **Readable** (keyword-based) — per sviluppo umano

### 3.1 Keyword Syntax (nuovo)

```axl
agent search_agent {
  goal "find relevant content"
  
  tool web_search { effect read }
  tool db_query { effect read; approval true }
  
  memory user_ctx {
    scope "user:{user_id}"
    ttl 3600
    confidence 80
  }
  
  on query(q: string) -> string {
    let ctx = recall("user:{user_id}:prefs")
    let results = call web_search(q)
    let ranked = reason("rank by relevance", results)
    emit ranked
    write("user:{user_id}:last_query", q)
    return ranked
  }
}

workflow orchestrate {
  on request(user_id: int) {
    let profile = delegate(user_agent, "get_profile", user_id)
    let recs = delegate(recommend_agent, "recommend", profile.genre)
    emit { profile, recs }
  }
}
```

### 3.2 Compact Syntax (backward-compatible)

```axl
3;50|search_agent|web_search,db_read;40|query|q:s|->s;20|ctx|@user:prefs|s;12|!web_search/1,$q;12|~reason/2,"rank",results;99;99
```

## 4. Primitive Agent-Native

### 4.1 Reasoning

```axl
// Chain-of-thought
let answer = reason("step by step", problem)

// Classification
let category = classify("news or opinion", text, ["news", "opinion"])

// Extraction
let entities = extract("person, org, location", text)

// Summarization
let summary = summarize(text, max_tokens: 200)
```

### 4.2 Generation

```axl
// Text generation
let response = generate("You are a helpful assistant", messages)

// Structured output
let data = generate_json("Extract as JSON", text, schema)

// Streaming
stream("prompt") | chunk => emit chunk
```

### 4.3 Memory Semantic

```axl
// Write con semantica
write("user:{id}:prefs", prefs, confidence: 90, ttl: 3600)

// Recall semantico
let relevant = recall("what does user like", scope: "user:{id}")

// Search semantico
let similar = search_similar("similar to this content", embedding)

// Forget selettivo
forget("user:{id}:old_data", reason: "outdated")
```

### 4.4 Inter-Agent Communication

```axl
// Direct messaging
send(other_agent, "task", data)

// Broadcast
broadcast("event", data)

// Delegation (sync)
let result = delegate(other_agent, "method", args...)

// Delegation (async)
defer(other_agent, "method", args...)

// Agent discovery
let agents = find_agents(capability: "search")
```

### 4.5 Event-Driven

```axl
// Event handlers
on message(msg) { ... }
on schedule("*/5 * * * *") { ... }
on tool_result(tool, result) { ... }
on memory_change(key, old, new) { ... }
```

## 5. Tool System

### 5.1 Tool Definition

```axl
tool web_search {
  effect read
  approval false
  timeout 30s
  retries 3
  
  params {
    query: string
    max_results: int = 10
  }
  
  returns string
}
```

### 5.2 Tool Discovery

```axl
// Dynamic discovery
let tools = find_tools(effect: "read", capability: "search")

// Runtime creation
let custom_tool = create_tool("my_tool", |args| {
  // custom logic
  return result
})
```

## 6. Type System

Tipi base: `int`, `string`, `bool`, `float`
Tipi composti: `list<T>`, `map<K,V>`, `tuple<T...>`
Tipi speciali: `embedding`, `stream<T>`, `promise<T>`, `agent_ref`

```axl
let embedding: embedding = embed("text to vector")
let stream: stream<string> = generate_stream("prompt")
let agent: agent_ref = find_agent("search_agent")
```

## 7. Memory Model

### 7.1 Scopes

```axl
scope "global"          // shared across all agents
scope "user:{id}"       // per-user
scope "session:{id}"    // per-session
scope "agent:{name}"    // per-agent
scope "task:{id}"       // per-task
```

### 7.2 Semantic Memory

```axl
// Rich memory record
memory record {
  key: string
  value: any
  embedding: embedding  // auto-computed
  confidence: float     // 0.0 - 1.0
  ttl: duration
  source: string        // provenance
  tags: list<string>    // for filtering
}
```

## 8. Observability

### 8.1 Built-in Tracing

```axl
// Automatic trace
trace("agent started", { agent: "search", query: q })

// Metrics
metric("tool_calls", 1)
metric("reasoning_time", duration)
```

### 8.2 Debugging

```axl
// Breakpoints (dev mode)
breakpoint("check state")

// State inspection
inspect(agent_state)
inspect(memory_state)
inspect(tool_calls)
```

## 9. Security

### 9.1 Tool Permissions

```axl
// Declarative
tool dangerous_tool {
  effect write
  approval true
  require_capability "admin"
}

// Runtime
if not has_capability("admin") {
  deny("insufficient permissions")
}
```

### 9.2 Sandboxing

```axl
// Agent sandbox
sandbox agent {
  max_memory 1GB
  max_tools 10
  max_duration 30m
  network false
}
```

## 10. Execution Model

### 10.1 Single Agent

```axl
program main {
  agent searcher { ... }
  run searcher
}
```

### 10.2 Multi-Agent Orchestration

```axl
program netflix {
  agent catalog { ... }
  agent search { ... }
  agent user { ... }
  agent recommend { ... }
  
  workflow main {
    on request {
      let profile = delegate(user, "get_profile", uid)
      let results = delegate(search, "search", query, profile)
      let recs = delegate(recommend, "recommend", profile)
      emit { results, recs }
    }
  }
  
  run main
}
```
