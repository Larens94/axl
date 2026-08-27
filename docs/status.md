# AXL 4 implementation status

This table is the short source of truth for the current experiment.

| Area | Implemented now | Not implemented yet |
|---|---|---|
| Source | canonical multiline AXL, comments, declarations | general functions and control flow |
| Types | built-ins, entities, capacities, recursive generics | tuples and record operation parameters |
| Blocks | open protocol, typed instances/overrides and fourteen foundation contracts | package imports, cross-package overlays and registry |
| Contracts | `requires`, `ensures`, `invariant` stored in IR | expression type checking and execution |
| Safety | diagnostics, repair candidates, safety levels | automatic application of risky repairs |
| Policies | effects and capabilities validated and stored | runtime budgets and enforcement |
| Agents | belief/goal/plan graph model | planning and execution runtime |
| Targets | Rust contracts, React slot registry, SQL DDL, agent and open-block manifests | executable full-stack application generation |
| IR | canonical JSON graph, packed opcode round-trip | stable compatibility guarantee |

## Evidence

- The compiler unit and integration tests validate parsing, semantics,
  diagnostics, IR determinism, packing and target adapters.
- Every sample in `examples/blocks` and the CRM graph is compiled by
  `documented_examples.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the lint gate.

The project remains an experiment. The next meaningful milestone is executing
one narrow vertical slice: a generated Rust handler, SQL persistence and a React
component connected through the existing typed graph.
