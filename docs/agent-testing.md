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
```

The integration suite compiles eight valid documented programs, round-trips each
through Packed Graph IR and verifies three intentionally invalid programs.

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

## 5. Verify invalid programs

These commands are expected to fail:

```sh
cargo run -p axl-compiler -- \
  check examples/invalid/closed-blueprint.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/wrong-parameter.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/instance-overrides.axl --json
```

The first diagnostic set must include `AXL-O401`; the second must include
`AXL-V403`; the third must include `AXL-I605`, `AXL-I607` and `AXL-P405`.

## 6. Verify canonical formatting and transport

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
- documentation examples remain coupled to compiler tests.

It does not prove runtime UI rendering or Rust execution. Generated target files
are still contracts, registries and manifests rather than a deployable app.
