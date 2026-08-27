# AXL

AXL is an experimental agent-native semantic blueprint language.

The current compiler implements one focused pipeline:

```text
readable AXL
  -> typed AST
  -> Semantic Graph IR
  -> Packed Graph IR
  -> Rust, React, SQL and agent contracts
```

AXL describes software through entities, capacities, skills and blueprints with
typed open ports. Rust and React are target implementations rather than the
source language.

## Try the experiment

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  check examples/next/crm.axl --json

~/.cargo/bin/cargo run -p axl-compiler -- \
  experiment examples/next/crm.axl build/axl4
```

The second command generates canonical AXL, JSON Graph IR, Packed IR and initial
target contracts.

## Project map

- `SPEC-4.0.md` — implemented language boundary.
- `examples/next/crm.axl` — semantic CRM experiment.
- `runtime/axl-compiler/src/next` — parser, analyzer, IR and target adapters.
- `schema/axl-ir-4.0.schema.json` — Graph IR JSON schema.

## Verification

```sh
~/.cargo/bin/cargo test --workspace
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Status: experiment. Executable full-stack Rust/React generation is the next gate.
