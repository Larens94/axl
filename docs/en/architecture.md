# Stack Architecture

[Italiano](../architecture.md)

## Stable pipeline

The complete AA/AM/AW/AP/AT/AE/AS/AD/AI taxonomy is defined in [AX — Agent eXecution](../ax-ecosystem.md). All surfaces converge on AX-IR before execution.

```text
Compact Source 2
      │
      ▼
deterministic parser
      │
      ▼
AST + resolver + type-checker
      │
      ▼
AX-HIR ── general-purpose + agent primitives
      │
      ▼
AX-MIR ── CFG, lowered types, effects, capability ABI
      │
      ├── VM
      ├── Rust/native
      ├── WASM/WASI
      └── platform bridge
             ├── filesystem/network/HTTP/database
             ├── DOM/browser/WebGPU
             ├── desktop/mobile/OS
             └── future backends
```

## Source layer

The canonical source is optimized for agents: numeric opcodes, delimited frames, RPN expressions, and no indentation. It is compact but versioned and fully deterministic.

The existing verbose frontend is used only for migration, debugging, and conversion with `axl pack`.

## Source analysis

Responsibilities:

- strict framing and parsing;
- per-frame/token diagnostics;
- module and namespace resolution;
- static type-checking;
- construction of valid HIR;
- no runtime effects.

Today, the frontend/reference runtime is written in Python. The test corpus makes its semantics transferable.

## AX-IR, HIR, and MIR

- **AX-IR JSON 1.x:** the current interoperable contract.
- **AX-HIR:** functions, types, agents, workflows, memory, and high-level effects.
- **AX-MIR:** basic blocks, control flow, value layouts, and lowered calls and capabilities.

Keeping the layers separate enables optimization and multiple targets without contaminating the source language.

## Execution engine

The runtime governs:

- execution, scheduling, and cancellation;
- AM memory and scopes;
- capabilities, policies, and approvals;
- audit and budgets;
- bridges to hosts and platforms.

Rust is the first planned definitive implementation for safety and performance. It is not part of the syntax, nor is it the only permitted backend.

## Capability ABI and bridges

Each bridge exposes typed, versioned contracts:

```text
capability id + ABI version + input/output types + effects + target + cancellation
```

This allows the same AXL program to use different implementations for Linux, Windows, macOS, browsers, Android, iOS, GPUs, or the cloud.

The concrete application design, including AX-UI, modern networking, renderers, and vertical demos, is described in [Application Demo and Platform Analysis](../platform-demo-analysis.md).

## Current components

```text
axl/compact.py        Compact Source 2 parser/writer
axl/parser.py         dispatcher + legacy frontend
axl/compiler.py       modules and namespaces
axl/ir.py             current typed IR
axl/typechecker.py    type-checker
axl/validation.py     semantic validation
axl/serialization.py  AX-IR JSON
axl/interpreter.py    reference runtime
axl/memory.py         AM
axl/policy.py         capability policy/audit
axl/__main__.py       CLI
```

## Compatibility

- versioned source;
- stable canonical output;
- immutable published IR schemas;
- tested legacy decoder;
- observational equivalence between the reference runtime, VM, native, and WASM;
- replaceable bridges that do not require changing the program.
