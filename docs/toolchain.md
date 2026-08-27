# AXL 4 toolchain

## Pipeline

```text
readable AXL
  -> typed AST
  -> Semantic Graph IR
  -> Packed Graph IR
  -> Rust / React / SQL / agent contracts
```

Readable AXL is the authoring form. Semantic Graph IR is explicit and suitable
for analysis. Packed Graph IR is a deterministic opcode representation intended
for transport; it is compared with JSON Graph IR, not claimed to beat source
text in every case.

## Commands

From the repository root:

```sh
cargo run -p axl-compiler -- check examples/next/crm.axl --json
cargo run -p axl-compiler -- ir examples/next/crm.axl
cargo run -p axl-compiler -- pack examples/next/crm.axl --matrix
cargo run -p axl-compiler -- fmt examples/next/crm.axl
cargo run -p axl-compiler -- experiment examples/next/crm.axl build/axl4
cargo run -p axl-compiler -- unpack build/axl4/app.packed.axl
```

`check` parses and semantically validates a program. `fmt` prints canonical
multiline source. `ir` and `pack` expose the two machine representations.
`unpack` reconstructs JSON Graph IR. `experiment` writes all representations
and target contracts.

## Experiment output

```text
build/axl4/
  app.axl
  app.axir.json
  app.packed.axl
  targets/
    manifest.json
    rust/axl_contracts.rs
    react/axl_slots.ts
    sql/schema.sql
    agents/agents.json
```

These target files are contracts and registries. They are not a deployable CRM.

## Verification

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The documented examples are included at compile time in
`runtime/axl-compiler/tests/documented_examples.rs`, preventing syntax examples
from drifting away from the compiler.
