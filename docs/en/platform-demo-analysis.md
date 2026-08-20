# Application Demo and Platform Analysis

[Italiano](../platform-demo-analysis.md)

> Analysis status: August 20, 2026. This document distinguishes implemented capabilities, prerequisites, and demonstrable targets. It does not claim that backends or bridges that do not yet exist are available.

## 1. Decision

AXL must not create six disconnected demos or delegate all logic to host wrappers. It must build a **shared vertical slice**:

```text
AXL Compact Source
→ frontend + types
→ application AX-HIR
→ AX-MIR/effect IR
→ Rust runtime
→ capability ABI
   ├─ HTTP/database backend
   ├─ browser DOM/WASM
   ├─ desktop WebView/native host
   └─ iOS SwiftUI / Android Compose
```

The first demonstration product will be **AXL Syncboard**: a collaborative board with local accounts, projects, tasks, real-time updates, offline cache, audit, and notifications. The same domain logic and API contract power all clients.

Two supporting demos ensure that the product does not conceal gaps:

1. **AXL UI Gallery** — components, tokens, adaptive layout, input, focus, accessibility, themes, and visual snapshots;
2. **AXL Network Lab** — HTTP client/server, streaming, SSE, WebSocket, timeouts, cancellation, backpressure, and reconnection.

## 2. Actual Baseline

| Area | Available today | Blocking gap |
|---|---|---|
| Source | Compact Source 2, canonical writer, legacy pack | new types and opcodes/compatible version |
| Core | `string`, `integer`, `boolean`, `list<T>`, bindings, flow, functions | bytes, float/decimal, option/result, map/record/enum |
| Modules | confined imports and namespaces | packages, manifest, lockfile, public exports |
| Runtime | synchronous, budgeted Python interpreter | async, tasks, streams, cancellation, Rust runtime |
| IR | AX-IR JSON 1.0/1.1/1.2 | HIR/MIR, effect IR, value/resource ABI |
| Capability | deny-by-default host tools, policy/audit | typed capabilities, resource handles, async lifecycle |
| Backend | none | sockets, HTTP, routing, DB, config, observability |
| UI | none | UI IR, state, events, layout, accessibility, renderer |
| Packaging | Python wheel/CLI | WASM, executable/app bundle, APK/AAB, Xcode app |

Conclusion: host adapters can be prototyped today, but they cannot yet be called “apps written in AXL.” A demo passes the gate only when structure, state, events, and application logic are represented in AXL/HIR; the host implements only the ABI and platform integration.

## 3. Language Prerequisites

### P0 — General-Purpose Value Algebra

Before UI and networking, the following are needed:

- `bytes`, `float`, or an explicit decimal type;
- `list<T>`, `map<K,V>`, tuples, and records;
- enums, `option<T>`, `result<T,E>`;
- structured errors;
- ownership/lifetimes for resource handles;
- deterministic JSON and byte serialization.

Without collections, there can be no request headers, task lists, UI trees, or typed database rows.

### P1 — Asynchronous Effects

The model must be semantic, not copied from Tokio, JavaScript, or Swift:

- `future<T>` and `stream<T>`;
- structured task groups;
- cancellation tokens and deadlines;
- bounded channels;
- backpressure;
- deterministic `select` where applicable;
- guaranteed resource cleanup;
- distinct timeout/cancel errors.

Tokio may implement the first Rust backend, but it must not appear in the grammar or public ABI.

### P2 — HIR, MIR, and Capability ABI

Minimum contract:

```text
capability-id
abi-version
input/output type-id
resource handles
future/stream result
required effects
limits + deadline + cancellation
platform targets
stable error code
```

Capabilities must be granular: `net.client`, `net.listen`, `db.query`, `ui.window`, `ui.notify`; not a global `network` or `system` grant.

## 4. AXL UI/UX Framework

### 4.1 Recommended Model

Create **AX-UI**, a renderer-independent declarative model:

```text
state + reducer/event
→ immutable semantic UI tree
→ keyed diff
→ renderer adapter
```

AX-UI owns:

- component trees and stable identity;
- state binding and one-way data flow;
- event dispatch;
- layout constraints/adaptive breakpoints;
- semantic design tokens;
- navigation and deep links;
- forms/validation;
- animation timelines;
- accessibility semantics;
- focus, keyboard, pointer, and touch;
- localization, RTL, dynamic type;
- test trees and semantic snapshots.

It must not own DOM, CSS, SwiftUI, Compose, WinUI, or GPU APIs. Those are renderers/bridges.

### 4.2 Renderers and Order

| Target | First renderer | Actual nature | Decision |
|---|---|---|---|
| Browser | DOM + CSS + Web APIs | native web | first UI target |
| Windows/macOS | Tauri 2 + system WebView | native binary, web UI | rapid desktop bootstrap |
| iOS | SwiftUI/UIKit adapter | native app and controls | canonical mobile target |
| Android | Jetpack Compose adapter | native app and UI | canonical mobile target |
| Advanced desktop | WinUI/AppKit/SwiftUI adapter | native controls | after AX-UI conformance |
| Graphics/games | `wgpu`/WebGPU renderer | custom GPU UI | not for the first app framework |

