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
- operation name;
- JSON input already checked against the AXL operation type.

The adapter returns a successful JSON value or a provider error string. The AXL
runtime validates successful output. For `Result<T>` operations, `?` binds `T`
or propagates the provider error from the surrounding `Result<T>` flow.

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
The SQLite connection is currently in-memory and scoped to one runtime; durable
configuration, schemas and transactions are not implemented yet.

## Conformance gate for future adapters

Every backend, database, AI, IoT or agent-tool adapter must prove:

1. static capacity compatibility;
2. input and output validation;
3. deterministic `Result` propagation;
4. isolation between provider instances;
5. no application-specific branching in the general runtime;
6. positive, failure and Packed IR round-trip tests;
7. documentation and manifest schema updates.
