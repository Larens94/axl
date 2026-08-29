# Executable AXL flows

AXL Flow Runtime 2 executes behavior and typed capacity calls over Semantic
Graph IR. Rust implements the general interpreter and provider ABI; application
behavior stays in AXL.

## Syntax implemented now

```axl
enum MovementKind
  income
  expense

flow ValidateMovement Movement -> Result<Movement>
  let positive_amount = input.amount > 0
  require positive_amount else "amount_must_be_positive"
  return input

flow CalculateBalance BalanceInput -> money
  let balance = input.income - input.expense
  return balance

capacity MovementStore
  op save Movement -> Result<Movement>
  op find uuid -> Result<Movement>

skill MemoryMovements provides MovementStore
  native rust axl::store::memory

skill DurableMovements provides MovementStore
  native rust axl::store::sqlite
  config path: text = "./build/movements.db"

flow StoreAndLoadMovement Movement -> Result<Movement>
  in store: MovementStore = MemoryMovements
  call saved = store.save(input)?
  call loaded = store.find(saved.id)?
  return loaded

flow BuildMovementView Movement -> MovementView
  let direction = if input.kind == MovementKind.income then "Entrata" else "Uscita"
  make view: MovementView
    id = input.id
    direction = direction
    signed_amount = if input.kind == MovementKind.income then input.amount else -input.amount
    category = input.category
  return view
```

`input` has the type declared in the flow header. `let` introduces an immutable
typed value. `require` accepts only `bool` and is available only to flows that
return `Result<T>`. `return` must occur exactly once and must be last.

For a `Result<T>` flow, a successful return becomes `{ "ok": value }`; a failed
require becomes `{ "error": message }`.

`in` is an open dependency port. Its provider must implement the declared
capacity. `call` checks the operation and argument type. A `Result<T>` operation
requires `?`; success binds `T`, while provider failure immediately returns
`{ "error": message }` from the surrounding `Result<T>` flow.

`make name: Entity` constructs a record over multiple lines. Every assignment
is type checked; optional fields may be omitted, while every required field must
appear exactly once. `if condition then value else value` is a lazy expression:
the condition must be boolean and both value branches must have compatible
types.

`fold` iterates a `List<T>` or `Set<T>` with an immutable typed accumulator.
`value` is the current accumulator and the name after `as` is the current item:

```axl
fold balance: money = input.movements from 0 as movement
  next = if movement.kind == MovementKind.income
    then value + movement.amount
    else value - movement.amount
```

`run name = Flow(argument)` composes a non-fallible flow. A target returning
`Result<T>` requires `?`, exactly like a capacity call. The nested flow shares
the same provider runtime, so composed operations see the same runtime state.

`match` maps every variant of an enum to a typed value and is exhaustive:

```axl
match signed_amount: money = input.kind
  income => input.amount
  expense => -input.amount
```

Unknown, duplicate or missing variants and incompatible case values are
rejected before runtime.

`filter` preserves a collection type and `map` transforms its items:

```axl
filter incomes: List<Movement> = input.movements as movement
  where = movement.kind == MovementKind.income
map amounts: List<money> = incomes as movement
  value = movement.amount
```

Both statements create scoped immutable item variables. Sources and outputs are
limited to `List<T>` and `Set<T>`; predicates and mapped values are checked
statically and again at runtime.

`sort` produces a deterministic `List<T>` from a `List<T>` or `Set<T>`:

```axl
sort newest: List<Movement> = input.movements as movement
  by = movement.occurred_at
  direction = desc
```

Keys are ordered numeric, string-like or enum values. Equal keys preserve source
order. The compiler rejects a changed item type, an unordered key or a direction
other than `asc` and `desc`.

`group` creates `Map<K,List<T>>` buckets without a target-language helper:

```axl
group by_category: Map<text,List<Movement>> = input.movements as movement
  by = movement.category
```

The source can be `List<T>` or `Set<T>`. Keys are string-like scalars or enums,
and each bucket retains the encounter order of its source items. Map keys,
bucket item types and runtime values are all validated.

List literals are inferred, executable and canonically multiline:

```axl
let categories = [
  "consulting",
  "software"
]
```

Items may be expressions and must have one compatible type. Empty lists are
rejected until AXL gains an explicit contextual type annotation for them.

