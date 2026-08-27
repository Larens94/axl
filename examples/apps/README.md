# Cashflow executable core

`cashflow-core.axl` is the first AXL example that executes application behavior
instead of stopping at contracts. It implements eight deliberately narrow flows:

- `ValidateMovement` checks a typed movement kind and positive amount;
- `CalculateBalance` subtracts expenses from income using `money` arithmetic.
- `BuildMovementView` constructs a typed record with conditional and exhaustive match values;
- `CalculateLedgerBalance` folds a list of movements into one balance;
- `StoreAndLoadMovement` calls a generic in-memory store provider;
- `StoreAndLoadMovementSqlite` runs the same calls through SQLite.
- `SaveDurableMovement` and `FindDurableMovement` reopen a configured SQLite file.
- `ValidateAndStoreMovement` composes validation and storage flows.
- `IncomeAmounts` filters and maps the movement collection.

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
  eval examples/apps/cashflow-core.axl IncomeAmounts \
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

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl SaveDurableMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-id.json
```

Expected results:

- the valid movement is returned inside `{ "ok": ... }`;
- the zero amount returns `{ "error": "amount_must_be_positive" }`;
- the balance input returns `80000`.
- the movement view returns direction `Entrata` and signed amount `125000`;
- the ledger fold returns `80000`;
- the filter/map pipeline returns `[125000]`;
- the typed sort returns `movement-002` before `movement-001`;
- grouping creates `consulting` and `software` buckets with typed movements;
- the multiline list literal creates the default category list;
- the parallel flow builds movement views concurrently in source order;
- resilient lookup uses bounded retry and timeout on an idempotent operation;
- raced lookup ignores failed candidates and returns the first successful record;
- both replaceable storage providers return movement `movement-001`.
- two independent durable evaluations return movement `movement-001` from the same file;
- the composed validation/storage flow returns the valid movement.

The implemented expression operators are `!`, unary `-`, `*`, `/`, `+`, `-`,
comparison operators, equality, `&&` and `||`, with normal precedence and
parentheses. It also supports lazy `if ... then ... else ...` expressions and
multiline `make name: Entity` record construction. Flow Runtime 2 adds typed `in` dependencies and
`call value = dependency.operation(argument)?` with `Result` propagation.
`fold` provides immutable collection aggregation and `run` composes flows.

The `CashflowApi` declaration exposes `/movements`, `/movement-by-id`,
`/balance`, `/income-amounts`, `/movements/sorted`, `/movements/grouped`,
`GET /categories`, `/movement-views`, `/movement-by-id/resilient` and
`/movement-first`, `/movements/durable` and `/movement-by-id/durable` through the generic Axum runtime:

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

The server shares one provider runtime across requests. Memory state is
process-local. The durable routes use the path declared on
`DurableSqliteMovements` and survive a server restart.

`POST /secure/balance` belongs to a separate guarded API. It requires
`Authorization: Bearer axl-cashflow-demo`; missing and invalid credentials return
401 and 403. The credential is intentionally visible test data until AXL gains
secret references and production auth adapters.

`POST /guarded/balance` uses ordered request middleware. It requires
`x-axl-client: cashflow-demo` and returns 403 when the header is missing or
wrong. The header-gate skill is a replaceable capacity, not route-specific Rust.

`POST /annotated/balance` uses ordered response middleware. It returns the same
balance body and sets `x-axl-middleware: ok` through the replaceable
`axl::middleware::response_headers` skill.

The durable lookup is also exposed as `GET /movements/{id}` and
`GET /movements/find?id=...`. These routes bind a path or query string directly
to the typed `uuid` flow input; exact `/movements/find` matching takes precedence
over the `{id}` template.

`POST /accounts/{account}/movement-preview?dry_run=true` assembles a
`MovementPreviewRequest`: `account` comes from the path, `movement` is the full
JSON body and `dry_run` comes from the query. No Rust request DTO or extractor is
specific to this route.

`GET /me` and `GET /session` bind `header.x-user` and `cookie.sid` into a typed
`text` flow. `POST /client-preview` assembles `ClientSessionRequest` from
`header.x-user`, `cookie.sid` and the JSON body through the same open bind model.

This is not yet the complete cashflow application. There are no transaction or
migration primitives, general store queries, state mutation or rendered UI.
Those missing capabilities must be added to AXL rather than implemented inside
this application with handwritten Rust or React.
