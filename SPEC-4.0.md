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
`rust::axl::store::memory` and `rust::axl::store::sqlite`. They are not tied to
`Movement` or to the cashflow application. SQLite uses an in-memory connection
when no path is configured and opens a durable file when the skill declares a
`path` config. Independent runtimes configured with the same file observe the
same records.

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
claims. Demo secrets may appear as plaintext skill config (same honesty rule as
the static bearer fixture). True secret references that never enter Graph or
manifest plaintext are Gate 8. OAuth adapters are not implemented.

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
axl-compiler check <input.axl> [--json]
axl-compiler ir <input.axl>
axl-compiler pack <input.axl> [--matrix]
axl-compiler fmt <input.axl>
axl-compiler blocks <input.axl>
axl-compiler experiment <input.axl> <output-dir>
axl-compiler unpack <packed.axl>
axl-compiler eval <input.axl> <flow> <input.json>
axl-compiler serve <input.axl> [address]
```

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
- SQL relationships, migrations and target-specific schema evolution;
- native ABI verification;
- transactions, migrations, queries and multi-database adapters;
- blueprint package registry;
- package imports and cross-package blueprint overlays;
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
runtime UI behavior is not implemented yet.

## 10. Verified examples and guides

- `examples/blocks/01-store.axl` — capacity, Rust skill and explicit binding.
- `examples/blocks/02-ui-slot.axl` — typed React slot with a default provider.
- `examples/blocks/03-hook.axl` — typed lifecycle hook and recorded contracts.
- `examples/blocks/04-agent.axl` — belief/goal/plan graph model.
- `examples/blocks/05-open-dataview.axl` — all implemented open-block surfaces.
- `examples/blocks/06-instance-override.axl` — typed parameter and provider overrides.
- `examples/catalog/software-foundation.axl` — fourteen primary open block contracts.
- `examples/apps/cashflow-core.axl` — executable validation and balance flows.
- `examples/next/crm.axl` — composed CRM graph.
- `docs/blocks.md` — construction guide and current limitations.
- `docs/executable-flows.md` — executable syntax, commands and current boundary.
- `docs/status.md` — concise implemented/planned matrix.

The Rust integration test `documented_examples.rs` compiles every example and
verifies Packed IR round-trip equality with Semantic Graph IR.
