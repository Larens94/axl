# AXL 4.0 Experiment — Semantic Blueprint Language

Status: executable experiment. This document distinguishes implemented behavior
from the planned language so that agents do not infer features that do not exist.

## 1. Purpose

AXL describes a typed software graph. Rust, React, SQL, AI and IoT are target
implementations below the graph; they are not the source of truth.

AXL 4 is designed around three representations:

```text
readable AXL -> typed AST -> Semantic Graph IR -> Packed Graph IR
```

- Readable AXL is canonical, multiline source for humans and agents.
- Semantic Graph IR makes every component and connection explicit.
- Packed Graph IR is a deterministic opcode transport format.

The packed form is intended to be smaller than JSON Graph IR. It is not expected
to be smaller than every readable AXL program because Graph IR contains semantic
information that is implicit in source.

## 2. Implemented declarations

### Application

```axl
axl 4
app SalesCRM
```

Both declarations are required. The compiler rejects other language versions.

### File import

Multi-file composition merges declarations from other AXL modules into one
compilation unit. Imports are resolved relative to the importing source file.

```axl
axl 4
app ImportDemo
import "../modules/math-lib.axl"
```

Rules:

- import path must be a quoted relative path (`"./lib.axl"` or `"../modules/lib.axl"`);
- imported modules are ordinary AXL files with their own `axl`/`app` headers;
- imported declarations merge before local declarations in import order;
- duplicate declaration names across merged modules are rejected (`AXL-N002`);
- **diamond imports** (A→B, A→C, B→D, C→D) merge D once; true cycles still report `AXL-P932`;
- missing paths report `AXL-P931`; circular imports report `AXL-P932`;
- `compile_source` without a base file rejects programs that contain imports
  (`AXL-P933`); use `compile_file` or `compile_source_at`.

Imported modules do not yet carry package names, semantic versions, overlays or
registry metadata. Those remain Gate 8.

### Entity

```axl
entity Customer
  id: uuid key readonly
  email: email required unique
```

Implemented field qualifiers:

```text
key required optional unique index private readonly
```

### Enum

```axl
enum MovementKind
  income
  expense
```

Enum variants are closed, unique typed values. In expressions they are
referenced as `MovementKind.income`.

### Capacity

A capacity describes what can be done without selecting an implementation.

```axl
capacity CustomerStore
  op find uuid -> Result<Customer>
  op save Customer -> Result<Customer>
```

The experiment accepts one input type and one output type per operation. Tuple
and record parameters are planned.

### Skill

A skill implements a capacity.

```axl
skill SqliteCustomers provides CustomerStore
  native rust axl::store::sqlite
  config path: text = "./data/crm.db"
  effect db.read
  effect db.write
  capability crm.customer.write
```

Implemented native targets:

```text
rust react sql ai iot wasm
```

AXL validates the declared target and capacity. A skill may expose typed scalar
`config` values. Config names, types and values are checked, lowered as owned
Graph IR nodes and delivered to the provider ABI; applications do not hardcode
provider paths in Rust. Target ABI validation is planned.

### Blueprint

```axl
blueprint CustomerCRM
  param page_size: int = 25
  param density: text = "comfortable"
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text

  in store: CustomerStore
  out api: CrudApi<Customer>

  action refresh: CustomerStore = SqliteCustomers
  policy access: AuthProvider = JwtAuth
  slot table.row: CustomerRow = DefaultCustomerRow
  hook before.save: CustomerPolicy = ValidateCustomer

  use store = SqliteCustomers

  requires auth.authorized
  ensures customer.persisted
  invariant Customer.email unique

  effect db.write
  capability crm.customer.manage
```

Blueprint connection points:

| Construct | Meaning | Must be connected |
|---|---|---|
| `in` | required consumed capacity | yes, unless it has a default |
| `out` | capacity exposed by the blueprint | no |
| `slot` | replaceable UI or structural component | no |
| `hook` | replaceable lifecycle behavior | no |
| `param` | typed scalar configuration with a required default | n/a |
| `state` | typed observable state surface | no |
| `event` | typed event payload exposed by the block | no |
| `action` | replaceable invokable capacity | no |
| `error` | typed failure surface exposed by the block | no |
| `policy` | replaceable policy capacity | no |
| `use` | explicit provider binding | n/a |

`in`, `slot`, `hook`, `action` and `policy` accept providers. Providers are type
checked: a skill satisfies one of these surfaces only when its `provides`
capacity equals the declared type. A blueprint output may be referenced as
`Blueprint.output`.

Parameters currently support checked scalar defaults: `bool`, `int`, `float`,
`money` and JSON-quoted `text`, `string`, `email`, `uuid`, `datetime` or
`duration`. `state`, `event` and `error` are typed Graph IR surfaces and do not
accept defaults or `use` bindings.

Every blueprint must expose at least one customization surface among `in`,
`slot`, `hook`, `param`, `action` and `policy`. A closed blueprint produces
diagnostic `AXL-O401`. This rule is the first executable version of the AXL Open
Block Protocol.

### Blueprint instance

An instance customizes an existing blueprint without modifying its definition
or generated target code:

```axl
instance CompactCustomerList of CustomerList
  set page_size = 10
  set density = "compact"
  use table.row = CompactCustomerRow
```