Tauri 2 declares support for Windows, macOS, Linux, Android, and iOS with Rust logic and a WebView frontend. It is suitable for quickly validating packaging and bridges, but **is not equivalent to native UI controls**. Therefore, it does not replace the SwiftUI and Compose renderers required by the mobile gate.

Compose Multiplatform is production-ready for mobile and desktop, while web is still described as beta. It may be an optional adapter or accelerator, not the semantic foundation of AX-UI: that would make Kotlin/Compose an architectural dependency of AXL.

### 4.3 V1 Components

`App`, `Window`, `Screen`, `Nav`, `Stack`, `Grid`, `Scroll`, `Text`, `Image`, `Button`, `TextField`, `Toggle`, `List`, `Dialog`, `Progress`, `Canvas`.

Each component exposes semantic and accessibility properties. Free-form CSS-like styling, SwiftUI APIs, or Compose modifiers do not enter canonical source; they are normalized into HIR tokens/layout.

## 5. Complete Backend

### 5.1 Bootstrap Stack

- Rust runtime: Tokio;
- HTTP/routing: Hyper + Axum + Tower;
- TLS: rustls or declared reverse-proxy termination;
- database: PostgreSQL in production, SQLite for local/test;
- pools and typed queries behind the `db.*` ABI;
- versioned migrations;
- structured logging, metrics, and OpenTelemetry traces;
- OpenAPI generated from the route/type model;
- graceful shutdown, health/readiness, and typed config;
- secrets resolved by the host, never in source/IR/audit.

These libraries are replaceable implementations. AXL defines HTTP semantics, requests/responses, routes, middleware, transactions, and streams.

### 5.2 “Complete Backend” Gate

The Syncboard backend must demonstrate:

- CRUD with PostgreSQL and migrations;
- transactions and constraints;
- session auth/OIDC-ready, with password hashing only through a host capability;
- RBAC per workspace;
- OpenAPI-documented JSON REST;
- SSE for the activity feed;
- WebSocket for bidirectional collaboration;
- bounded uploads and a content-type allowlist;
- pagination, idempotency keys, and rate limits;
- timeouts, cancellation, and graceful shutdown;
- correlated logs/metrics/traces;
- unit tests, DB integration tests, and end-to-end protocol tests;
- a reproducible Linux container.

## 6. Modern Networking

### Required V1

| Capability | Reason |
|---|---|
| DNS + TCP | portable foundation |
| TLS 1.3 and host trust store | secure transport |
| HTTP semantics | common contract for HTTP/1.1, 2, and 3 |
| HTTP/1.1 + HTTP/2 | immediate server/client interoperability |
| UTF-8 JSON | API and debugging baseline |
| WebSocket | widespread bidirectional real-time communication |
| SSE | simple server push, standard reconnection |
| streaming bodies | files/events without full buffering |
| timeouts/deadlines/cancellation | operational safety |
| backpressure and limits | prevent unbounded memory/queues |
| proxy, CORS, cookie, and redirect policy | real-world web operation |

### V2

- HTTP/3 over QUIC;
- Connect RPC and gRPC/Protobuf;
- CBOR for compact payloads;
- mTLS and service identity;
- controlled retry/circuit breaking/load balancing;
- WebTransport after an experimental adapter and with a fallback.

### Not Baseline

- MessagePack: useful but not an Internet standard; CBOR is an IETF standard and covers the same initial need;
- raw UDP exposed to apps: only as a specialized capability;
- GraphQL integrated into the grammar: a library/bridge, not core semantics;
- a proprietary real-time protocol when SSE/WebSocket suffice.

HTTP/3 is an IETF standard; as of August 20, 2026, WebTransport is still a W3C Candidate Recommendation, and the specification itself warns that APIs/protocols may change. It must therefore be isolated behind the ABI and not used as a requirement for the first demo.

## 7. Canonical Demos

### D1 — Network Lab

**Objective:** validate the async runtime and Capability ABI before the UI.

- server `/health`, `/echo`, `/stream`, `/events`, `/socket`;
- concurrent AXL client;
- HTTP JSON, streaming, SSE, and WebSocket;
- cancellation, deadlines, body limits, and reconnection;
- tests against the reference host and Rust runtime.

**Gate:** no route or state machine encoded in the Rust/Python wrapper; the wrapper only registers capabilities.

### D2 — Syncboard Web

- D1 backend extended with PostgreSQL;
- AX-UI → DOM web app;
- local development login;
- board CRUD and real-time updates;
- responsive/accessible, keyboard and screen reader support;
- PWA/offline cache;
- WASM/browser build when the runtime is ready.

**Gate:** Playwright end-to-end tests across two browser sessions; an update appears in real time and survives reload.

### D3 — Syncboard Desktop

- same domain and AX-UI;
- Tauri 2 host for Windows/macOS;
- file picker, secure storage, notifications, and deep links through capabilities;
- signable installer: MSIX/MSI and `.app`/DMG;
- updates disabled until signing and the supply chain are defined.

