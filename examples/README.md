# AXL Examples — Esempi Complessi

## 1. API Server (`api-server/`)

Server REST completo con:
- CRUD per utenti (GET, POST, PUT, DELETE)
- Autenticazione JWT con SHA-256
- Rate limiting (100 req/min per IP)
- Cache in-memory con TTL
- Logging strutturato
- Frontend AX-UI

```bash
cargo run -p ai-platform -- 8080 examples/api-server/api-server.axl
```

### API Endpoints

```bash
GET  /api/health        # Health check
GET  /api/users         # List all users
GET  /api/users/1       # Get user by ID
POST /api/users         # Create user
PUT  /api/users/1       # Update user
DELETE /api/users/1     # Delete user
```

### Primitiva usate

- `json_parse`, `json_stringify` — parsing JSON
- `hash_sha256` — autenticazione JWT
- `time_now` — timestamp per cache TTL
- `text_split` — parsing CSV
- `file_read`, `file_write` — I/O file
- `list_sort`, `list_filter` — manipolazione dati
- `map_get`, `map_set` — gestione stato

---

## 2. Data Pipeline (`data-pipeline/`)

Pipeline ETL completa con:
- Lettura file CSV/JSON
- Parsing e validazione
- Transformazione (filter, map, aggregate)
- Cache risultati
- Export JSON

```bash
cargo run -p ai-platform -- 8081 examples/data-pipeline/pipeline.axl
```

### Operazioni

- **Extract**: legge file, parsa CSV/JSON
- **Transform**: filtra, mappa, aggrega
- **Validate**: controlla schema, qualità dati
- **Load**: esporta in JSON, CSV, database

### Primitiva usate

- `file_read`, `file_write` — I/O
- `json_parse`, `json_stringify` — JSON
- `text_split`, `text_join` — CSV
- `list_filter`, `list_map`, `list_sort` — trasformazione
- `math_sum`, `math_average` — aggregazione
- `map_get`, `map_set` — cache

---

## 3. Multi-Agent System (`multi-agent/`)

Sistema multi-agente con:
- Agenti specializzati (research, analysis, summary)
- Workflow collaborativo
- Memoria semantica
- Integrazione LLM

```bash
cargo run -p ai-platform -- 8082 examples/multi-agent/agents.axl
```

### Agenti

- **Research Agent**: cerca e raccoglie informazioni
- **Analysis Agent**: analizza e classifica dati
- **Summary Agent**: crea riepiloghi concisi
- **Memory Agent**: memorizza e recupera conoscenza

### Workflow

1. Input query → 2. Research → 3. Analyze → 4. Summarize → 5. Output

### Primitiva usate

- `reason`, `classify`, `extract` — LLM reasoning
- `embed`, `similarity` — memoria semantica
- `json_parse`, `json_stringify` — serializzazione
- `list_push`, `list_filter` — manipolazione

---

## Architettura

Tutti gli esempi usano:

```text
AXL Source (.axl)
→ parse_compact()
→ validate()
→ build_web() → HTML/CSS/JS
→ AxlServer → HTTP API
→ primitives → Rust nativo
```

### Stack

- **Frontend**: AX-UI (componenti HTML generati)
- **Backend**: AXL Server con route handlers
- **Primitiva**: 90+ primitive native Rust
- **Data**: In-memory + cache
- **Security**: Rate limiting, auth
