**Languages:** [Italiano](SPEC.md) · [English](SPEC.en.md)

# AXL 2 Development Specification with Compact Source

## 1. Identity and goal

AXL — Agent eXecution Language — is a general-purpose language designed exclusively for software agents. It does not optimize for human readability: it optimizes tokens, determinism, generation, validation, hashing, and automatic correction.

AXL must be able to express any category of software. Rust is the first low-level runtime/backend, not the only target. Versioned bridges and backends must support native, server, browser/WASM, desktop, mobile, GPU, operating systems, and future platforms without changing source semantics.

## 2. Pipeline

```text
AXL Compact Source 2
→ deterministic parser
→ typed AST/AX-HIR
→ validation + type-check
→ AX-MIR
→ runtime/compiler backend
→ Rust/native | VM | WASM | platform bridge
```

The Python reference implementation currently defines the semantics and conformance corpus. It is not the final runtime.

## 3. Canonical source

```ebnf
source = "2", { ";", frame } ;
frame  = opcode, { "|", field } ;
```

Lines and indentation have no meaning. `;`, `|`, and `,` are structural separators outside JSON strings. The complete format is specified in [`docs/compact-syntax.md`](docs/compact-syntax.md).

### Opcodes 2.0

| Opcode | Operation |
|---:|---|
| 1 | module import |
| 10 | binding |
| 11 | return |
| 12 | emit |
| 20 | memory write |
| 21 | forget |
| 30 | if |
| 31 | else |
| 32 | while |
| 40 | function |
| 50 | agent |
| 51 | workflow |
| 52 | run |
| 99 | end block |

### Expressions

Expressions use RPN and are delimited by commas. Atoms: `#int`, `"string"`, `?1`, `?0`, `$variable`, `@memory`. Operators: `+ - * / = ! > < G L`. Postfix calls: `^function/arity` and `!tool/arity`. `~arity` constructs a list from the values on the stack.

The previous keyword-based format is only a migration and debugging frontend. `axl pack` produces canonical source.

## 4. Types and operators

Current values: strings, integers, booleans, and homogeneous immutable lists. Source codes: `s`, `i`, `b`; the recursive prefix `l` forms `list<T>` (`li`, `ls`, `lb`, `lli`) up to 16 levels. Host coercions are forbidden.

- `+`: integer addition or string concatenation with identical types;
- `-`, `*`, `/`, ordering: integers;
- `/`: error on zero or a fractional result;
- equality: identical runtime types;
- conditions: booleans;
- tool results outside the value algebra: error.

Functions declare typed parameters and returns, have an isolated local frame, and have bounded depth. Arity, types, missing returns, and unknown references are static errors.

## 5. Modules

`1|alias|relative-path` imports function declarations. Paths are relative to the importer. Duplicate aliases, cycles, missing modules, and top-level effects in modules are errors. Namespaces qualify calls (`^math.add/2`).

## 6. Agents and workflows

An agent is a principal with explicit tool grants and local scope. A workflow is a sequential block that runs agents/workflows. Unknown references and cycles are rejected before effects occur.

A tool call succeeds only if:

1. the host registers the capability;
2. the agent declares it in its grants;
3. the policy authorizes the effect;
4. any required approval returns exactly `True`;
5. the budgets permit the call;
6. the result belongs to the value algebra.

## 7. AM — memory

AM is provider-independent. Scope belongs to the host. Each record contains a key, scope, typed value, version, reliability, source, timestamp, and optional TTL. Expiration is checked on read. `forget` is idempotent and respects scope.

## 8. Capabilities and bridges

Filesystem, network, HTTP, database, models, UI, GPU, and operating-system APIs are not vendor-specific keywords. They are typed capability contracts translated to host adapters and bridges.

A bridge declares:

- ABI and version;
- accepted/produced types;
- effects and required capabilities;
- supported targets;
- limits, cancellation, and error behavior.

AXL source remains independent of the Rust implementation, C ABI, WASI, JVM, JavaScript host, or future backend.

## 9. Policy, auditing, and budgets

Tools are denied by default. The `approval_required`, `approved`, `denied`, `executed`, and `failed` decisions are verifiable through auditing. Secrets must not appear in source, IR, output, or audit arguments.

Positive budgets limit steps/expressions, call depth, bytes and nodes of intermediate values, collection depth, bytes in the canonical serialization of output values (excluding the line delimiter), tool calls, and memory operations. Blocking host plugins require external isolation and timeouts.

## 10. AX-IR 1.2

Envelope:

```json
{"ir_version":"1.2","program":{"type":"Program","instructions":[]}}
```

Strict decoder: versions, nodes, fields, types, placement, references, and cycles are validated. AX-IR 1.0 and 1.1 remain readable. Published schemas are immutable.

## 11. Compatibility

- same Compact Source 2 → same semantics;
- canonical writer → stable representation;
- reference runtime and optimized backends → observational equivalence;
- new targets → same effects, errors, and capability boundaries;
- incompatible source or IR changes → explicit new version.
