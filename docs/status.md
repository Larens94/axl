# AXL 4 implementation status

This table is the short source of truth for the current experiment.

| Area | Implemented now | Not implemented yet |
|---|---|---|
| Source | multiline AXL, typed list literals, transforms, flows, enum match, `parallel`, idempotent `attempt` and `race` | mutable statement blocks intentionally deferred |
| Types | built-ins, entities, enums, capacities, recursive generics | tuples and record operation parameters |
| Blocks | open protocol, typed instances/overrides and fourteen foundation contracts | package imports, cross-package overlays and registry |
| Contracts | `requires`, `ensures`, `invariant` stored in IR | expression type checking and execution |
| Safety | diagnostics, repair candidates, safety levels | automatic application of risky repairs |
| Policies | effects and capabilities validated and stored | runtime budgets and enforcement |
| Agents | belief/goal/plan graph model | planning and execution runtime |
| Runtime | records, transforms, `parallel`, `race`, retry/timeout, flow/capacity calls, typed `emit`/subscriptions, jobs (`enqueue`/`tick`), `Result` propagation, forkable configured provider ABI and HTTP | state and UI |
| Storage | generic memory and SQLite providers; typed durable SQLite paths | transactions, migrations, queries and other databases |
| Backend | scalar and composite body/path/query/header/cookie binding, Axum, typed bearer auth (static + HS256 JWT skills), ordered request and response middleware, capacity-backed rate-limit (`allow` → 429), capacity-backed CORS (`Access-Control-*` + OPTIONS preflight), typed events/subscriptions, capacity-backed jobs, Cache get/put/invalidate (memory + durable SQLite), Logger/Metrics/Tracer observability (memory) and durable SQLite | secret references (Gate 8) and OAuth |
| Targets | Rust/React/SQL contracts plus agent, block, flow, HTTP and provider manifests | executable full-stack application generation |
| IR | canonical JSON graph, packed opcode round-trip | stable compatibility guarantee |

## Evidence

- The compiler unit and integration tests validate parsing, semantics,
  diagnostics, IR determinism, packing and target adapters.
- Every sample in `examples/blocks`, the CRM graph and the executable cashflow
  core is compiled by
  `documented_examples.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the lint gate.

The project remains an experiment. AXL flows call replaceable capacities; the
same cashflow graph executes against memory and SQLite. A runtime test saves to
a configured SQLite file, destroys the runtime and reads through a new runtime.
Typed events reach multiple subscribers. Capacity-backed jobs enqueue, claim and
retry through memory or durable SQLite stores (`axl-compiler tick`). Header and
cookie request bindings share the same open bind model as path/query/body.
Cache put/get/invalidate works through memory and durable SQLite skills; a
configured cache path survives runtime recreate. Logger write/list, Metrics
increment/get and Tracer start/finish/list work through memory skills; cashflow
proves two structured lines and counters via eval and HTTP. Capacity-backed
rate-limit middleware proves N allowed requests then HTTP 429 on the cashflow
demo route. Capacity-backed CORS middleware proves `Access-Control-Allow-Origin`
on `/cors/balance` and OPTIONS preflight 204. Capacity-backed HS256 JWT auth
(`axl::auth::jwt`) proves 401/403/200 on `/jwt/balance` with demo `secret`/
`issuer` config. True secret references (no plaintext in IR) remain Gate 8;
OAuth remains open. Gate 2 auth-adapter slice is otherwise complete.
