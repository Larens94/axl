# AXL — Agent eXecution Language

Executable reference implementation of a compact agent-native programming language.

## V0.3 milestone

- deterministic parser and typed IR;
- strings, integers, booleans, arithmetic and comparisons;
- `if/else/end` and budgeted `while/end` control flow;
- configurable execution-step limit against infinite programs;
- typed memory through provider-neutral adapters;
- in-memory and persistent SQLite stores;
- explicit `call tool(...)` registry, deny-by-default;
- CLI, specification, examples, and automated tests.

## Run

```bash
python3 -m axl run examples/loop.axl
```

Persistent memory and custom execution budget:

```bash
python3 -m axl run --memory .axl-memory.sqlite --max-steps 5000 program.axl
```

## Test

```bash
python3 -m unittest discover -s tests -v
```

## Direction

Next layers: scoped memory metadata, tool policies/approvals, agent declarations, workflows, serialized IR, then an optimized Rust/WASM runtime.

See [SPEC.md](SPEC.md) for normative syntax and semantics.