`parallel` executes a target flow concurrently for every source item:

```axl
parallel views: List<MovementView> = input.movements as movement
  run = BuildMovementView(movement)
```

Arguments, target signatures and `List<Output>` are checked statically. Results
retain source order regardless of completion order. A target returning
`Result<T>` requires `?`; errors propagate in source order. Provider-backed
workers use the runtime `fork` contract, so integrations decide explicitly how
their handles and state are shared.

`attempt` adds bounded resilience to an idempotent capacity operation:

```axl
attempt found = store.find(input)?
  retry = 2
  timeout_ms = 250
```

Every invocation runs behind a real timeout using a provider fork. Provider
errors and deadlines are retried, then propagate through the normal `Result<T>`
contract. The compiler refuses non-idempotent operations, more than ten retries
and deadlines outside 1–60000 milliseconds.

`race` returns the first successful concurrent candidate:

```axl
race found: Movement = input.ids as candidate
  run = FindMovement(candidate)?
```

Losing workers are not treated as cancelled side effects, so the analyzer
recursively permits only flows composed from idempotent operations. Result
errors remain candidates for failure until every worker has failed; the final
error is selected in source order.

## Provider ABI

`ProviderRuntime` is public and replaceable. The interpreter passes it the
provider name, capacity, native implementation identifier, typed configuration,
operation and JSON input. A custom runtime can therefore implement backend, IoT, AI or agent tools
without adding application-specific branches to the interpreter.

Two general storage implementations exist today:

| Native binding | Operations | Current lifetime |
|---|---|---|
| `rust::axl::store::memory` | `save`, `find`, `delete`, `list`, `query` | one runtime |
| `rust::axl::store::sqlite` | `save`, `find`, `delete`, `list`, `query` | in-memory by default; durable with `config path` |
| `rust::axl::store::postgres` | `save`, `find`, `delete`, `list`, `query`, `find_by` | requires `config url` (e.g. `secret("AXL_POSTGRES_URL")`) |
| `rust::axl::store::mysql` | `save`, `find`, `delete`, `list`, `query`, `find_by` | requires `config url` (e.g. `secret("AXL_MYSQL_URL")`) |
| `rust::axl::store::document` | `save`, `find`, `delete`, `list`, `query` | in-memory by default; durable JSON file with `config path` |

Skill configuration is a checked, first-class graph surface. `config path:
text = "..."` reaches Graph IR, Packed IR, `axl-provider/1` and the runtime.
Two separate `eval` processes can therefore reopen the same SQLite database or
document JSON file. Transaction begin/commit/rollback is executable through
`TransactionManager`. Migration up/down/status is executable through
`MigrationRunner` (memory + SQLite schema history). Typed store `query`
(filter/order/page → page entity) is executable on memory, SQLite and document
adapters. PostgreSQL/MySQL and document tx/migrate remain later Gate 3 work.

## Expression semantics

Implemented values and operations:

| Area | Implemented |
|---|---|
| Access | `input.field`, immutable local values, `Enum.variant` |
| Literals | boolean, integer, float and JSON string |
| Arithmetic | `+`, `-`, `*`, `/`, unary `-` |
| Comparison | `==`, `!=`, `>`, `>=`, `<`, `<=` |
| Logic | `!`, `&&`, `||` |
| Choice | `if condition then value else value` |
| Grouping | parentheses and standard precedence |

Input and return JSON values are checked against entities, enum variants,
scalar values, `Option<T>`, `List<T>`, `Set<T>` and `Map<K,V>` at runtime.

## CLI

```text
axl-compiler eval <input.axl> <flow> <input.json>
```

The generated `targets/flows/flows.json` manifest uses `axl-flow/2` and exposes
dependencies, selected providers and ordered statements without requiring a
source parser.

## Current boundary

Flow Runtime 2 does not implement mutable variables, branch statement blocks,
state mutation or UI bindings. Transactions are capacity-backed
(`TransactionManager` begin/commit/rollback). Migrations are capacity-backed
(`MigrationRunner` up/down/status with schema history). Typed store queries are
capacity-backed (`query` QuerySpec → page with filter/order/limit/offset on
memory and SQLite). The next data slice is additional database families.
