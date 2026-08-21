# Roadmap

## Completato

### v3.0 — Agent-Native (attuale)
- [x] Parser compact (Source 2/3)
- [x] IR con primitive LLM definite
- [x] Validation + type-checking
- [x] Interpreter con tool policy, memory, budget
- [x] 90+ primitiva native Rust
- [x] LLM backend trait + MockBackend
- [x] Memory store (in-memory, SQLite)
- [x] CLI completo (run, compile, exec, pack, build, serve)
- [x] Serializzazione JSON IR 1.0/1.1/1.2
- [x] Renderer HTML/CSS/JS
- [x] 28 test passanti
- [x] Netflix demo applicazione
- [x] AI Content Platform demo
- [x] SPEC-3.0.md completa
- [x] PRIMITIVES.md (500+ primitiva taxonomy)

## In Lavorazione

### v3.1 — Runtime Completo
- [ ] Integrazione primitiva native nell'interpreter (in corso)
- [ ] Compiler con supporto moduli/import
- [ ] LLM backend reali (OpenAI, Anthropic)
- [ ] Streaming output LLM

### v3.2 — Agent Avanzati
- [ ] Comunicazione inter-agenti completa
- [ ] Event-driven execution
- [ ] Semantic memory con embedding
- [ ] Tool discovery dinamico
- [ ] Observability (trace, metric)

## Futuro

### v4.0 — Backend Multipli
- [ ] WASM target
- [ ] Native binary target
- [ ] VM interpreter
- [ ] C ABI bridge
- [ ] Mobile/Desktop bridge

### v4.1 — Ecosistema
- [ ] Package manager (AXL packages)
- [ ] LSP server
- [ ] Debugger
- [ ] Profiler
- [ ] Framework applicativi