**Gate:** builds and smoke tests on Windows and macOS runners. Linux cannot certify bundles, signing, or OS behavior for both.

### D4 — Syncboard Mobile Native

- Rust runtime library with a stable ABI;
- Xcode + SwiftUI host on iOS;
- Gradle + Jetpack Compose host on Android;
- AX-UI semantic tree translated into native components;
- keychain/keystore, notifications, lifecycle, background policy, deep links;
- offline SQLite and incremental synchronization.

**Gate:** tests on an iOS simulator and Android emulator, plus at least one real device per platform; `.ipa` requires macOS/Xcode and Apple signing, AAB requires the Android SDK/Gradle and configured signing.

### D5 — UI Gallery

- component and design-token catalog;
- light/dark, locale, RTL, dynamic text;
- keyboard/focus/touch;
- accessibility tree assertions;
- golden screenshots per renderer.

**Gate:** same semantic corpus on DOM, SwiftUI, and Compose; differences allowed only if declared in the platform profile.

## 8. Target Repository Structure

```text
runtime/
  axl-core-rs/
  axl-vm/
  axl-component/
bridges/
  net-rust/
  db-sql/
  ui-dom/
  ui-tauri/
  ui-swiftui/
  ui-compose/
frameworks/
  ax-ui/
demos/
  network-lab/
  syncboard/shared/
  syncboard/backend/
  syncboard/web/
  syncboard/desktop/
  syncboard/ios/
  syncboard/android/
  ui-gallery/
conformance/
  source-ir-runtime/
  capability-abi/
  ax-ui/
```

## 9. Recommended Sequence

1. **Core types:** collections, records, option/result, bytes, errors.
2. **Async semantics:** future/stream/task/cancel/backpressure.
3. **HIR/MIR + ABI:** resource handles and typed effects.
4. **Rust tracer:** same Python/Rust corpus.
5. **Network Lab:** HTTP/1.1-2, JSON, SSE, WebSocket.
6. **Syncboard backend:** DB, auth, OpenAPI, observability.
7. **AX-UI + DOM:** UI Gallery and Syncboard Web.
8. **Tauri desktop:** Windows/macOS packaging and OS bridge.
9. **SwiftUI/Compose:** truly native mobile.
10. **WASM Component/WASI and HTTP/3:** conformance and optimization.
11. **WebTransport/gRPC/Connect/GPU:** later adapters, driven by real cases.

Do not start with all five targets simultaneously: that would multiply unstable adapters before values, async, ABI, and UI semantics are fixed.

## 10. Cross-Cutting Criteria

Every demo must have:

- canonical AXL source as its primary logic;
- reproducible build and a single command;
- zero credentials in the repository;
- deny-by-default capabilities;
- equivalent reference/Rust tests;
- end-to-end tests on the actual target;
- input/output/connection/queue limits;
- verified cancellation and cleanup;
- accessibility and keyboard/touch support where applicable;
- SBOM, lockfile, and artifact hash;
- documentation distinguishing “native binary,” “native UI,” and “WebView.”

## 11. Final Choice

The recommended strategy is hybrid and progressive:

- **Rust** under the hood for the runtime, async, networking, and bridges;
- **WASM/Component Model** for portability and sandboxing, without depending entirely on WASI;
- **AX-UI semantic IR** as the language's UI/UX framework;
- **DOM** as the first renderer;
- **Tauri 2** for desktop bootstrap;
- **SwiftUI and Jetpack Compose** as native mobile renderers;
- **wgpu/WebGPU** only for canvas, graphics, and a future custom renderer;
- **HTTP + SSE + WebSocket** as the baseline; HTTP/3, Connect/gRPC, CBOR, and WebTransport later.

This architecture produces real demos early without turning a temporary choice—WebView, Kotlin, Swift, Tokio, or Axum—into the AXL language.

## Primary Sources

- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110)
- [RFC 9114 — HTTP/3](https://www.rfc-editor.org/rfc/rfc9114)
- [RFC 6455 — WebSocket](https://www.rfc-editor.org/rfc/rfc6455)
- [WHATWG — Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [W3C — WebTransport](https://www.w3.org/TR/webtransport/)
- [WASI and WebAssembly Component Model](https://wasi.dev/)
- [Tauri 2 and architecture](https://v2.tauri.app/concept/architecture/)
- [Apple — SwiftUI](https://developer.apple.com/documentation/swiftui)
- [Android — Jetpack Compose](https://developer.android.com/compose)
- [Microsoft — WinUI 3](https://learn.microsoft.com/en-us/windows/apps/winui/winui3/)
- [Compose Multiplatform](https://kotlinlang.org/compose-multiplatform/)
- [Tokio](https://tokio.rs/) and [Axum](https://docs.rs/axum/latest/axum/)
- [OpenAPI 3.2](https://spec.openapis.org/oas/latest.html)
- [OpenTelemetry](https://opentelemetry.io/docs/specs/otel/)
- [RFC 8949 — CBOR](https://www.rfc-editor.org/rfc/rfc8949)
