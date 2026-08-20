# AXL Vision

[Italiano](../overview.md)

## For agents only

AXL — **Agent eXecution Language** — is a general-purpose language designed exclusively for software agents. It does not seek familiarity with Python, Rust, or JavaScript. Its priority is to reduce tokens, ambiguity, and generation cost.

The canonical source is a versioned stream:

```axl
2;10|x|#2,#3,#4,*,+|i;12|$x
```

It requires no newlines, indentation, or long keywords. Numeric opcodes describe instructions; RPN expressions eliminate parentheses and syntactic precedence.

## General-purpose goal

Compact does not mean limited. AXL must be able to express:

- backends, APIs, services, and distributed applications;
- browsers and frontends through WASM/DOM;
- native desktop and mobile applications;
- CLIs, automation, and system software;
- GPU, graphics, audio, and games;
- agents, workflows, memory, tasks, events, and models;
- reusable libraries and components.

## Backends and bridges

Rust is the first low-level runtime/compiler, chosen for security, performance, and portability. AXL is not tied to Rust: AX-HIR/AX-MIR and a versioned ABI will enable multiple backends and bridges.

```text
AXL → AX-HIR → AX-MIR → Rust/native
                       → VM
                       → WASM/WASI
                       → C ABI
                       → DOM/WebGPU
                       → mobile/desktop/OS bridge
                       → future backends
```

The AXL source does not change when the target changes.

## Principles

1. **Agent-only:** token efficiency before human readability.
2. **General-purpose:** no category of software is excluded from the architecture.
3. **Determinism:** parsing and compilation do not depend on LLMs.
4. **Canonical form:** a normal representation facilitates hashing, caching, and signing.
5. **Capability security:** external effects are denied by default.
6. **Versioned IR:** source, HIR, MIR, and ABI evolve through explicit contracts.
7. **Portability:** the same results, effects, and errors across different backends.
8. **Machine diagnostics:** stable, localizable, and automatically correctable errors.

## Actual status

### Available

- single-line Compact Source 2;
- canonical writer and `axl pack`;
- deterministic parser and RPN;
- basic types, functions, and modules;
- control flow;
- agents, workflows, and tools;
- scoped AM with SQLite, TTL, and metadata;
- policy, approval, audit, and budgets;
- versioned AX-IR JSON;
- Python reference runtime and CLI.

### To be built

- collections, records, enums, option/result, and pattern matching;
- tasks, events, async, and structured concurrency;
- AX-HIR/AX-MIR and capability ABI;
- Rust runtime/compiler;
- VM, native, WASM/WASI;
- filesystem, network, HTTP, database, DOM, GPU, and desktop/mobile bridges;
- package manager, LSP, debugger, and profiler;
- application frameworks.

The Python reference implementation establishes semantics and conformance; it is not the final runtime.
