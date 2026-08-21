# Architettura

## Stack AXL 3.0

```text
AXL Compact Source 3
→ parser deterministico
→ AX-IR 2.0 (agent-centric)
→ type-check
→ interpreter
  ├─ primitive native (90+)
  ├─ LLM backend (reason, classify, extract)
  ├─ tool system (policy, approval, audit)
  ├─ memory (in-memory, SQLite, semantic)
  └─ bridge (native, WASM, VM)
```

## Rust Workspace

```text
runtime/
├── axl-core-rs/          # Libreria principale
│   ├── ir.rs             # Tipi IR
│   ├── compact.rs        # Parser compatto
│   ├── interpreter.rs    # Esecuzione
│   ├── primitives/       # 90+ primitiva native
│   │   ├── io.rs         # File I/O
│   │   ├── text.rs       # Elaborazione testo
│   │   ├── collections.rs # List/Map/Set
│   │   ├── math.rs       # Operazioni matematiche
│   │   ├── crypto.rs     # Hash, encoding, random
│   │   ├── system.rs     # Env, time, path, process
│   │   ├── serialize.rs  # JSON parsing
│   │   └── net.rs        # HTTP client
│   ├── llm.rs            # LLM backend trait
│   ├── memory.rs         # Memory stores
│   ├── policy.rs         # Tool policy
│   ├── validation.rs     # Validazione programma
│   ├── typechecker.rs    # Type-checking
│   ├── serialization.rs  # JSON IR
│   └── render_web.rs     # Renderer HTML
├── axl-cli/              # CLI binario
├── netflix-server/       # Netflix demo
└── ai-platform/          # AI Platform demo
```

## Pipeline di Esecuzione

1. **Parser** — converte sorgente compact in AST
2. **Validation** — verifica struttura programma
3. **Type-check** — verifica tipi
4. **Interpreter** — esegue con:
   - Primitive native (chiamate dirette Rust)
   - Tool utente (handler custom)
   - Memory store (scoped, TTL)
   - Policy system (approval, audit)
   - Budget limits (steps, bytes, depth)

## Primitive vs Tools

| Aspetto | Primitive Native | Tool Utente |
|---|---|---|
| Implementazione | Rust statically linked | Closure dinamica |
| Performance | Ottimale | Overhead chiamata |
| Sicurezza | Safe Rust | Sandbox necessario |
| Numero | 90+ | Illimitato |
| Esempio | `!file_read/1` | `!search_catalog/1` |
