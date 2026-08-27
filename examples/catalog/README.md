# Software foundation catalog

`software-foundation.axl` is the first compiler-verified catalog built on the
AXL Open Block Protocol. It contains fourteen blueprint contracts:

- repository, query, command and transaction;
- API, events and jobs;
- page, DataView, form and navigation;
- observability, agent tool and scenario testing.

Every blueprint exposes at least one typed customization surface. Validate and
inspect the catalog from the repository root:

```sh
cargo run -p axl-compiler -- \
  check examples/catalog/software-foundation.axl --json

cargo run -p axl-compiler -- \
  blocks examples/catalog/software-foundation.axl

cargo run -p axl-compiler -- \
  experiment examples/catalog/software-foundation.axl /tmp/axl-foundation

jq '.blocks | length' \
  /tmp/axl-foundation/targets/blocks/open-blocks.json
```

The last command must return `14`.

The native skill symbols are contracts for future target adapters. This catalog
does not claim that their Rust, React or AI bodies exist or that the fourteen
blocks execute at runtime today.
