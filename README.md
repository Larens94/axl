# AXL

AXL is an experimental agent-native semantic blueprint language.

The current compiler implements one focused pipeline:

```text
readable AXL
  -> typed AST
  -> Semantic Graph IR
  -> Packed Graph IR
  -> Flow runtime + replaceable providers + target contracts
```

AXL describes software through entities, capacities, skills and blueprints with
typed open surfaces. The compiler rejects closed blueprints. Rust and React are
target implementations rather than the source language.

## Try the experiment

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  check examples/next/crm.axl --json

~/.cargo/bin/cargo run -p axl-compiler -- \
  experiment examples/next/crm.axl build/axl4
```

The second command generates canonical AXL, JSON Graph IR, Packed IR and initial
target contracts.

Execute behavior written in AXL:

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateBalance \
  examples/apps/inputs/balance.json
```

This returns `80000`. The balance rule is an AXL flow, not an
application-specific Rust function.

Serve routes declared in AXL through the generic Axum runtime:

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

## Project map

- `SPEC-4.0.md` — implemented language boundary.
- `docs/index.html` — concise browser documentation.
- `docs/blocks.md` — verified guide to open block construction.
- `docs/agent-testing.md` — repeatable test handoff for another agent.
- `docs/executable-flows.md` — executable Flow Runtime 2 semantics.
- `docs/runtime-abi.md` — replaceable native provider contract.
- `docs/backend-http.md` — executable Axum route adapter and current boundary.
- `docs/roadmap.md` — executable autoloop gates and current position.
- `presentation.html` — simplified, responsive project presentation.
- `examples/blocks` — small examples compiled by the test suite.
- `examples/catalog/software-foundation.axl` — fourteen open foundation blocks.
- `examples/apps/cashflow-core.axl` — executable validation and balance flows.
- `examples/next/crm.axl` — semantic CRM experiment.
- `runtime/axl-compiler/src/next` — parser, analyzer, IR and target adapters.
- `schema/axl-ir-4.0.schema.json` — Graph IR JSON schema.
- `schema/axl-open-block-2.schema.json` — block and instance manifest schema.
- `schema/axl-flow-2.schema.json` — executable flow and capacity-call manifest schema.
- `schema/axl-http-1.schema.json` — checked HTTP route manifest schema.

## Build an open block

```axl
capacity CustomerRow
  op render Customer -> UI

skill DefaultCustomerRow provides CustomerRow
  native react crm::DefaultCustomerRow

blueprint CustomerList
  slot table.row: CustomerRow = DefaultCustomerRow
```

This exact example is compiled from `examples/blocks/02-ui-slot.axl`. See the
[block guide](docs/blocks.md) for backend ports, hooks and agent declarations.

The complete protocol example is `examples/blocks/05-open-dataview.axl`. It
uses typed parameters, state, events, actions, errors, policies, slots and hooks
without modifying generated target files.

`examples/blocks/06-instance-override.axl` derives a configured instance with
`set` and `use`; the original blueprint and generated Rust/React remain untouched.

The foundation catalog adds fourteen compiler-verified contracts covering data,
commands, API, UI, events, jobs, observability, agent tools and scenarios.

## Verification

```sh
~/.cargo/bin/cargo test --workspace
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Status: experiment. Pure typed flows execute; persistence, HTTP and React
runtime generation are not implemented yet.
