# Roadmap

AXL evolve per vertical slice dimostrabili. Ogni milestone richiede sorgente AXL,
artefatti eseguibili, test e limiti dichiarati.

## Completato — `0.1.0-alpha.1`

- [x] runtime e CLI Rust;
- [x] Compact Source 2, RPN e formatter multilinea;
- [x] Compact UI Source 3 con component registry;
- [x] compilatore di entità, API, auth, seed e query;
- [x] backend Rust/Axum/SeaORM e migrazioni SQLite;
- [x] frontend React/Refine/MUI/TanStack/Lucide;
- [x] CRM con 6 entità, 30 CRUD e 7 viste;
- [x] navigazione responsive, bottom menu e table-to-card;
- [x] documentazione bilingue e presentazione pubblicabile.

## M1 — Stabilizzare il compilatore applicativo

- manifest applicativo e contratti di target versionati;
- diagnostica con span e codici errore stabili;
- relazioni, enum, option/result e validazione form;
- OpenAPI generata e suite E2E browser/API;
- benchmark riproducibile di dimensione, token e tempo di generazione.

**Gate:** il CRM si rigenera da zero e passa test Rust, API ed E2E su CI.

## M2 — Backend production-oriented

- PostgreSQL, transazioni e pool;
- autenticazione, RBAC, secret management e rate limit;
- SSE/WebSocket, upload bounded e job async;
- logging, metriche e tracing OpenTelemetry;
- container, health/readiness e graceful shutdown.

**Gate:** servizio distribuibile con threat model e osservabilità verificati.

## M3 — AX-UI oltre il web

- stato, eventi, form, accessibilità e design token semantici;
- renderer DOM stabile e test visuali;
- desktop WebView come bootstrap;
- adapter nativi SwiftUI e Jetpack Compose;
- renderer canvas/WebGPU per grafica specialistica.

**Gate:** lo stesso semantic UI tree passa la suite di conformità su due renderer.

## M4 — Runtime agentico

- scheduler async e concorrenza strutturata;
- capability ABI versionata, cancellazione e deadline;
- backend modello reali, routing ed eval;
- memoria semantica e provider esterni;
- audit e sandbox degli handler non fidati.

**Gate:** workflow multi-agente riproducibile con policy e trace end-to-end.

## M5 — Ecosistema

- AX-HIR/AX-MIR, VM e target WASM/native;
- package manager, registry, lockfile e firma;
- LSP, formatter, debugger, profiler e documentazione API;
- SDK e bridge per TypeScript, Python, C, IoT e servizi esterni.

**Gate:** package riproducibile e stessa suite semantica su runtime multipli.