`set` accepts only a declared `param` and validates the value against that
parameter's scalar type. `use` accepts only `in`, `slot`, `hook`, `action` or
`policy`, and the provider must satisfy the same capacity as the base surface.
Duplicate settings and overrides are rejected.

Instances become `instance`, `setting` and `override` nodes. An `instantiates`
edge links the instance to its blueprint, while provider overrides use normal
typed `bind` edges. Packed IR round-trips these nodes and edges without losing
the customization.

### Agent

```axl
agent SalesAssistant
  believe crm.pipeline
  goal qualify_lead
  plan automatic_qualification
  plan request_human_review
  effect ai.generate
  capability crm.lead.read
```

An agent requires at least one goal and one plan. Beliefs, goals and plans become
explicit graph nodes. Execution semantics are planned.

### Flow

A flow is the first executable AXL behavior. It has one typed input, one typed
output and an immutable, ordered body:

```axl
flow CalculateBalance BalanceInput -> money
  let balance = input.income - input.expense
  return balance
```

`Result<T>` flows can stop with a typed validation error:

```axl
flow ValidateMovement Movement -> Result<Movement>
  let positive = input.amount > 0
  require positive else "amount_must_be_positive"
  return input
```

A flow can expose an open capacity dependency and invoke its bound provider:

```axl
flow StoreAndLoadMovement Movement -> Result<Movement>
  in store: MovementStore = MemoryMovements
  call saved = store.save(input)?
  call loaded = store.find(saved.id)?
  return loaded
```

`in` declares the required capacity. A compatible default provider may follow
`=`, or `use store = Provider` can bind it separately. `call` is checked against
the selected capacity operation. `?` is required for `Result<T>` operations and
propagates provider failure through a `Result<T>` flow.

AXL constructs typed records without a single long expression:

```axl
make view: MovementView
  id = input.id
  direction = if input.kind == MovementKind.income then "Entrata" else "Uscita"
  signed_amount = if input.kind == MovementKind.income then input.amount else -input.amount
```

The compiler rejects unknown, duplicate, missing required and incorrectly typed
fields. Implemented statements are `let`, `make`, `require ... else`, `call`
and `return`. Values are immutable. Expressions support paths, enum variants,
`if ... then ... else ...`, parentheses, `!`, unary `-`, `*`, `/`, `+`, `-`,
comparisons, equality, `&&` and `||`, with normal operator precedence. The
compiler checks every expression, call argument and return type.

The public Rust `ProviderRuntime` ABI receives provider, capacity,
implementation, typed configuration, operation and JSON input. The built-in experiment implements
generic `save`, `find`, `delete` and `list` operations for
`rust::axl::store::memory`, `rust::axl::store::sqlite`,
`rust::axl::store::postgres` and
`rust::axl::store::mysql` and
`rust::axl::store::document`. They are not tied to
`Movement` or to the cashflow application. SQLite uses an in-memory connection
when no path is configured and opens a durable file when the skill declares a
`path` config. Document skills use the same path model: process-local JSON when
unconfigured, a durable JSON object file when `path` is set. PostgreSQL skills
require `config url` (typically `secret("AXL_POSTGRES_URL")`); connections are
pooled per URL inside a runtime and independent runtimes share the same records.
Independent runtimes configured with the same file observe the same records.

Flows compose other flows with the same explicit `Result` propagation:

```axl
run valid = ValidateMovement(input)?
```

`fold` is the first collection loop. It keeps the accumulator immutable and
types both the initial and next value:

```axl
fold balance: money = input.movements from 0 as movement
  next = if movement.kind == MovementKind.income
    then value + movement.amount
    else value - movement.amount
```

Inside `next`, `value` is the current accumulator and the name after `as` is the
current collection item. The source must be `List<T>` or `Set<T>`.

Enum decisions can be exhaustive:

```axl
match signed: money = input.kind
  income => input.amount
  expense => -input.amount
```

`match` requires an enum subject and exactly one compatible case for every
variant. Missing, duplicate and unknown variants are compiler errors.

Collection transforms are typed and multiline:

```axl
filter incomes: List<Movement> = input.movements as movement
  where = movement.kind == MovementKind.income

map amounts: List<money> = incomes as movement
  value = movement.amount
```

`filter` preserves the exact source collection type and requires a boolean
predicate. `map` requires a `List<T>` or `Set<T>` output and checks every mapped
value against `T`. A mapped `Set<T>` removes duplicate values deterministically.

`sort` converts a `List<T>` or `Set<T>` source into an ordered `List<T>`:

```axl
sort newest: List<Movement> = input.movements as movement
  by = movement.occurred_at
  direction = desc
```

The key must be a number, string-like scalar or enum. Direction is exactly
`asc` or `desc`; equal keys keep their original order deterministically.

`group` creates a typed map whose values preserve source order:

```axl
group by_category: Map<text,List<Movement>> = input.movements as movement
  by = movement.category
```

Group keys are string-like scalars or enums because the runtime representation
is a JSON object. The declared `Map<K,List<T>>` key and item types must match the
key expression and source collection.

Non-empty list literals infer one compatible item type and format vertically:

```axl
let categories = [
  "consulting",
  "software"
]
```

Nested expressions are allowed. Numeric items use the normal numeric promotion
rules; mixed incompatible items and untyped empty lists are compiler errors.

`parallel` applies one typed flow concurrently while preserving source order:

```axl
parallel views: List<MovementView> = input.movements as movement
  run = BuildMovementView(movement)
```

