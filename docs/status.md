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
| Backend | scalar and composite body/path/query binding, Axum, typed bearer auth, ordered request middleware, typed events/subscriptions, capacity-backed jobs and durable SQLite | headers/cookies, secrets, JWT/OAuth and response middleware |
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
retry through memory or durable SQLite stores (`axl-compiler tick`). Header/cookie
binding, secret references, JWT/OAuth providers and response middleware are the
next backend milestones.
