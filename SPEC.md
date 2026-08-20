# AXL V0.2 — Core specification

AXL (Agent eXecution Language) is a compact, deterministic language for agentic programs.

```text
AXL source → parser → typed IR → interpreter → result
                                      ↓
                              explicit tool registry
```

## Grammar

```ebnf
program      = { instruction } ;
instruction  = memory_write | binding | emit | conditional ;
memory_write = "memory", identifier, "=", expression ;
binding      = "let", identifier, "=", expression ;
emit         = "emit", expression ;
conditional  = "if", expression, { instruction },
               [ "else", { instruction } ], "end" ;
expression   = primary, { operator, primary } ;
primary      = string | integer | boolean | identifier |
               "recall", identifier | tool_call |
               "(", expression, ")" ;
tool_call    = "call", identifier, "(", [ expression,
               { ",", expression } ], ")" ;
operator     = "+" | "-" | "*" | "/" |
               "==" | "!=" | ">" | "<" | ">=" | "<=" ;
boolean      = "true" | "false" ;
identifier   = (letter | "_"), { letter | digit | "_" } ;
```

Blank lines and lines beginning with `#` are ignored. Multiplication and division bind before addition and subtraction; comparisons bind last.

## Semantics

- Values currently include strings, integers, and booleans.
- `memory key = expression` writes a typed value to execution memory.
- `recall key` reads memory and fails explicitly if absent.
- `let name = expression` creates or replaces a local binding.
- `emit expression` appends a typed value to deterministic output.
- `if` requires a boolean and executes exactly one selected branch.
- `call tool(args)` invokes only a host-registered tool. Unknown tools are denied by default.
- Parse and runtime failures never invent fallback values.

## Runtime boundary

Syntax is user-facing. Typed IR is the stable runtime boundary. A future optimized Rust/WASM runtime should consume equivalent IR rather than reimplement semantics ad hoc.

## Security baseline

The language has no implicit filesystem, network, shell, model, or secret access. Capabilities enter through the explicit tool registry. Policy and approval metadata will extend this boundary in later versions.
