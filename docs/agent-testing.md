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
jq empty schema/axl-http-1.schema.json
jq empty schema/axl-provider-1.schema.json
```

The integration suite compiles nine valid documented programs, round-trips each
through Packed Graph IR and verifies twenty-one intentionally invalid programs.

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

The first results must respectively contain an `ok` movement, the error
`amount_must_be_positive`, the integer `80000`, a view with direction `Entrata`
and a folded ledger balance of `80000`. The storage evaluations must return
movement `movement-001`; the composed flow must validate before saving. No
application-specific Rust function contains these rules. The final two commands
run in independent processes and must still find `movement-001`, proving that
the configured SQLite path survives a runtime restart.

## 6. Verify the HTTP backend

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

From another terminal:

```sh
curl -X POST http://127.0.0.1:8080/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response must be `80000`. Posting `movement-invalid.json` to `/movements`
must return HTTP 422 with `amount_must_be_positive`.

Verify state continuity while the server remains running:

```sh
curl -X POST http://127.0.0.1:8080/movements \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-valid.json

curl -X POST http://127.0.0.1:8080/movement-by-id \
  -H 'content-type: application/json' \
  --data-binary '"movement-001"'
```

Both responses must contain movement `movement-001`. The analogous `/durable`
routes use a configured SQLite file and remain readable after restarting the
server.

Verify the open bearer guard:

```sh
curl -X POST http://127.0.0.1:8080/secure/balance \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer axl-cashflow-demo' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response is `80000`. Omitting the authorization header returns 401; using a
different token returns 403. The token is deliberately a visible demo fixture,
not a production secret.

Verify the open request middleware gate:

```sh
curl -X POST http://127.0.0.1:8080/guarded/balance \
  -H 'content-type: application/json' \
  -H 'x-axl-client: cashflow-demo' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response is `80000`. Omitting the client header or using another value
returns 403.

After saving the durable movement, verify both request bindings:

```sh
curl http://127.0.0.1:8080/movements/movement-001
curl 'http://127.0.0.1:8080/movements/find?id=movement-001'
```

Both return the movement. The second URL proves that the exact `/movements/find`
route wins over the `/movements/{id}` template.

Verify composite request assembly:

```sh
curl -X POST \
  'http://127.0.0.1:8080/accounts/account-1/movement-preview?dry_run=true' \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-valid.json
```

The response contains the validated movement. The flow input was assembled as
`{ account, movement, dry_run }` from path, complete body and query.

## 7. Verify invalid programs

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

cargo run -p axl-compiler -- \
  check examples/invalid/flow-transforms.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-parallel.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-attempts.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-races.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-routes.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/provider-config-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/provider-configs.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-auth-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-auth.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-request-bindings.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-middleware-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-middleware.axl --json
```

The first diagnostic set must include `AXL-O401`; the second must include
`AXL-V403`; the third must include `AXL-I605`, `AXL-I607` and `AXL-P405`; the
fourth must include `AXL-X803` and `AXL-X806`; the fifth must include every code
from `AXL-X816` through `AXL-X821`; the sixth must include every code from
`AXL-X831` through `AXL-X835`; the seventh must include `AXL-N805` and
`AXL-X841` through `AXL-X843`; the eighth must include `AXL-X851` through
`AXL-X856`; the ninth must include `AXL-X861` through `AXL-X865`.
The tenth must include `AXL-X802`, `AXL-N806`, `AXL-X871` through `AXL-X879` and
`AXL-X881` through `AXL-X884`.
The eleventh must include every code from `AXL-X891` through `AXL-X895`.
The twelfth must include every code from `AXL-X901` through `AXL-X907`.
The thirteenth must include every code from `AXL-X911` through `AXL-X916`.
The fourteenth must include every code from `AXL-H901` through `AXL-H907`.
The fifteenth must include `AXL-P313` and `AXL-P314`. The sixteenth must include
`AXL-N303`, `AXL-N304` and `AXL-V305`.
The seventeenth must include every code from `AXL-P913` through `AXL-P917`. The eighteenth must
include every code from `AXL-H908` through `AXL-H912`.
The nineteenth must include every code from `AXL-H913` through `AXL-H917`.
The twentieth must include `AXL-P918`. The twenty-first must include every code
from `AXL-H918` through `AXL-H922`.

## 8. Verify canonical formatting and transport

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
- map/filter transforms are scoped, typed and executable;
- stable ascending/descending sort is typed and executable;
- grouping produces a checked `Map<K,List<T>>` without handwritten Rust;
- non-empty list literals infer a common type and retain multiline formatting;
- `parallel` uses concurrent provider forks and preserves source order;
- `attempt` enforces idempotency, bounded retry and real deadlines;
- `race` returns the first successful idempotent worker;
- HTTP routes dispatch through the generic Axum runtime;
- consecutive HTTP requests share one process-local provider runtime;
- memory and SQLite providers execute through the same replaceable ABI;
- typed provider config survives Graph/Packed IR and `axl-provider/1` generation;
- configured SQLite data survives destruction and recreation of the runtime;
- API auth is capacity-backed and proves missing, denied and accepted requests;
- ordered request middleware is capacity-backed over typed envelopes;
- scalar path/query bindings are checked, decoded and exact-route-safe;
- composite request entities are assembled from checked body/path/query nodes;
- documentation examples remain coupled to compiler tests.

It does not prove transaction/migration semantics or runtime UI rendering.
HTTP execution, process-local memory and restart-durable configured SQLite are
proven. Generated target files are not yet a deployable app.
