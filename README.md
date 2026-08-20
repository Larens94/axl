# AXL — Agent eXecution Language

Executable reference implementation of a compact agent-native programming language.

## V0.2 milestone

- deterministic parser and typed IR;
- typed strings, integers, and booleans;
- arithmetic and comparison expressions;
- `if/else/end` control flow;
- typed memory write and recall;
- explicit `call tool(...)` capability registry;
- deny-by-default unknown tools;
- CLI and automated tests.

## Run

```bash
python3 -m axl run examples/decision.axl
```

Expected output:

```text
ready
14
```

## Test

```bash
python3 -m unittest discover -s tests -v
```

## Direction

Next layers: loops with execution budgets, persistent memory adapters, tool policies/approvals, agent declarations, workflows, serialized IR, then an optimized Rust/WASM runtime.

See [SPEC.md](SPEC.md) for normative syntax and semantics.