The source is `List<T>` or `Set<T>`, the argument must match the target flow
input and the declared result is `List<Output>`. A `Result<T>` target requires
`?` and propagates the first error in source order. Every worker receives a
provider runtime fork; the built-in forks share synchronized provider state.

Capacity operations may declare a machine-checked resilience contract:

```axl
capacity MovementStore
  op find uuid -> Result<Movement> idempotent

attempt found = store.find(input)?
  retry = 2
  timeout_ms = 250
```

`attempt` runs each invocation behind a real deadline and retries provider
errors or timeouts up to the declared count. It is rejected unless the operation
is `idempotent`, preventing invisible duplicate writes. Retry is limited to ten
and timeout to 1–60000 milliseconds.

`race` returns the first successful result from concurrent candidates:

```axl
race found: Movement = input.ids as candidate
  run = FindMovement(candidate)?
```

The target flow, argument and output are checked like `parallel`. The compiler
recursively proves that every capacity operation reachable from the target is
`idempotent`, because losing workers may still finish. Result errors are ignored
while another candidate can succeed; if all fail, the first source-order error
is propagated deterministically.

### HTTP API

An API exposes checked flows without handwritten controllers:

```axl
api CashflowApi
  post /movements Movement -> Result<Movement> = ValidateAndStoreMovement
  post /movement-by-id uuid -> Result<Movement> = FindMovement
  post /balance MovementBatch -> money = CalculateLedgerBalance
```

An API can protect all of its routes through an open typed auth capacity:

```axl
capacity HttpAuth
  op authorize text -> Result<bool> idempotent

skill DemoBearer provides HttpAuth
  native rust axl::auth::bearer
  config token: text = "demo-only"

skill DemoJwt provides HttpAuth
  native rust axl::auth::jwt
  config secret: text = "demo-only"
  config issuer: text = "axl-demo"

api SecuredApi
  auth bearer: HttpAuth = DemoBearer
  post /secure/balance MovementBatch -> money = CalculateLedgerBalance

api JwtSecuredApi
  auth bearer: HttpAuth = DemoJwt
  post /jwt/balance MovementBatch -> money = CalculateLedgerBalance
```

The compiler requires the exact idempotent `authorize` contract and a compatible
provider. The Axum adapter maps a missing bearer header to 401 and denial to 403.
A replaceable HS256 JWT skill (`native rust axl::auth::jwt`) validates bearer
tokens against typed `secret` and `issuer` config and requires `sub`/`iss`
claims. Skills may bind secrets with Gate 8 references that never enter Graph or
manifest plaintext:

```axl
skill AuthDemoJwtIssuer provides JwtIssuer
  native rust axl::auth::jwt_sign
  config secret: text = secret("AXL_AUTH_JWT")
  config issuer: text = "axl-auth"
```

The IR stores `secret_ref` metadata with a null value; `provider_config` resolves
the environment variable at invoke time. Provider manifests redact the value.
Demo plaintext skill config remains allowed for fixtures that have not migrated.
OAuth demo provider `rust::axl::auth::oauth` implements `authorize_url` and
`exchange` on the `OAuthClient` capacity (`examples/apps/oauth-boundary.axl`).
Demo authorization codes use the `axl-demo-{16hex}` shape; `client_id` /
`client_secret` resolve from `secret("ENV")` at invoke time. HTTP redirect and
callback routes remain open.

Routes may also declare **per-route guards** that call AXL flows (no application
logic in Rust). Guards run after API middleware and before bearer auth:

```axl
post /clienti Cliente -> Result<Cliente> = CreaCliente
  guard session RequireSession from cookie.sid
  guard can RequireSessionPermesso "vendite.clienti.read" from cookie.sid

post /auth/login LoginInput -> Result<LoginResult> = LoginUtente
  guard guest RequireSession from cookie.sid
```

Kinds:

- `session` — bind a scalar from cookie/header/query/path, evaluate the flow; failure → 401
- `can` — bind session id + permission string into `{session_id, permesso}`, evaluate the flow; failure → 403
- `guest` — if the bound session flow succeeds, reject with 403 `already_authenticated`

An API can also attach an ordered open request middleware pipeline. Each entry is
a capacity over a typed request envelope:

```axl
entity HttpRequest
  method: text required
  path: text required
  headers: Map<text,text> required

capacity HttpMiddleware
  op process HttpRequest -> Result<HttpRequest> idempotent

skill DemoClientGate provides HttpMiddleware
  native rust axl::middleware::header_gate
  config header: text = "x-axl-client"
  config value: text = "demo"

api GuardedApi
  middleware request: HttpMiddleware = DemoClientGate
  post /guarded/balance MovementBatch -> money = CalculateLedgerBalance
```

Middleware runs in declaration order before auth. The built-in header gate is a
replaceable fixture; rejection maps to HTTP 403.

Request middleware may also bind a replaceable rate-limit capacity. The adapter
calls `allow` with a stable `method path` key and maps exhaustion to HTTP 429:

```axl
capacity RateLimit
  op allow text -> Result<bool> idempotent

skill MemoryRateLimit provides RateLimit
  native rust axl::middleware::rate_limit
  config limit: int = 5
  config window_ms: int = 60000

api LimitedApi
  middleware request: RateLimit = MemoryRateLimit
  post /limited/balance MovementBatch -> money = CalculateLedgerBalance
```

