# AXL HTTP Runtime 1

AXL HTTP Runtime 1 exposes typed flows through a generic Axum server. An
application declares routes in AXL; it does not require application-specific
Rust controllers.

```axl
api CashflowApi
  post /movements Movement -> Result<Movement> = ValidateAndStoreMovement
  post /movement-by-id uuid -> Result<Movement> = FindMovement
  post /balance MovementBatch -> money = CalculateLedgerBalance
```

Auth is an open capacity attached to an API, not controller-specific Rust:

```axl
capacity HttpAuth
  op authorize text -> Result<bool> idempotent

api SecuredCashflowApi
  auth bearer: HttpAuth = CashflowDemoBearer
  post /secure/balance MovementBatch -> money = CalculateLedgerBalance
```

The selected skill can be replaced by any provider satisfying `HttpAuth`.

The body is the default flow input. Scalar and enum flows can bind a path or
query value without a handwritten extractor:

```axl
get /movements/{id} uuid -> Result<Movement> = FindDurableMovement from path.id
get /movements/find uuid -> Result<Movement> = FindDurableMovement from query.id
```

The compiler checks placeholder/name alignment and rejects record inputs for a
single scalar binding. Runtime extraction includes percent decoding and numeric
or boolean conversion. Exact paths are matched before template paths.

Composite entity inputs join multiple surfaces explicitly:

```axl
post /accounts/{account}/movement-preview MovementPreviewRequest -> Result<Movement> = PreviewMovement
  bind account = path.account
  bind movement = body
  bind dry_run = query.dry_run
```

Bindings are checked against entity fields, including duplicates and required
fields, and survive Graph/Packed IR as discoverable nodes. `body.field` extracts
one member; `body` assigns the complete JSON body to a nested field.

The compiler verifies:

- supported methods: `get`, `post`, `put`, `patch`, `delete`;
- absolute exact or whole-segment template paths;
- unique routes locally and across APIs;
- known input/output types;
- a declared target flow with exactly the same signature.

Routes become `api` and `route` nodes plus `dispatch` edges in Graph IR and
Packed IR. `targets/http/routes.json` uses protocol `axl-http/1`.

## Run

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080

curl -X POST http://127.0.0.1:8080/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

Expected response: `80000`.

The server owns one provider runtime. This makes state available to consecutive
requests while the process is running. For example, post
`movement-valid.json` to `/movements`, then post the JSON string
`"movement-001"` to `/movement-by-id`: the second response contains the saved
movement.

The cashflow demo also exposes `/movements/durable` and
`/movement-by-id/durable`. Their provider declares a typed SQLite `path` in AXL,
so data remains available after restarting the server.

Status mapping:

| Condition | Status |
|---|---:|
| successful flow value | 200 |
| AXL `Result` error | 422 |
| missing authorization | 401 |
| authorization denied/failed | 403 |
| invalid JSON or runtime input | 400 |
| unknown method/path | 404 |

## Current boundary

Scalar bindings and composite entity assembly are implemented. Header/cookie
fields and nested target paths are not yet request sources. Memory and
unconfigured SQLite remain process-local; configured SQLite paths are durable.
Transactions and migrations remain data gates. The built-in static bearer
provider is a demo fixture and its config is visible in the manifest; secret
references, JWT/OAuth validation, middleware chains, CORS, streaming, events,
jobs, cache, rate limits and observability remain later backend gates.
