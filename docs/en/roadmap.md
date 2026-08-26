# Roadmap

AXL evolves through demonstrable vertical slices. Every milestone requires AXL
source, runnable artifacts, tests, and declared limitations.

## Completed — `0.1.0-alpha.1`

- [x] Rust runtime and CLI;
- [x] Compact Source 2, RPN, and multiline formatter;
- [x] Compact UI Source 3 with component registry;
- [x] entities, APIs, auth, seeds, and query compiler;
- [x] Rust/Axum/SeaORM backend and SQLite migrations;
- [x] React/Refine/MUI/TanStack/Lucide frontend;
- [x] CRM with 6 entities, 30 CRUD operations, and 7 views;
- [x] responsive navigation, bottom menu, and table-to-card;
- [x] bilingual documentation and publishable presentation.

## M1 — Stabilize the application compiler

- application manifest and versioned target contracts;
- diagnostics with spans and stable error codes;
- relationships, enums, option/result, and form validation;
- generated OpenAPI and browser/API E2E suite;
- reproducible size, token, and generation-time benchmark.

**Gate:** the CRM regenerates from scratch and passes Rust, API, and E2E tests in CI.

## M2 — Production-oriented backend

- PostgreSQL, transactions, and pooling;
- authentication, RBAC, secret management, and rate limiting;
- SSE/WebSocket, bounded uploads, and async jobs;
- OpenTelemetry logging, metrics, and tracing;
- containers, health/readiness, and graceful shutdown.

**Gate:** deployable service with a verified threat model and observability.

## M3 — AX-UI beyond the web

- state, events, forms, accessibility, and semantic design tokens;
- stable DOM renderer and visual tests;
- desktop WebView bootstrap;
- native SwiftUI and Jetpack Compose adapters;
- Canvas/WebGPU renderer for specialist graphics.

**Gate:** one semantic UI tree passes conformance on two renderers.

## M4 — Agent runtime

- async scheduler and structured concurrency;
- versioned capability ABI, cancellation, and deadlines;
- real model backends, routing, and evals;
- semantic memory and external providers;
- audit and sandboxing for untrusted handlers.

**Gate:** reproducible multi-agent workflow with end-to-end policy and traces.

## M5 — Ecosystem

- AX-HIR/AX-MIR, VM, and WASM/native targets;
- package manager, registry, lockfile, and signing;
- LSP, formatter, debugger, profiler, and API docs;
- TypeScript, Python, C, IoT, and external-service SDKs and bridges.

**Gate:** reproducible packages and the same semantic suite on multiple runtimes.
