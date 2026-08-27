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

api JwtSecuredCashflowApi
  auth bearer: HttpAuth = CashflowDemoJwt
  post /jwt/balance MovementBatch -> money = CalculateLedgerBalance
```

The selected skill can be replaced by any provider satisfying `HttpAuth`,
including the built-in static bearer and HS256 JWT adapters.

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

Each request middleware runs before auth. Providers may transform the envelope or
reject the request. The built-in `axl::middleware::header_gate` checks one
configured header and is replaceable.

Rate limiting uses the same middleware declaration with an open `RateLimit`
capacity. The HTTP adapter calls `allow` with a `method path` key:

```axl
capacity RateLimit
  op allow text -> Result<bool> idempotent

skill MemoryRateLimit provides RateLimit
  native rust axl::middleware::rate_limit
  config limit: int = 5
  config window_ms: int = 60000

api LimitedCashflowApi
  middleware request: RateLimit = MemoryRateLimit
  post /limited/balance MovementBatch -> money = CalculateLedgerBalance
```

Exhaustion maps to HTTP 429 (`rate_limit_exceeded`). The memory skill is a
process-local fixture; any compatible provider can replace it.

CORS reuses request and response middleware with one replaceable native skill.
Response middleware merges `Access-Control-Allow-Origin`,
`Access-Control-Allow-Methods` and `Access-Control-Allow-Headers` from config.
Request middleware may reject a mismatched `Origin` when `origin` is not `*`.
When an API binds `axl::middleware::cors`, `OPTIONS` for a matching path returns
204 with those headers and skips the route flow:

```axl
skill CashflowCorsOrigin provides HttpMiddleware
  native rust axl::middleware::cors
  config origin: text = "*"

skill CashflowCorsHeaders provides HttpResponseMiddleware
  native rust axl::middleware::cors
  config origin: text = "*"
  config methods: text = "GET,POST,OPTIONS"
  config headers: text = "content-type,authorization"

api CorsCashflowApi
  middleware request: HttpMiddleware = CashflowCorsOrigin
  middleware response: HttpResponseMiddleware = CashflowCorsHeaders
  post /cors/balance MovementBatch -> money = CalculateLedgerBalance
```

Response middleware uses the same declaration form with phase `response` and a
typed response envelope:

```axl
entity HttpResponse
  status: int required
  headers: Map<text,text> required
  body: text required

capacity HttpResponseMiddleware
  op process HttpResponse -> Result<HttpResponse> idempotent

api AnnotatedCashflowApi
  middleware response: HttpResponseMiddleware = CashflowResponseHeaders
  post /annotated/balance MovementBatch -> money = CalculateLedgerBalance
```

Response middleware runs after the flow. The built-in
`axl::middleware::response_headers` skill merges one configured header into the
envelope; providers remain replaceable. `HttpResult` carries the merged headers
to Axum.

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

Typed text caching is an open capacity with replaceable skills:

```axl
entity CacheEntry
  key: text required
  value: text required

capacity Cache
  op get text -> Result<text> idempotent
  op put CacheEntry -> Result<unit>
  op invalidate text -> Result<bool>

skill MemoryCache provides Cache
  native rust axl::cache::memory

skill DurableCache provides Cache
  native rust axl::cache::sqlite
  config path: text = "./build/cashflow-cache.db"
```

`get` returns `cache_miss` on absence; `invalidate` reports whether a key was
removed. Memory and durable SQLite adapters share the contract. The cashflow
demo caches a ledger balance key through ordinary capacity calls.

Structured logs, counters and spans are open capacities with replaceable memory
skills:

```axl
capacity Logger
  op write text -> Result<unit>
  op list unit -> Result<List<text>>

skill MemoryLogger provides Logger
  native rust axl::telemetry::logger

capacity Metrics
  op increment text -> Result<int>
  op get text -> Result<int> idempotent

skill MemoryMetrics provides Metrics
  native rust axl::telemetry::metrics

capacity Tracer
  op start text -> Result<text>
  op finish text -> Result<unit>
  op list unit -> Result<List<text>>

skill MemoryTracer provides Tracer
  native rust axl::telemetry::tracer
```

Cashflow proves two `write` lines via `list`, a counter of `2` after two
`increment`s, and one finished span name. HTTP routes under `/observability/*`
share the process-local provider runtime. Production exporters stay behind the
same ports.

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
| CORS origin rejected | 403 |
| rate-limit exceeded | 429 |
| invalid JSON or runtime input | 400 |
| unknown method/path | 404 |

## Current boundary

Scalar bindings and composite entity assembly cover body, path, query, header
and cookie sources. Ordered request and response middleware with typed envelopes
are executable. Typed events and durable/scheduled jobs with replaceable JobStore
providers are executable. Nested target paths beyond a single field name are
not yet request sources. Memory and unconfigured SQLite remain process-local;
configured SQLite paths are durable for records and jobs. Transactions and
migrations remain data gates. The built-in static bearer, HS256 JWT, header-gate,
response-headers, memory rate-limit and CORS providers are demo fixtures and their
config is visible in the manifest. JWT validates HMAC-SHA256 `sub`/`iss` claims
through `rust::axl::auth::jwt`. True secret references (no plaintext in IR) are
Gate 8; OAuth remains later. Cache and observability (Logger, Metrics, Tracer) are
executable through replaceable skills. Rate-limit is executable as capacity-backed
request middleware. CORS is executable as capacity-backed request/response
middleware with OPTIONS preflight.
