# AX-IR and Compatibility

[Italiano](../ax-ir.md)

## Definition

**AX-IR** is AXL's typed, versioned intermediate representation. It decouples syntax, static analysis, and the runtime.

## JSON envelope

AX-IR 1.2 is serialized as follows:

```json
{
  "ir_version": "1.2",
  "program": {
    "type": "Program",
    "instructions": []
  }
}
```

Nodes use a `type` discriminator and closed sets of fields. The decoder rejects:

- unknown versions;
- unknown nodes or fields;
- duplicate JSON keys;
- literal types not supported by AXL;
- expressions in instruction positions;
- malformed collections;
- invalid identifiers and operators;
- invalid workflow references or graphs;
- payloads exceeding the configured limit.

## Published versions

- **1.0:** agents, workflows, memory, tools, and basic control flow.
- **1.1:** typed functions, `return`, function calls, and binding annotations.
- **1.2:** `ListExpression` and homogeneous `list<T>` types.

The 1.2 decoder reads 1.0 and 1.1 documents and applies controlled upgrades where necessary. Every published schema remains immutable and lives in a separate file.

## Modules

Imports are compiler directives and do not survive in the IR. The compiler resolves files, applies qualified namespaces, and produces a single validated `Program`.

## Future evolution

The current JSON IR is high-level. The definitive stack will distinguish between:

### AX-HIR

- resolved functions and types;
- agents, workflows, memory, and capabilities;
- semantics close to the language;
- source mapping for diagnostics.

### AX-MIR

- basic blocks and control-flow graphs;
- lowered calls and values;
- layout, ownership, and ABI;
- operations suitable for VM, native, and WASM targets;
- verifiable optimizations.

Each lowering stage must preserve observable output, effects, errors, audit, and limits relative to the reference runtime.

## Schemas

- [`../../schema/axl-ir-1.0.schema.json`](../../schema/axl-ir-1.0.schema.json)
- [`../../schema/axl-ir-1.1.schema.json`](../../schema/axl-ir-1.1.schema.json)
- [`../../schema/axl-ir-1.2.schema.json`](../../schema/axl-ir-1.2.schema.json)
