# AXL — Agent eXecution Language

Initial executable implementation of a compact agent-native programming language.

## Current milestone

- deterministic parser;
- typed intermediate representation;
- memory write/recall operations;
- output primitive;
- reference interpreter;
- CLI and automated tests.

## Run

```bash
python3 -m axl run examples/hello.axl
```

Expected output:

```text
short
```

## Test

```bash
python3 -m unittest discover -s tests -v
```

## Direction

V0.1 is intentionally small. Planned layers: values and expressions, agents/tools, policy gates, workflows, adapter-based persistent memory, serialized IR, then an optimized Rust/WASM runtime.

See [SPEC.md](SPEC.md) for normative syntax and semantics.