The memory adapter is a process-local fixture. Providers remain replaceable;
Redis or other backends can satisfy the same `allow` contract without changing
routes.

An API can also attach ordered response-phase middleware over a typed response
envelope. Providers may set or mutate response headers after the flow runs:

```axl
entity HttpResponse
  status: int required
  headers: Map<text,text> required
  body: text required

capacity HttpResponseMiddleware
  op process HttpResponse -> Result<HttpResponse> idempotent

skill DemoResponseHeaders provides HttpResponseMiddleware
  native rust axl::middleware::response_headers
  config header: text = "x-axl-middleware"
  config value: text = "ok"

api AnnotatedApi
  middleware response: HttpResponseMiddleware = DemoResponseHeaders
  post /annotated/balance MovementBatch -> money = CalculateLedgerBalance
```

Response middleware runs after the flow result is produced. The built-in
`response_headers` skill merges one configured header; any compatible provider
can replace it. The response body travels as JSON text inside the envelope.

CORS reuses the same request and response middleware phases behind a replaceable
native skill. Response middleware merges `Access-Control-*` headers from config.
Request middleware may reject a mismatched `Origin` when `origin` is not `*`.
When an API binds `axl::middleware::cors`, `OPTIONS` preflight for a matching
path returns 204 with those headers and does not run the route flow:

```axl
skill DemoCorsOrigin provides HttpMiddleware
  native rust axl::middleware::cors
  config origin: text = "*"

skill DemoCorsHeaders provides HttpResponseMiddleware
  native rust axl::middleware::cors
  config origin: text = "*"
  config methods: text = "GET,POST,OPTIONS"
  config headers: text = "content-type,authorization"

api CorsApi
  middleware request: HttpMiddleware = DemoCorsOrigin
  middleware response: HttpResponseMiddleware = DemoCorsHeaders
  post /cors/balance MovementBatch -> money = CalculateLedgerBalance
```

### Events

Applications declare typed top-level events, subscribe any number of flows, and
emit payloads from flow statements:

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

Event names are simple identifiers without dots. Each `on` line names the event,
repeats the payload type and binds a subscriber flow whose input must match.
`emit Event(expression)` evaluates the payload, then calls every matching
subscription in declaration order through the shared provider runtime. Subscriber
outputs are discarded; a `{ "error": ... }` Result or runtime failure becomes an
error for the emitting flow. Graph IR stores app-owned `event` nodes,
`subscription` nodes (opcode 50) and `emit` statements (opcode 51).

### Jobs

Jobs are open, capacity-backed work units bound to flows:

```axl
capacity JobStore
  op enqueue text -> Result<text> idempotent
  op claim unit -> Result<List<text>> idempotent
  op finish text -> Result<text>

skill MemoryJobs provides JobStore
  native rust axl::job::memory

skill DurableJobs provides JobStore
  native rust axl::job::sqlite
  config path: text = "./build/cashflow-jobs.db"

job PersistMovementJob
  run ValidateAndStoreMovement
  retry 3
  idempotent
  in store: JobStore = MemoryJobs

job DurablePersistMovementJob
  run SaveDurableMovement
  retry 3
  idempotent
  in store: JobStore = DurableJobs

job MovementTickJob
  schedule "every 60s"
  run RecordJobTick
  retry 1
  idempotent
  in store: JobStore = MemoryJobs

flow ScheduleMovementPersist Movement -> Result<Movement>
  enqueue PersistMovementJob(input)
  return input
```

A job requires `run`, `retry` (0..10), `idempotent` when retry > 0, and
`in store: JobStore = Provider`. Optional `schedule "every <n>ms|s|m"` marks a
unit-input tick job. `enqueue Job(expr)` stores a JSON envelope through the
bound store; `axl-compiler tick` (and `run_due_jobs`) ensures scheduled
registrations, claims due work, executes the bound flow with retry/backoff, and
requeues the next schedule after success. Memory and SQLite adapters share the
same `JobStore` contract; a configured SQLite path survives runtime restart.
Graph IR uses `job` (opcode 52) and `enqueue` (opcode 53).

### Cache

Applications cache typed text values through an open `Cache` capacity:

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

flow CacheBalanceSnapshot CacheEntry -> Result<text>
  in cache: Cache = MemoryCache
  call stored = cache.put(input)?
  call loaded = cache.get(input.key)?
  return loaded
```

`get` is idempotent and returns `cache_miss` when the key is absent.
`invalidate` returns whether a key was removed. Memory and SQLite adapters share
the same contract; a configured SQLite path survives runtime recreate. No new
Graph IR opcodes are required: cache uses ordinary capacity calls.

### Observability

Applications record structured logs, counters and spans through open capacities:

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

flow RecordTwoObservabilityLines unit -> Result<List<text>>
  in logger: Logger = MemoryLogger
  call first = logger.write("ledger.balance")?
  call second = logger.write("ledger.balance")?
  call lines = logger.list(input)?
  return lines
```

`write` appends a structured text line; `list` returns recorded lines for
assertion. `increment`/`get` share a named counter (missing keys read as `0`).
`start` returns a span id; `finish` records the span name; `list` returns
finished span names. Memory adapters store events in-process for tests and
shared HTTP runtimes. No new Graph IR opcodes: observability uses ordinary
capacity calls. Production exporters remain replaceable skills behind the same
ports.

