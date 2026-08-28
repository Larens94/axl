# Building software with open AXL blocks

An AXL block is not a closed code generator preset. It is a typed graph formed
from a contract, one or more implementations and explicit connection points.
The current compiler represents that model with `capacity`, `skill` and
`blueprint`.

```text
capacity (what) <- skill (how)
        ^              |
        +-- blueprint port/use --+
```

The snippets on this page are copied from files compiled by the test suite.

## 1. Data block

The capacity declares the operations. The skill identifies a native Rust
implementation. The blueprint keeps the port visible and binds it explicitly.

```axl
capacity CustomerStore
  op find uuid -> Result<Customer>
  op save Customer -> Result<Customer>

skill SqliteCustomers provides CustomerStore
  native rust demo::sqlite_customers
  effect db.read
  effect db.write

blueprint CustomerData
  in store: CustomerStore
  use store = SqliteCustomers
```

Source: [`examples/blocks/01-store.axl`](../examples/blocks/01-store.axl).
The compiler checks that `SqliteCustomers` provides exactly `CustomerStore`. It
does not yet inspect the Rust function signature or generate its executable
body.

## 2. Replaceable UI slot

UI is modeled as a typed capacity. The React component is a skill, while the
blueprint exposes a slot with a default provider.

```axl
capacity CustomerRow
  op render Customer -> UI

skill DefaultCustomerRow provides CustomerRow
  native react crm::DefaultCustomerRow

blueprint CustomerList
  slot table.row: CustomerRow = DefaultCustomerRow
```

Source: [`examples/blocks/02-ui-slot.axl`](../examples/blocks/02-ui-slot.axl).
Another skill providing `CustomerRow` can replace the default without changing
the blueprint. Today AXL emits a React slot registry contract; it does not emit
the component implementation.

## 3. Replaceable lifecycle hook

A hook is another typed entry point. This one makes validation replaceable and
records three contracts in Graph IR.

```axl
capacity CustomerPolicy
  op validate Customer -> Result<Customer>

skill ValidateCustomer provides CustomerPolicy
  native rust crm::validate_customer

blueprint CustomerWrite
  hook before.save: CustomerPolicy = ValidateCustomer
  requires customer.valid
  ensures customer.accepted
  invariant Customer.email unique
```

Source: [`examples/blocks/03-hook.axl`](../examples/blocks/03-hook.axl).
The current compiler records `requires`, `ensures` and `invariant` expressions;
it does not yet type-check or execute them.

## 4. Agent model

AXL currently makes beliefs, goals and plans explicit graph nodes.

```axl
agent SalesAssistant
  believe crm.pipeline
  goal qualify_lead
  plan automatic_qualification
  plan request_human_review
  effect ai.generate
  capability crm.lead.read
```

Source: [`examples/blocks/04-agent.axl`](../examples/blocks/04-agent.axl).
At least one goal and one plan are required. Runtime planning and execution are
not implemented.

## 5. Complete open DataView

The complete protocol example exposes configuration, observation and
replaceable behavior without editing generated Rust or React:

```axl
blueprint CustomerDataView
  param page_size: int = 25
  param density: text = "comfortable"
  state selection: Set<uuid>
  event row.selected: Customer
  error load.failed: text

  in data: CustomerQuery
  action refresh: CustomerQuery = SqliteCustomerQuery
  policy view: AccessPolicy = RolePolicy
  slot toolbar: DataToolbar = DefaultDataToolbar
  slot table.row: CustomerRow = DefaultCustomerRow
  slot mobile.card: CustomerCard = DefaultCustomerCard
  hook before.load: LoadHook = TraceLoad

  use data = SqliteCustomerQuery
```

Source:
[`examples/blocks/05-open-dataview.axl`](../examples/blocks/05-open-dataview.axl).

| Surface | Current compiler behavior |
|---|---|
| `param` | checks its type and scalar default; stores it in Graph IR |
| `state` | checks the type; exposes observable state metadata |
| `event` | checks the payload type; exposes emitted-event metadata |
| `action` | checks an optional capacity provider |
| `error` | checks the failure payload type |
| `policy` | checks an optional policy provider |
| `slot` | checks a replaceable structural/UI provider |
| `hook` | checks a replaceable lifecycle provider |

