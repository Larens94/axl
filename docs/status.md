# AXL 4 implementation status

This table is the short source of truth for the current experiment.

| Area | Implemented now | Not implemented yet |
|---|---|---|
| Source | multiline AXL, exhaustive enum match and flows with `let`, `make`, `fold`, `run`, `require`, `call`, `return` | branch blocks, collection transforms and async |
| Types | built-ins, entities, enums, capacities, recursive generics | tuples and record operation parameters |
| Blocks | open protocol, typed instances/overrides and fourteen foundation contracts | package imports, cross-package overlays and registry |
| Contracts | `requires`, `ensures`, `invariant` stored in IR | expression type checking and execution |
| Safety | diagnostics, repair candidates, safety levels | automatic application of risky repairs |
| Policies | effects and capabilities validated and stored | runtime budgets and enforcement |
| Agents | belief/goal/plan graph model | planning and execution runtime |
| Runtime | records, fold aggregation, flow/capacity calls, `Result` propagation and provider ABI | map/filter/group, state, events, durable persistence, HTTP and UI |
| Storage | generic in-memory and SQLite `save/find/delete/list` providers | durable paths, transactions, migrations, queries and other databases |
| Targets | Rust contracts, React slot registry, SQL DDL, agent, open-block and flow manifests | executable full-stack application generation |
| IR | canonical JSON graph, packed opcode round-trip | stable compatibility guarantee |

## Evidence

- The compiler unit and integration tests validate parsing, semantics,
  diagnostics, IR determinism, packing and target adapters.
- Every sample in `examples/blocks`, the CRM graph and the executable cashflow
  core is compiled by
  `documented_examples.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the lint gate.

The project remains an experiment. AXL flows now call replaceable capacities;
the same cashflow graph executes against memory and SQLite. The next milestone
is collection transforms plus async execution, followed by durable storage
configuration and an HTTP vertical slice.