### Transactions

Applications coordinate multi-write durability through an open
`TransactionManager` capacity. Store skills keep their existing `save`/`find`
contract; SQLite transactions join when the transaction skill and store skill
share the same configured path. Memory transactions snapshot provider state and
support nested begin/commit/rollback.

```axl
capacity TransactionManager
  op begin text -> Result<text>
  op commit text -> Result<unit>
  op rollback text -> Result<unit>

skill DurableSqliteTransactions provides TransactionManager
  native rust axl::tx::sqlite
  config path: text = "./build/cashflow-core.db"
  effect db.write

flow CommitTwoDurableMovements MovementPair -> Result<Movement>
  in tx: TransactionManager = DurableSqliteTransactions
  in store: MovementStore = DurableSqliteMovements
  call tid = tx.begin("commit-two")?
  call first = store.save(input.first)?
  call second = store.save(input.second)?
  call done = tx.commit(tid)?
  return second
```

`begin` returns the transaction id (the label argument). `commit` makes writes
visible to a new runtime on the same SQLite path. `rollback` discards writes so
subsequent `find` calls return `not_found`. Skills bound to
`rust::axl::tx::memory` or `rust::axl::tx::sqlite` must expose the begin/commit/
rollback contract (`AXL-D901`). No new Graph IR opcodes: transactions use
ordinary capacity calls. A dedicated `transaction { ... }` statement block is
not required for this slice.

### Migrations and schema history

Applications advance and inspect schema versions through an open
`MigrationRunner` capacity. Skills record ordered version history; SQLite skills
also create and drop version marker tables so schema change is durable.

```axl
capacity MigrationRunner
  op up text -> Result<text>
  op down text -> Result<text>
  op status unit -> Result<text>

skill DurableSqliteMigrations provides MigrationRunner
  native rust axl::migrate::sqlite
  config path: text = "./build/cashflow-core.db"
  effect db.write

flow ApplyDurableMigration text -> Result<text>
  in migrations: MigrationRunner = DurableSqliteMigrations
  call version = migrations.up(input)?
  return version
```

`up` applies a version id and returns it. `down` rolls back only the current
head version. `status` returns the head version or `"0"` when history is empty.
Skills bound to `rust::axl::migrate::memory` or `rust::axl::migrate::sqlite`
must expose the up/down/status contract (`AXL-D902`). No new Graph IR opcodes:
migrations use ordinary capacity calls. Declared SQL migration scripts,
PostgreSQL/MySQL providers and document tx/migrate remain later Gate 3 work.

### Typed repository queries

Store capacities may declare an idempotent `query` operation over a single
typed request entity and a typed page entity. Because AXL operations take one
input type and one output type (record/tuple parameters remain deferred), filter,
order and page live on one `QuerySpec`-shaped entity:

```axl
entity MovementQuery
  filter: Map<text,text> optional
  order_by: text optional
  direction: text optional
  limit: int optional
  offset: int optional

entity MovementPage
  items: List<Movement> required
  total: int required
  limit: int required
  offset: int required

capacity MovementStore
  op save Movement -> Result<Movement>
  op find uuid -> Result<Movement> idempotent
  op query MovementQuery -> Result<MovementPage> idempotent

flow QueryDurableMovements MovementQuery -> Result<MovementPage>
  in store: MovementStore = DurableSqliteMovements
  call page = store.query(input)?
  return page
```

Runtime `rust::axl::store::memory`, `rust::axl::store::sqlite`,
`rust::axl::store::postgres` and
`rust::axl::store::document` interpret conventional fields: `filter` is equality
on stored JSON fields (map values are text; numbers and booleans coerce),
`order_by`/`direction` sort stably, `offset`/`limit` page after filtering.
`total` is the filtered count before paging. SQLite, PostgreSQL and MySQL
adapters push `filter`/`order_by`/`limit`/`offset` into SQL over JSON payload
fields (safe field names only; invalid names return `invalid_query_field`).
Memory and document stores still evaluate queries in-process. Skills that declare `query` must
use an idempotent entity → `Result<PageEntity>` contract (`AXL-D903`). No new
Graph IR opcodes. Document skills persist a JSON object file when `config path`
is set (same path model as SQLite). SQL pushdown for store `query` is executable
on SQLite, PostgreSQL and MySQL. PostgreSQL store (`rust::axl::store::postgres`),
transactions (`rust::axl::tx::postgres`) and migrations
(`rust::axl::migrate::postgres`) are executable with the same save/find/query and
begin/commit/rollback and up/down/status contracts; see
`examples/apps/postgres-boundary.axl`.

Route inputs use the JSON body by default. A scalar or enum input can instead
come directly from a named path, query, header or cookie value:

```axl
get /movements/{id} uuid -> Result<Movement> = FindMovement from path.id
get /movements/find uuid -> Result<Movement> = FindMovement from query.id
get /me text -> text = EchoText from header.x-user
get /session text -> text = EchoText from cookie.sid
```

Path placeholders and binding names are checked. Runtime decoding converts
`bool`, `int`, `float` and `money`, percent-decodes string-like path/query
values, reads headers case-insensitively and parses `Cookie` as
`name=value` pairs separated by `;`. The normal flow validator checks the
bound result. Exact paths win over templates.

An entity input can be assembled from several request surfaces:

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

