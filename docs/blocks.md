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

## 5. Composing the CRM graph

[`examples/next/crm.axl`](../examples/next/crm.axl) combines entities, two
backend inputs, two UI slots, one lifecycle hook, contracts and an agent. Its
open ports are:

| Port | Type | Current provider | Replaceable with a compatible skill |
|---|---|---|---|
| `store` | `CustomerStore` | `SqliteCustomers` | yes |
| `auth` | `AuthProvider` | `JwtAuth` | yes |
| `table.row` | `CustomerRow` | `DefaultCustomerRow` | yes |
| `mobile.card` | `CustomerCard` | `DefaultCustomerCard` | yes |
| `before.save` | `CustomerPolicy` | `ValidateCustomer` | yes |

This is the implemented answer to an open block: stable typed ports on the AXL
surface, with target-specific Rust or React referenced below them.

## Current boundary

AXL 4 can validate the graph and generate contracts for targets. It cannot yet
express arbitrary `if`, `else`, loops or function bodies. Those must not be
presented as current AXL syntax. See [implementation status](status.md).
