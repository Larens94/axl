# AXL provider runtime ABI

Flow Runtime 2 separates application behavior from native integrations. A flow
calls a typed capacity port; the selected skill identifies the provider; a
runtime adapter performs the operation.

```text
AXL call
  -> flow input port
  -> capacity operation
  -> compatible skill
  -> ProviderRuntime
  -> native service
```

The Rust ABI is the public `ProviderRuntime` trait. Each invocation contains:

- provider and capacity names;
- native implementation identifier;
- typed provider configuration from the skill declaration;
- operation name;
- JSON input already checked against the AXL operation type.

The adapter returns a successful JSON value or a provider error string. The AXL
runtime validates successful output. For `Result<T>` operations, `?` binds `T`
or propagates the provider error from the surrounding `Result<T>` flow.

`ProviderRuntime` is `Send` and exposes `fork`. The `parallel` statement asks
for one fork per worker. A provider that cannot define safe concurrent behavior
returns an explicit fork error instead of being silently serialized or cloned.
The built-in runtime forks share synchronized memory and SQLite handles.

The same fork contract powers `attempt`. Each attempt runs on an isolated worker
with a real deadline. Retry is statically restricted to capacity operations
marked `idempotent`; late workers may finish, but repeating their operation is
contractually safe.

`race` also uses forked runtimes and may return before losing workers finish.
The analyzer recursively checks that its target flow reaches only idempotent
operations, making late completion safe by contract.

## Openness rule

Application-specific logic must not be added to the interpreter. A new native
system is introduced as a reusable provider adapter behind a capacity. A custom
runtime can implement the trait without changing the parser, Graph IR or AXL
application.

## Built-in storage adapters

Flow Runtime 2 recognizes these implementation identifiers:

```text
rust::axl::store::memory
rust::axl::store::sqlite
```

Both implement generic `save`, `find`, `delete` and `list` operation names.
Saved records require a string `id`. Storage is namespaced by provider skill.
SQLite is in-memory when the skill has no `path`. A typed path makes it durable:

```axl
skill DurableMovements provides MovementStore
  native rust axl::store::sqlite
  config path: text = "./build/movements.db"
```

Connections are selected lazily from AXL configuration. Concurrent forks share
the connection registry, while two independent runtimes reopening the same path
observe the same records. The generated `axl-provider/1` manifest exposes the
configuration for external adapters. Transactions and migrations are not yet
language primitives.

The built-in `rust::axl::auth::bearer` adapter implements an idempotent
`authorize text -> Result<bool>` capacity using a typed `token` config. The
built-in `rust::axl::auth::jwt` adapter implements the same contract with typed
`secret` and `issuer` config, validating compact HS256 JWTs that carry `sub` and
matching `iss` claims. Both are conformance fixtures whose config may appear in
the Graph/manifest; Gate 8 secret references and OAuth providers must use the
same open ABI without plaintext credentials in IR.

## Conformance gate for future adapters

Every backend, database, AI, IoT or agent-tool adapter must prove:

1. static capacity compatibility;
2. input and output validation;
3. deterministic `Result` propagation;
4. isolation between provider instances;
5. no application-specific branching in the general runtime;
6. positive, failure and Packed IR round-trip tests;
7. documentation and manifest schema updates.
8. explicit `fork` behavior when the adapter supports parallel execution.
