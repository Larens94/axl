# AXL 4 implementation status

This table is the short source of truth for the current experiment.

| Area | Implemented now | Not implemented yet |
|---|---|---|
| Source | multiline AXL, enum match, map/filter/sort/group/fold and flows with `make`, `run`, `call`, `return` | branch blocks, collection literals and async |
| Types | built-ins, entities, enums, capacities, recursive generics | tuples and record operation parameters |
| Blocks | open protocol, typed instances/overrides and fourteen foundation contracts | package imports, cross-package overlays and registry |
| Contracts | `requires`, `ensures`, `invariant` stored in IR | expression type checking and execution |
| Safety | diagnostics, repair candidates, safety levels | automatic application of risky repairs |
| Policies | effects and capabilities validated and stored | runtime budgets and enforcement |
| Agents | belief/goal/plan graph model | planning and execution runtime |
| Runtime | records, map/filter/sort/group/fold, flow/capacity calls, `Result` propagation, provider ABI and HTTP | state, events, durable persistence and UI |
| Storage | generic in-memory and SQLite `save/find/delete/list` providers | durable paths, transactions, migrations, queries and other databases |
| Backend | checked `api` routes, Graph IR dispatch, Axum JSON server and process-local shared provider state | durable runtime configuration, params/query, auth, middleware, events and jobs |
| Targets | Rust/React/SQL contracts plus agent, block, flow and HTTP manifests | executable full-stack application generation |
| IR | canonical JSON graph, packed opcode round-trip | stable compatibility guarantee |

## Evidence

- The compiler unit and integration tests validate parsing, semantics,
  diagnostics, IR determinism, packing and target adapters.
- Every sample in `examples/blocks`, the CRM graph and the executable cashflow
  core is compiled by
  `documented_examples.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the lint gate.

The project remains an experiment. AXL flows call replaceable capacities; the
same cashflow graph executes against memory and SQLite. HTTP requests share one
provider runtime for the server process and the test saves in one request and
reads in the next. Durable database configuration, auth and middleware are the
next backend milestones; collection literals and async providers remain
language and runtime gates.
