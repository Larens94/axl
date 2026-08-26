# AXL vision

## The primary source

AXL — **Agent eXecution Language** — aims to be the language in which an agent
describes a complete system. Rust, React, SQL, and future bridges must not replace
AXL in source; they implement its semantic model.

```text
AXL intent
→ deterministic analysis
→ typed model
→ backend · frontend · data · runtime · bridges
```

## Why compact

Numeric syntax removes repeated keywords, implicit precedence, and decorative
structure. Compact does not mean single-line: the formatter lays frames out over
multiple lines for review and diffs, while minified form serves caching, hashing,
signing, and transport.

We do not claim token percentages or speed multipliers without a reproducible
benchmark. The demonstrated advantage today is structural: one AXL contract
generates several coherent targets.

## General-purpose goal

The architecture must be able to cover:

- backends, APIs, services, databases, and networking;
- web, desktop, and mobile frontends;
- CLIs, automation, systems, and IoT;
- AI, agents, workflows, memory, tools, and events;
- graphics, GPU, audio, and specialist applications.

## `0.1.0-alpha.1` status

### Available

- Rust runtime and CLI;
- Compact Source 2, RPN, AX-IR, functions, memory, agents, and workflows;
- application compiler for entities, APIs, auth, and seeds;
- Compact UI Source 3 and component/property registry;
- Rust/Axum/SeaORM, React/Refine/MUI/TanStack, and SQL/SQLite targets;
- responsive, tested full-stack CRM.

### Not yet guaranteed

- stable format compatibility across alpha releases;
- production-ready hardening and deployment;
- async scheduler and structured concurrency;
- WASM/native mobile/desktop targets;
- package manager, LSP, debugger, and public SDKs;
- real LLM backends and sandboxing for untrusted capabilities.

## Principles

1. **AXL first:** semantics live in AXL source.
2. **Agent native:** deterministic form and machine-readable diagnostics.
3. **Standard targets:** generated code uses real, inspectable ecosystems.
4. **Capability security:** external effects are explicit and bounded.
5. **Portability:** the language does not copy one framework's APIs.
6. **Evidence:** demos, tests, and benchmarks must be reproducible.