The generated `blocks/open-blocks.json` manifest lets another agent inspect all
these surfaces without parsing source. Runtime execution of state, events,
actions, errors and policies is not implemented yet.

## 6. The openness rule

The compiler rejects a blueprint that exposes only outputs or no ports at all.
It must include at least one of `in`, `slot`, `hook`, `param`, `action` or
`policy`. Diagnostic `AXL-O401` identifies a closed blueprint and proposes
manual opening strategies.

This rule checks that a customization surface exists. It cannot yet prove that
every internal target behavior is exposed, because native ABI inspection and
blueprint composition are later gates.

## 7. Customize without editing the blueprint

An `instance` applies typed overrides while keeping the original blueprint and
all generated targets untouched:

```axl
instance CompactCustomerList of CustomerList
  set page_size = 10
  set density = "compact"
  use table.row = CompactCustomerRow
```

Source:
[`examples/blocks/06-instance-override.axl`](../examples/blocks/06-instance-override.axl).

The compiler resolves `page_size` and `density` against parameters declared by
`CustomerList`. It also verifies that `CompactCustomerRow` provides the same
`CustomerRow` capacity required by `table.row`. Unknown parameters, attempts to
bind state/value surfaces and incompatible providers are rejected.

The instance is encoded as part of Graph IR and Packed IR. Protocol
`axl-open-block/2` exposes its `settings` and `overrides`; the React registry
also publishes `axlInstances` for target integration.

## 8. Composing the CRM graph

[`examples/next/crm.axl`](../examples/next/crm.axl) combines entities, two
backend inputs, two UI slots, one lifecycle hook, contracts and an agent. Its
open ports are:

| Port | Type | Current provider | Replaceable with a compatible skill |
|---|---|---|---|
| `store` | `CustomerStore` | `SqliteCustomers` | yes |
| `auth` | `AuthProvider` | `JwtAuth` | yes |
| `refresh` | `CustomerStore` | `SqliteCustomers` | yes |
| `access` | `AuthProvider` | `JwtAuth` | yes |
| `table.row` | `CustomerRow` | `DefaultCustomerRow` | yes |
| `mobile.card` | `CustomerCard` | `DefaultCustomerCard` | yes |
| `before.save` | `CustomerPolicy` | `ValidateCustomer` | yes |

This is the implemented answer to an open block: stable typed ports on the AXL
surface, with target-specific Rust or React referenced below them.

The CRM also publishes scalar parameters, state, event and error surfaces. Its
generated open-block manifest is therefore sufficient for an agent to discover
both configurable and observable parts of the block.

`CompactSalesCRM` then overrides `page_size`, `density` and `mobile.card` as a
real instance of `CustomerCRM`; no generated target file is edited.

## 9. Foundation catalog for software agents

[`examples/catalog/software-foundation.axl`](../examples/catalog/software-foundation.axl)
is a compiler-verified catalog of fourteen primary blueprint contracts:

| Layer | Open blueprint contracts |
|---|---|
| Data and behavior | `RepositoryBlock`, `QueryBlock`, `CommandBlock`, `TransactionBlock` |
| Network | `ApiBlock`, `EventBlock`, `JobBlock` |
| Interface | `PageBlock`, `DataViewBlock`, `FormBlock`, `NavigationBlock` |
| Operations and agents | `ObservabilityBlock`, `AgentToolBlock`, `ScenarioBlock` |

The catalog exists so an agent can inspect a consistent open surface for the
main software concerns. It compiles to Graph IR and an open-block manifest. The
referenced native skills are target contracts: their executable bodies are not
generated by this milestone.

## Current boundary

AXL 4 can validate the graph and generate contracts for targets. It cannot yet
express arbitrary `if`, `else`, loops or function bodies. Those must not be
presented as current AXL syntax. Local blueprint instances and relative file imports are implemented; cross-package
overlays, package names/versions and registry lockfiles are not. See [implementation
status](status.md).
