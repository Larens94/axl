# AXL V0.1 — Core specification

AXL (Agent eXecution Language) is a compact, deterministic language for agentic programs. The V0.1 reference interpreter proves the pipeline:

```text
AXL source → parser → typed IR → interpreter → result
```

## Grammar

```ebnf
program      = { instruction } ;
instruction  = memory_write | memory_recall | emit ;
memory_write = "memory", identifier, "=", string ;
memory_recall= "let", identifier, "=", "recall", identifier ;
emit         = "emit", identifier ;
identifier   = (letter | "_"), { letter | digit | "_" } ;
string       = '"', { character - '"' - "\\" }, '"' ;
```

Blank lines and lines beginning with `#` are ignored.

## Semantics

- `memory key = "value"`: writes string data to execution memory.
- `let target = recall key`: recalls memory into a local binding.
- `emit target`: appends a local binding to deterministic output.
- Unknown instructions are parse errors carrying their source line.
- Missing memories are runtime errors; no implicit value is invented.

## Next compatibility boundary

Syntax is user-facing. Typed IR is the stable runtime boundary. Future optimized Rust/WASM runtimes should consume equivalent IR rather than reimplement language semantics ad hoc.