Each `bind` is a first-class `request_binding` Graph/Packed IR node. Targets must
be unique declared fields; every required field must be bound. `body.field`
selects one JSON member, while plain `body` assigns the complete JSON value to a
nested entity field. Missing optional fields are omitted. Request sources are
`body`, `path`, `query`, `header` and `cookie`.

Implemented methods are `get`, `post`, `put`, `patch` and `delete`. Paths are
absolute and may contain named whole-segment placeholders. Input/output types must exactly match the bound
flow signature. Duplicate routes inside one API and conflicts across APIs are
rejected.

`axl-compiler serve` runs a generic Axum adapter over the HTTP nodes in Graph
IR. JSON input is validated by the flow runtime. A successful value returns
HTTP 200, an AXL `{ "error": ... }` returns 422, invalid input returns 400 and
an unknown route returns 404. One built-in provider runtime is shared by all
requests for the lifetime of the server process. Memory and unconfigured SQLite
state survive consecutive requests only; a configured SQLite file also survives
server and CLI restarts.

### UI pages and forms

UI declarations compile to `axl-ui/1` and lower to `ui` / `page` / `form` / `ui_action` /
`ui_drawer` and `ui_modal` nodes in Graph IR. The HTML renderer evaluates bound flows. The React target emits open codegen artifacts from the
same manifest — `axl_routes.tsx` (React Router table + layout assignment), `axl_layouts.tsx`
(Guest/App/Admin slots), and `axl_registry.ts` (component registry). Hosts bind concrete React
components to those slots; product routes and forms are never authored by hand in React.
The built-in HTML renderer emits a dashboard shell (`theme: dashboard-apple` in the UI manifest):
sidebar navigation grouped by section on desktop, fixed **mobile bottom navigation**
(`shell.mobile = bottom-nav`) from declared page routes, top bar, cards, stat blocks, and
responsive tables/forms styled with system typography and light/dark support. Application
layout stays in the open renderer; domain logic remains in AXL flows. A page binds an absolute path to a flow with an exact input/output signature. Pages may
use path templates with `{param}` placeholders, reusing the same `from path.name` binding
model as HTTP routes:

```axl
ui BalanceScreen
  page /balance BalanceInput -> money = CalculateBalance

ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
```

A drawer is a detail overlay bound like a templated page, optionally anchored to a list
page with `on`:

```axl
ui ClienteDrawerUi
  page /clienti unit -> Result<ClientePage> = ListaClienti
  drawer /clienti/{id} uuid -> Result<Cliente> = DettaglioCliente from path.id on /clienti
```

`render` and `serve` GET emit a side panel (`role="dialog"`) with a close link to `on`.
See `examples/apps/drawer-boundary.axl`.

A modal is a centered overlay bound like a drawer:

```axl
ui ClienteModalUi
  page /clienti unit -> Result<ClientePage> = ListaClienti
  modal /clienti/{id}/confirm uuid -> Result<Cliente> = ConfermaCliente from path.id on /clienti
```

`render` and `serve` GET emit a centered dialog (`modal-panel`, `role="dialog"`) with a
close link to `on`. See `examples/apps/modal-boundary.axl`.

A form binds an absolute path to an entity type and flow. The optional `submit` clause
names the POST api route that receives the entity JSON; when omitted, the analyzer
infers a POST route at the same path as the form. The optional `redirect` clause
names the list page returned after a successful browser form POST; when omitted, the
runtime derives it from the parent path of the form (for example `/clienti/new` →
`/clienti`):

```axl
ui ClienteScreen
  form /clienti/new Cliente -> Result<Cliente> = CreaCliente submit /clienti redirect /clienti
```

An action binds a label path to a POST api route and optional redirect page. Actions render
as inline POST forms on the page named by `redirect` (or on pages that declare them).
Redirect paths may use the same `{param}` templates; after a successful POST the runtime
substitutes placeholders from the response `ok` object (for example `ok.id`). The action
`submit` path may also use `{param}` templates when the redirect page binds the same
parameter from `path` (for example `from path.id`); the renderer resolves placeholders
from the current page path and evaluated page data, emits a concrete `action` URL, and
includes hidden inputs for each template parameter (for example `id` from `ok.id`):

```axl
ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
  action /preventivi/invia POST /preventivi/{id}/invia redirect /preventivi/{id}
```

On `serve`, POST to a templated API route binds `path.id` (and other path parameters) from
the request URL; when the body is `application/x-www-form-urlencoded`, a form field whose
name matches the template parameter takes precedence over the URL segment.

List pages render `id` fields with `uuid` type as links to a detail page template such as
`/preventivi/{id}` (substituting the row `id` value) when that templated page exists in
the same UI manifest. The legacy `{list_path}/detail` convention is still recognized.

`axl-compiler ui` emits the manifest. `axl-compiler render` evaluates the bound
flow with JSON input and emits HTML that displays typed scalar or entity fields
from the eval result. `render_form` emits an HTML form with inputs derived from
entity fields (text, money, int, enum as select, optional fields) and a navigation
shell linking all `page` and `form` paths. `serve` returns `text/html` on GET for
matching page and form paths. Exact UI paths (without `{param}` placeholders) are always
served as HTML on GET. Templated UI pages are served when the client prefers HTML
(`Accept: text/html`). When the same path is also served by an HTTP API route, JSON
clients keep the API response unless they request HTML. After a successful POST to a form's `submit` route
with `application/x-www-form-urlencoded` (or `Accept: text/html`), `serve` returns
HTTP 303 with `Location` set to the form's `redirect` path or the inferred parent
list page. Application logic stays in AXL; the renderer only reads
Graph IR and manifest metadata. Duplicate paths inside one `ui` block and
conflicts across `ui` blocks are rejected.

