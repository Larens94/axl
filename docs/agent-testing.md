# Agent testing handoff

This guide lets another agent verify the implemented AXL Open Block Protocol
without relying on conversation history.

## 1. Verify the repository

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
jq empty schema/axl-ir-4.0.schema.json
jq empty schema/axl-open-block-2.schema.json
jq empty schema/axl-flow-2.schema.json
```

The integration suite compiles nine valid documented programs, round-trips each
through Packed Graph IR and verifies nine intentionally invalid programs.

The foundation program `examples/catalog/software-foundation.axl` contains
fourteen primary open blueprint contracts and must compile as application
`SoftwareFoundation`.

## 2. Compile the complete open block

```sh
cargo run -p axl-compiler -- \
  check examples/blocks/05-open-dataview.axl --json
```

The command must return `ok: true` for application `OpenDataViewBlock`.

## 3. Inspect the open surfaces

```sh
cargo run -p axl-compiler -- \
  blocks examples/blocks/05-open-dataview.axl

cargo run -p axl-compiler -- \
  experiment examples/blocks/05-open-dataview.axl /tmp/axl-open-block-test

jq '.protocol, .blocks[0].open_surface_count, .blocks[0].surfaces' \
  /tmp/axl-open-block-test/targets/blocks/open-blocks.json
```

Expected protocol: `axl-open-block/2`. The `CustomerDataView` block currently
contains twelve typed surfaces, nine of which are direct customization surfaces.
The other three are observable `state`, `event` and `error` surfaces.

## 4. Verify a typed instance override

```sh
cargo run -p axl-compiler -- \
  check examples/blocks/06-instance-override.axl --json

cargo run -p axl-compiler -- \
  blocks examples/blocks/06-instance-override.axl \
  | jq '.instances[0]'
```

The resolved manifest entry must name blueprint `CustomerList`, contain two
settings (`page_size`, `density`) and bind `table.row` to
`CompactCustomerRow`.

## 5. Execute the cashflow core

```sh
cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateMovement \
  examples/apps/inputs/movement-invalid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateBalance \
  examples/apps/inputs/balance.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl BuildMovementView \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateLedgerBalance \
  examples/apps/inputs/movement-batch.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl StoreAndLoadMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl StoreAndLoadMovementSqlite \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateAndStoreMovement \
  examples/apps/inputs/movement-valid.json
```

The first results must respectively contain an `ok` movement, the error
`amount_must_be_positive`, the integer `80000`, a view with direction `Entrata`
and a folded ledger balance of `80000`. The storage evaluations must return
movement `movement-001`; the composed flow must validate before saving. No
application-specific Rust function contains these rules.

## 6. Verify invalid programs

These commands are expected to fail:

```sh
cargo run -p axl-compiler -- \
  check examples/invalid/closed-blueprint.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/wrong-parameter.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/instance-overrides.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-types.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-calls.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-records.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-folds.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-runs.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-matches.axl --json
```

The first diagnostic set must include `AXL-O401`; the second must include
`AXL-V403`; the third must include `AXL-I605`, `AXL-I607` and `AXL-P405`; the
fourth must include `AXL-X803` and `AXL-X806`; the fifth must include every code
from `AXL-X816` through `AXL-X821`; the sixth must include every code from
`AXL-X831` through `AXL-X835`; the seventh must include `AXL-N805` and
`AXL-X841` through `AXL-X843`; the eighth must include `AXL-X851` through
`AXL-X856`; the ninth must include `AXL-X861` through `AXL-X865`.

## 7. Verify canonical formatting and transport

```sh
cargo run -p axl-compiler -- \
  fmt examples/blocks/05-open-dataview.axl

cargo run -p axl-compiler -- \
  pack examples/blocks/05-open-dataview.axl --matrix
```

`cargo test --workspace` performs the stronger check: decoding the matrix form
must reconstruct exactly the same canonical Semantic Graph IR.

## What this proves

- the new surfaces are parsed, type checked and lowered to Graph IR;
- compatible providers are checked for input, action, policy, slot and hook;
- closed blueprints and invalid scalar defaults are rejected;
- the open surface is machine-discoverable through a generated manifest;
- instance settings and provider overrides survive the Packed IR round-trip;
- enums and ordered flow statements survive the Packed IR round-trip;
- typed flows validate input and evaluate expressions at runtime;
- capacity dependencies, calls and provider bindings are statically checked;
- conditional expressions and multiline records are typed and executable;
- folds and composed flow runs survive formatting and Packed IR round-trips;
- enum matches are exhaustive and executable;
- memory and SQLite providers execute through the same replaceable ABI;
- documentation examples remain coupled to compiler tests.

It does not prove durable persistence, HTTP or runtime UI rendering. SQLite is
currently an in-memory connection scoped to one runtime evaluation. Generated
target files are not yet a deployable app.
