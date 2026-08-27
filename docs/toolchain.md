# AXL 4 toolchain

## Pipeline

```text
readable AXL
  -> typed AST
  -> Semantic Graph IR
  -> Packed Graph IR
  -> Flow runtime + replaceable providers + target contracts
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
cargo run -p axl-compiler -- blocks examples/next/crm.axl
cargo run -p axl-compiler -- experiment examples/next/crm.axl build/axl4
cargo run -p axl-compiler -- unpack build/axl4/app.packed.axl
cargo run -p axl-compiler -- eval examples/apps/cashflow-core.axl \
  CalculateBalance examples/apps/inputs/balance.json
```

`check` parses and semantically validates a program. `fmt` prints canonical
multiline source. `ir` and `pack` expose the two machine representations.
`blocks` prints the `axl-open-block/2` manifest without writing files. `unpack`
reconstructs JSON Graph IR. `experiment` writes all representations and target
contracts. `eval` validates JSON input and executes an AXL flow. Flow Runtime 2
can call a provider through the public ABI; built-in memory and SQLite store
providers are available, while arbitrary native symbol loading is not.

## Experiment output

```text
build/axl4/
  app.axl
  app.axir.json
  app.packed.axl
  targets/
    manifest.json
    blocks/open-blocks.json
    rust/axl_contracts.rs
    react/axl_slots.ts
    sql/schema.sql
    agents/agents.json
    flows/flows.json
```

`blocks/open-blocks.json` uses protocol identifier `axl-open-block/2` and exposes
the typed surface of every blueprint plus the settings and provider overrides
of every instance. These target files are contracts and registries. They are not
a deployable CRM.

Its JSON shape is documented by `schema/axl-open-block-2.schema.json`.
The flow manifest uses `axl-flow/2` and is documented by
`schema/axl-flow-2.schema.json`.

## Verification

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
jq empty schema/axl-ir-4.0.schema.json
jq empty schema/axl-open-block-2.schema.json
jq empty schema/axl-flow-2.schema.json
```

The documented examples are included at compile time in
`runtime/axl-compiler/tests/documented_examples.rs`, preventing syntax examples
from drifting away from the compiler.
