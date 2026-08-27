# Cashflow executable core

`cashflow-core.axl` is the first AXL example that executes application behavior
instead of stopping at contracts. It implements seven deliberately narrow flows:

- `ValidateMovement` checks a typed movement kind and positive amount;
- `CalculateBalance` subtracts expenses from income using `money` arithmetic.
- `BuildMovementView` constructs a typed record with conditional and exhaustive match values;
- `CalculateLedgerBalance` folds a list of movements into one balance;
- `StoreAndLoadMovement` calls a generic in-memory store provider;
- `StoreAndLoadMovementSqlite` runs the same calls through SQLite.
- `ValidateAndStoreMovement` composes validation and storage flows.

Run from the repository root:

```sh
cargo run -p axl-compiler -- \
  check examples/apps/cashflow-core.axl --json

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

Expected results:

- the valid movement is returned inside `{ "ok": ... }`;
- the zero amount returns `{ "error": "amount_must_be_positive" }`;
- the balance input returns `80000`.
- the movement view returns direction `Entrata` and signed amount `125000`;
- the ledger fold returns `80000`;
- both replaceable storage providers return movement `movement-001`.
- the composed validation/storage flow returns the valid movement.

The implemented expression operators are `!`, unary `-`, `*`, `/`, `+`, `-`,
comparison operators, equality, `&&` and `||`, with normal precedence and
parentheses. It also supports lazy `if ... then ... else ...` expressions and
multiline `make name: Entity` record construction. Flow Runtime 2 adds typed `in` dependencies and
`call value = dependency.operation(argument)?` with `Result` propagation.
`fold` provides immutable collection aggregation and `run` composes flows.

This is not yet the complete cashflow application. SQLite currently lives only
for one `eval`; there is no durable database configuration, list aggregation,
event emission, state mutation or rendered UI.
Those missing capabilities must be added to AXL rather than implemented inside
this application with handwritten Rust or React.
