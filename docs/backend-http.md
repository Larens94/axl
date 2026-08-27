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

Request middleware is an ordered open pipeline over a typed envelope:

```axl
entity HttpRequest
  method: text required
  path: text required
  headers: Map<text,text> required

capacity HttpMiddleware
  op process HttpRequest -> Result<HttpRequest> idempotent

api GuardedCashflowApi
  middleware request: HttpMiddleware = CashflowClientGate
  post /guarded/balance MovementBatch -> money = CalculateLedgerBalance
```

Each middleware runs before auth. Providers may transform the envelope or reject
the request. The built-in `axl::middleware::header_gate` checks one configured
header and is replaceable.

Typed application events are separate from HTTP:

```axl
event MovementSaved: Movement
on MovementSaved Movement = TagMovementPersisted
on MovementSaved Movement = TagMovementAnnounced

flow SaveAndAnnounce Movement -> Result<Movement>
  in store: MovementStore = MemoryMovements
  call saved = store.save(input)?
  emit MovementSaved(saved)
  return saved
```

`emit` fans out to every matching subscription in declaration order. A generic
`EventLog` capacity (`native rust axl::event::log`) lets listeners record side
effects without cashflow-specific Rust.

Durable and scheduled jobs are declarative and capacity-backed:

```axl
capacity JobStore
  op enqueue text -> Result<text> idempotent
  op claim unit -> Result<List<text>> idempotent
  op finish text -> Result<text>

job DurablePersistMovementJob
  run SaveDurableMovement
  retry 3
  idempotent
  in store: JobStore = DurableJobs

flow ScheduleDurableMovementPersist Movement -> Result<Movement>
  enqueue DurablePersistMovementJob(input)
  return input
```

`enqueue` persists an envelope through the bound JobStore. `axl-compiler tick`
claims due work, runs the bound flow with retry, and requeues schedules.
Memory (`axl::job::memory`) and SQLite (`axl::job::sqlite`) adapters keep the
provider replaceable; a configured SQLite path survives process restart.

The body is the default flow input. Scalar and enum flows can bind a path,
query, header or cookie value without a handwritten extractor:

```axl
get /movements/{id} uuid -> Result<Movement> = FindDurableMovement from path.id
get /movements/find uuid -> Result<Movement> = FindDurableMovement from query.id
get /me text -> text = EchoText from header.x-user
get /session text -> text = EchoText from cookie.sid
```

The compiler checks placeholder/name alignment and rejects record inputs for a
single scalar binding. Runtime extraction includes percent decoding, numeric
or boolean conversion, case-insensitive headers and simple `Cookie` parsing.
Exact paths are matched before template paths.

Composite entity inputs join multiple surfaces explicitly:

```axl
post /accounts/{account}/movement-preview MovementPreviewRequest -> Result<Movement> = PreviewMovement
  bind account = path.account
  bind movement = body
  bind dry_run = query.dry_run

post /client-preview ClientSessionRequest -> Result<Movement> = PreviewWithClientSession
  bind user = header.x-user
  bind sid = cookie.sid
  bind movement = body
```

Bindings are checked against entity fields, including duplicates and required
fields, and survive Graph/Packed IR as discoverable nodes. `body.field` extracts
one member; `body` assigns the complete JSON body to a nested field. Sources are
`body`, `path`, `query`, `header` and `cookie`.

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
| middleware rejection | 403 |
| invalid JSON or runtime input | 400 |
| unknown method/path | 404 |

## Current boundary

Scalar bindings and composite entity assembly cover body, path, query, header
and cookie sources. Ordered request middleware with typed envelopes is
executable. Typed events and durable/scheduled jobs with replaceable JobStore
providers are executable. Nested target paths beyond a single field name are
not yet request sources. Response-phase middleware and response header mutation
are not implemented. Memory and unconfigured SQLite remain process-local;
configured SQLite paths are durable for records and jobs. Transactions and
migrations remain data gates. The built-in static bearer and header-gate
providers are demo fixtures and their config is visible in the manifest; secret
references, JWT/OAuth validation, CORS, streaming, cache, rate limits and
observability remain later backend gates.
