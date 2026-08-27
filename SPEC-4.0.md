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
  native rust axl::store::sqlite_customers
  effect db.read
  effect db.write
  capability crm.customer.write
```

Implemented native targets:

```text
rust react sql ai iot wasm
```

AXL validates the declared target and capacity. Target ABI validation is planned.

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
implementation, operation and JSON input. The built-in experiment implements
generic `save`, `find`, `delete` and `list` operations for
`rust::axl::store::memory` and `rust::axl::store::sqlite`. They are not tied to
`Movement` or to the cashflow application. The CLI currently constructs a new
runtime for each `eval`; its SQLite connection is therefore in-memory and not
durable across CLI processes.

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

### HTTP API

An API exposes checked flows without handwritten controllers:

```axl
api CashflowApi
  post /movements Movement -> Result<Movement> = ValidateAndStoreMovement
  post /movement-by-id uuid -> Result<Movement> = FindMovement
  post /balance MovementBatch -> money = CalculateLedgerBalance
```

Implemented methods are `get`, `post`, `put`, `patch` and `delete`. Paths are
currently exact absolute paths. Input/output types must exactly match the bound
flow signature. Duplicate routes inside one API and conflicts across APIs are
rejected.

`axl-compiler serve` runs a generic Axum adapter over the HTTP nodes in Graph
IR. JSON input is validated by the flow runtime. A successful value returns
HTTP 200, an AXL `{ "error": ... }` returns 422, invalid input returns 400 and
an unknown route returns 404. One built-in provider runtime is shared by all
requests for the lifetime of the server process. Memory and in-memory SQLite
state therefore survive consecutive requests, but not a server restart.

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
```

The current target adapters generate Rust data/capacity contracts, a React slot
registry, SQL entity DDL, an agent manifest and an `axl-open-block/2` manifest
that lists every blueprint surface. `flows/flows.json` exposes the ordered,
typed executable bodies and dependencies using protocol `axl-flow/2`.
`http/routes.json` uses `axl-http/1`. They deliberately stop before claiming to
generate a complete production application.

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
- collection literals and grouping;
- `parallel`, `race`, retry and timeout execution;
- generated standalone Rust handlers and React components from Graph IR;
- path parameters, query decoding, middleware and streaming HTTP bodies;
- SQL relationships, migrations and target-specific schema evolution;
- native ABI verification;
- durable provider configuration and database paths;
- blueprint package registry;
- package imports and cross-package blueprint overlays;
- runtime behavior for state, events, actions, errors and policies;
- effect budgets and capability policy enforcement;
- source maps from generated target code;
- AI and agent runtime execution;
- stable backward compatibility.

These are later gates. The current experiment validates the source language,
open-port type model, agent diagnostics and deterministic IR pipeline. Flow
Runtime 2 executes expressions and capacity calls through a replaceable runtime
ABI. HTTP Runtime 1 dispatches exact JSON routes through Axum. Durable
persistence and runtime UI behavior are not implemented yet.

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