Implemented UI diagnostics:

| Code | Meaning |
|---|---|
| `AXL-P950` | unknown line inside `ui` |
| `AXL-P951` | page missing flow binding |
| `AXL-P952` | page missing output type |
| `AXL-P953` | page missing path and input type |
| `AXL-P954` | page binding needs a source and name |
| `AXL-P960` | form missing flow binding |
| `AXL-P961` | form missing output type |
| `AXL-P962` | form missing path and entity type |
| `AXL-P963` | form submit path must be absolute |
| `AXL-P964` | form redirect path must be absolute |
| `AXL-P970` | action missing path, method or submit route |
| `AXL-P971` | action label path must be absolute |
| `AXL-P972` | action submit path must be absolute |
| `AXL-P973` | action method must be POST |
| `AXL-P974` | action redirect path must be absolute |
| `AXL-P980` | drawer missing flow binding / path / output / binding |
| `AXL-P981` | drawer on path must be absolute |
| `AXL-P990` | modal missing flow binding / path / output / binding |
| `AXL-P991` | modal on path must be absolute |
| `AXL-U901` | `ui` requires at least one page, form, action, drawer or modal |
| `AXL-U902` | invalid page, form, drawer or modal path |
| `AXL-U903` | duplicate page path in one `ui` |
| `AXL-U904` | unknown or non-flow page/form/drawer/modal target |
| `AXL-U905` | page, form, drawer or modal signature does not match flow |
| `AXL-U906` | page or form path conflicts across `ui` blocks |
| `AXL-U907` | duplicate form path in one `ui` |
| `AXL-U908` | unknown submit route for form |
| `AXL-U910` | duplicate action path in one `ui` |
| `AXL-U911` | unknown submit route for action |
| `AXL-U912` | action method must be POST |
| `AXL-U913` | invalid UI page binding name |
| `AXL-U914` | page path binding has no matching placeholder |
| `AXL-U915` | page path binding cannot construct input type |
| `AXL-U920` | duplicate drawer path in one `ui` |
| `AXL-U921` | duplicate modal path in one `ui` |

Packed IR opcodes: `ui` = `54`, `page` = `55`, `form` = `56`, `ui_action` = `57`,
`route_guard` = `58`, `ui_drawer` = `59`, `ui_filter` = `60`, `ui_pagination` = `61`,
`ui_modal` = `62`.

## 3. Type system

Implemented built-ins:

```text
unit bool int float text string email uuid datetime money bytes duration
Result Option List Set Map Stream Future UI CrudApi
```

Declared entities, enums and capacities are also valid types. Generic references are
checked recursively, so `Result<Unknown>` is rejected.

## 4. Ports and repair diagnostics

An unconnected required input produces `AXL-P401` and a repair plan containing
all known compatible providers.

```json
{
  "code": "AXL-P401",
  "phase": "ports",
  "expected": "provider of CustomerStore",
  "found": "unconnected",
  "fix_safety": "risky",
  "repairs": [
    {
      "kind": "connect",
      "target": "CustomerCRM.store",
      "candidates": ["SqliteCustomers"]
    }
  ]
}
```

Implemented repair safety levels:

```text
safe likely risky manual
```

## 5. Semantic Graph IR

The Graph IR schema identifier is `ax-ir/4.0`. It contains:

- typed nodes;
- ownership and binding edges;
- contracts;
- effects;
- capabilities;
- native implementation references.

Node and edge ordering is canonical, making compilation deterministic.

## 6. Packed Graph IR

Packed Graph IR begins with version `4`. Frames are separated by `;` and fields
by `|`. Strings use JSON quoting only when delimiters or whitespace require it.

Implemented frame opcodes:

| Opcode | Frame |
|---|---|
| `1` | application |
| `10` | node |
| `11` | non-ownership edge |
| `20` | contract |
| `21` | effect |
| `22` | capability |

Node and edge kinds use numeric subcodes. Ownership edges are encoded as parent
references and reconstructed when decoding. The decoder must reproduce the same
canonical Graph IR.

The matrix formatter wraps complete frames at a configured width. Newlines are
formatting and do not change semantics.

## 7. CLI

```text
axl-compiler check|diagnose <input.axl> [--json]
axl-compiler ir <input.axl>
axl-compiler pack <input.axl> [--matrix]
axl-compiler fmt <input.axl>
axl-compiler blocks <input.axl>
axl-compiler experiment <input.axl> <output-dir>
axl-compiler unpack <packed.axl>
axl-compiler eval <input.axl> <flow> <input.json>
axl-compiler serve <input.axl> [address]
```

`check` and `diagnose` are aliases. With `--json` they emit protocol
`axl-check/1` (schema `schema/axl-check-1.schema.json`) on stdout for both
success and failure. A success report contains `ok`, `path`, `app`, `schema`,
`nodes` and `edges`. A failure report contains `ok: false`, `path` and a
`diagnostics` array. Each diagnostic includes stable `code`, `phase`, `severity`,
`message`, optional `path`, 1-based `span`, optional `expected`/`found`,
optional `hint`, `fix_safety` and `repairs`.

