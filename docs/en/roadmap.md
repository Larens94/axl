# AXL Roadmap

[Italiano](../roadmap.md)

AXL progresses through vertical slices: compact source → HIR/MIR → runtime → observable result.

The complete strategy for web, desktop, mobile, AX-UI, backend, and networking is defined in [Application Demo and Platform Analysis](platform-demo-analysis.md). The shared demonstration product is Syncboard, supported by Network Lab and UI Gallery: not six independent codebases or mockups.

## Status

| Area | Status |
|---|---|
| Compact Source 2 + RPN | working |
| Canonical writer + `axl pack` | working |
| Python reference runtime | working |
| AX-IR JSON 1.0/1.1/1.2 | working |
| Functions, modules, AM, agents | working, initial core |
| Homogeneous `list<T>` | working, first collection |
| `map<K,V>` Compact/runtime slice | working locally; IR/persistence next |
| Records, option/result, and user-defined types | next |
| AX-HIR/AX-MIR | planned |
| Rust runtime/compiler | planned |
| Native/VM/WASM | planned |
| Platform bridge | planned |

## M1 — Compact Core

- stabilize Source 2 opcodes;
- source spans for frames/tokens;
- lists, maps, tuples;
- records/structs, enums;
- option/result, pattern matching;
- defined iteration and mutability.

**Gate:** non-trivial general-purpose programs exclusively in compact format.

## M2 — Modules and Packages

- compact export/import;
- machine-first manifest and lockfile;
- deterministic resolver;
- signed packages and content-addressed cache.

**Gate:** reproducible offline multi-package application.

## M3 — AX-HIR, AX-MIR, and ABI

- typed HIR;
- lowering to CFG MIR;
- ABI for values, effects, and capabilities;
- source mapping to compact frames;
- equivalence with the reference runtime.

**Gate:** same corpus on tree-walk and MIR.

## M4 — Rust Runtime and VM

- Rust workspace;
- decoder/validator;
- budgeted, cancellable VM;
- compatible AM, policy, approval, and audit;
- isolated capability adapters.

**Gate:** Python/Rust suite without divergences.

## M5 — Backend and Base Bridge

- native and WASM/WASI;
- Rust/C ABI;
- filesystem, processes, clock, random, and networking;
- versioned bridge system and target discovery.

**Gate:** same CLI on VM, native, and WASM.

## M6 — Application Backend

- HTTP client/server;
- routing and middleware;
- databases and transactions;
- serialization, config, and observability.
- Network Lab demo: HTTP/1.1-2, SSE, WebSocket, timeouts, cancellation, and backpressure;
- Syncboard backend demo: PostgreSQL, auth, OpenAPI, and OpenTelemetry.

**Gate:** deployable backend service written in AXL.

## M7 — Web

- DOM/Web APIs bridge;
- component model and state;
- WASM/browser build;
- storage, workers, and WebGPU.
- AX-UI semantic IR with DOM renderer;
- accessible, real-time UI Gallery and Syncboard Web demos.

**Gate:** full-stack web app primarily in AXL.

## M8 — Native, Mobile, and Graphics

- windowing, input, audio;
- GPU/rendering;
- desktop packaging;
- Android/iOS bridge;
- asset pipeline.
- Tauri 2 desktop bootstrap, explicitly declared as WebView;
- native SwiftUI and Jetpack Compose mobile renderers;
- builds/smoke tests on Windows, macOS, iOS, and Android runners.

**Gate:** cross-platform graphical app.

## M9 — Agentic Platform

- tasks, events, and DAG scheduler;
- structured concurrency;
- checkpoint/suspend/resume;
- model provider capability;
- semantic/vector/graph memory;
- distributed policies.

**Gate:** durable, auditable workflow.

## M10 — Ecosystem

- token/source and MIR optimizer;
- machine-oriented LSP;
- debugger, profiler, and tracing;
- package/bridge registry;
- progressive self-hosting.

## Rule

Every feature requires a canonical opcode, RED→GREEN tests, specification, source/IR round trip, a real example, and backend compatibility.
