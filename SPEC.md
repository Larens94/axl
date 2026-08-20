# AXL V0.3 — Core specification

AXL (Agent eXecution Language) is a compact, deterministic language for agentic programs.

```text
AXL source → parser → typed IR → budgeted interpreter → result
                                      ↓          ↓
                           explicit tools   memory adapter
```

## Grammar

```ebnf
program      = { instruction } ;
instruction  = memory_write | binding | emit | conditional | loop ;
memory_write = "memory", identifier, "=", expression ;
binding      = "let", identifier, "=", expression ;
emit         = "emit", expression ;
conditional  = "if", expression, { instruction },
               [ "else", { instruction } ], "end" ;
loop         = "while", expression, { instruction }, "end" ;
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
- `memory key = expression` writes a typed value through the configured memory adapter.
- `recall key` reads memory and fails explicitly if absent.
- `let name = expression` creates or replaces a local binding.
- `emit expression` appends a typed value to deterministic output.
- `if` and `while` require boolean conditions.
- Every instruction and loop iteration consumes execution budget. Exceeding `max_steps` terminates execution with an error.
- `call tool(args)` invokes only a host-registered tool. Unknown tools are denied by default.
- Parse and runtime failures never invent fallback values.

## Memory adapters

`MemoryStore` defines `get`, `set`, and `snapshot`. V0.3 provides:

- `InMemoryStore` for isolated execution;
- `SQLiteMemoryStore` for typed persistence across processes.

The adapter boundary allows future semantic, graph, episodic, or remote providers without changing AXL syntax.

## Runtime and security boundary

Typed IR is the stable runtime boundary. The language has no implicit filesystem, network, shell, model, or secret access. Capabilities enter through explicit tool and memory adapters. Future Rust/WASM runtimes should consume equivalent IR and preserve budget/policy semantics.