`experiment` writes:

```text
app.axl          canonical readable source
app.axir.json    Semantic Graph IR
app.packed.axl   matrix-formatted Packed Graph IR
targets/
  manifest.json
  blocks/open-blocks.json
  rust/axl_contracts.rs
  react/axl_slots.ts
  sql/schema.sql
  agents/agents.json
  flows/flows.json
  http/routes.json
  providers/providers.json
```

The current target adapters generate Rust data/capacity contracts, a React slot
registry, SQL entity DDL, an agent manifest and an `axl-open-block/2` manifest
that lists every blueprint surface. `flows/flows.json` exposes the ordered,
typed executable bodies and dependencies using protocol `axl-flow/2`.
`http/routes.json` uses `axl-http/1`. They deliberately stop before claiming to
generate a complete production application. `providers/providers.json` uses
`axl-provider/1` and preserves each skill's typed runtime configuration.

## 8. Design sources adopted

- ilo: structured diagnostics, repair plans and safety classification.
- NURL: local deterministic grammar and canonical machine interface.
- KARN: planned resilience and concurrency primitives.
- IntentLang: contracts, intent/implementation separation and auditability.
- LMQL: planned schema-constrained AI generation.
- SGLang: planned AI graph scheduling and cache reuse.
- Jason: optional belief/goal/plan agent model.
- SARL: capacity/skill separation used for open ports.
- Agentlang: full-stack ontology used as the blueprint application model.

## 9. Explicitly not implemented yet

- contract expression type checking;
- branch statement blocks and mutable variables;
- generated standalone Rust handlers and React components from Graph IR;
- streaming HTTP bodies;
- production auth adapters (secret references, OAuth);
- SQL relationships and target-specific schema evolution beyond versioned
  history markers;
- native ABI verification;
- declared migration SQL scripts, queries and multi-database adapters
  (PostgreSQL, MySQL, document/KV);
- blueprint package registry;
- cross-package blueprint overlays and package lockfiles;
- runtime behavior for state, actions, errors and policies;
- effect budgets and capability policy enforcement;
- source maps from generated target code;
- AI and agent runtime execution;
- stable backward compatibility.

These are later gates. The current experiment validates the source language,
open-port type model, agent diagnostics and deterministic IR pipeline. Flow
Runtime 2 executes expressions and capacity calls through a replaceable runtime
ABI. HTTP Runtime 1 dispatches exact JSON routes through Axum. Durable persistence for configured SQLite stores, jobs and cache entries is
proven; Logger/Metrics/Tracer observability is proven through memory skills;
capacity-backed transactions prove commit durability and rollback across memory
and SQLite; capacity-backed migrations prove versioned schema history (up/down/
status) with SQLite persistence across runtime recreate; the Gate 4 UI slice
(`ui` / `page` / `form` / `drawer` / `modal`, `axl-ui/1`, `render`, sidebar +
mobile bottom-nav shell) is executable; component registry slots and KPI/charts
kit are not implemented yet.

## 10. Verified examples and guides

- `examples/apps/balance-ui.axl` — minimal UI page bound to a flow (`axl-ui/1`).
- `examples/apps/form-demo.axl` — minimal UI form bound to a POST api route with nav shell.
- `examples/blocks/01-store.axl` — capacity, Rust skill and explicit binding.
- `examples/blocks/02-ui-slot.axl` — typed React slot with a default provider.
- `examples/blocks/03-hook.axl` — typed lifecycle hook and recorded contracts.
- `examples/blocks/04-agent.axl` — belief/goal/plan graph model.
- `examples/blocks/05-open-dataview.axl` — all implemented open-block surfaces.
- `examples/blocks/06-instance-override.axl` — typed parameter and provider overrides.
- `examples/catalog/software-foundation.axl` — primary open block contracts
  including transactions and migrations.
- `examples/apps/import-demo.axl` — multi-file import of a shared module.
- `examples/apps/import-diamond-demo.axl` — diamond import merges shared email once.
- `examples/apps/oauth-boundary.axl` — OAuth capacity + demo `rust::axl::auth::oauth` provider.
- `examples/apps/postgres-boundary.axl` — PostgreSQL store, tx and migrate providers.
- `examples/apps/mysql-boundary.axl` — MySQL store, tx and migrate providers.
- `examples/apps/document-tx-boundary.axl` — document JSON store tx and migrate providers.
- `examples/apps/sql-pushdown-boundary.axl` — SQLite store query SQL pushdown demo.
- `examples/apps/drawer-boundary.axl` — Gate 4 UI drawer overlay demo.
- `examples/apps/modal-boundary.axl` — Gate 4 UI modal overlay demo.
- `examples/apps/bottom-nav-boundary.axl` — Gate 4 responsive shell mobile bottom nav.
- `examples/modules/math-lib.axl` — imported balance helpers.
- `hosts/portal-web` — Vite React host for `axl-ui/1` codegen (cookie proxy).
- `examples/next/crm.axl` — composed CRM graph.
- `docs/blocks.md` — construction guide and current limitations.
- `docs/executable-flows.md` — executable syntax, commands and current boundary.
- `docs/status.md` — concise implemented/planned matrix.

The Rust integration test `documented_examples.rs` compiles every example and
verifies Packed IR round-trip equality with Semantic Graph IR.
