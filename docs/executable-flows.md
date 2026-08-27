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

## Provider ABI

`ProviderRuntime` is public and replaceable. The interpreter passes it the
provider name, capacity, native implementation identifier, operation and JSON
input. A custom runtime can therefore implement backend, IoT, AI or agent tools
without adding application-specific branches to the interpreter.

Two general storage implementations exist today:

| Native binding | Operations | Current lifetime |
|---|---|---|
| `rust::axl::store::memory` | `save`, `find`, `delete`, `list` | one runtime |
| `rust::axl::store::sqlite` | `save`, `find`, `delete`, `list` | in-memory SQLite, one runtime |

The SQLite implementation uses the SQLite engine, but `eval` currently creates
a fresh in-memory connection. Durable paths and migrations are a later gate.

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
scalar values, `Option<T>`, `List<T>` and `Set<T>` at runtime.

## CLI

```text
axl-compiler eval <input.axl> <flow> <input.json>
```

The generated `targets/flows/flows.json` manifest uses `axl-flow/2` and exposes
dependencies, selected providers and ordered statements without requiring a
source parser.

## Current boundary

Flow Runtime 2 does not implement mutable variables, branch statement blocks,
collection literals/grouping, async execution, durable persistence, events,
state mutation, concurrency or UI bindings. The next vertical slice is grouping
and async semantics, then durable storage and richer HTTP behavior.
