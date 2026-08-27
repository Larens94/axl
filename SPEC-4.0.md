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
  in store: CustomerStore
  out api: CrudApi<Customer>

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
| `use` | explicit provider binding | n/a |

Providers are type checked. A skill satisfies a port only when its `provides`
capacity equals the port type. A blueprint output may be referenced as
`Blueprint.output`.

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

## 3. Type system

Implemented built-ins:

```text
unit bool int float text string email uuid datetime money bytes duration
Result Option List Set Map Stream Future UI CrudApi
```

Declared entities and capacities are also valid types. Generic references are
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
axl-compiler experiment <input.axl> <output-dir>
axl-compiler unpack <packed.axl>
```

`experiment` writes:

```text
app.axl          canonical readable source
app.axir.json    Semantic Graph IR
app.packed.axl   matrix-formatted Packed Graph IR
targets/
  manifest.json
  rust/axl_contracts.rs
  react/axl_slots.ts
  sql/schema.sql
  agents/agents.json
```

The current target adapters generate Rust data/capacity contracts, a React slot
registry, SQL entity DDL and an agent manifest. They deliberately stop before
claiming to generate a complete production application.

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
- function bodies and general control flow;
- `parallel`, `race`, retry and timeout execution;
- executable Rust handlers and React components from Graph IR;
- SQL relationships, migrations and target-specific schema evolution;
- native ABI verification;
- blueprint package registry;
- effect budgets and capability policy enforcement;
- source maps from generated target code;
- AI and agent runtime execution;
- stable backward compatibility.

These are later gates. The current experiment validates the source language,
open-port type model, agent diagnostics and deterministic IR pipeline.
