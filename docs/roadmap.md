# AXL autoloop roadmap

The autoloop advances only through executable gates. Every iteration begins
with an AXL example that cannot yet compile or run, implements the smallest
general primitive, adds negative diagnostics and updates schemas, tests,
documentation and the presentation.

| Gate | Scope | Exit evidence |
|---|---|---|
| 0 | universal provider ABI and typed capacity calls | complete in Flow Runtime 2 |
| 1 | records, branch/match, collections, loops and flow calls | complete cashflow domain behavior |
| 2 | HTTP, auth, middleware, events, jobs, cache and observability | executable backend API |
| 3 | durable SQLite, PostgreSQL, MySQL, document/key-value and migrations | same app switches database provider |
| 4 | UI IR and React renderer for routing, forms, tables and responsive admin UI | desktop/mobile application |
| 5 | embeddings, vector stores, model providers, streaming and RAG | provider-neutral AI knowledge app |
| 6 | tools, memory, goals, plans, approval and traces | executable agent workflows |
| 7 | device registry, telemetry, commands, MQTT/HTTP/WebSocket and edge rules | simulated IoT control center |
| 8 | packages, secrets, deployment, security and conformance SDK | external provider package passes suite |
| 9 | CRM, cashflow, AI and IoT reference applications | end-to-end logic authored in AXL |

## Current position

Gate 0 is implemented: capacity ports and calls are type checked, lowered to
Graph/Packed IR and executed through a replaceable ABI. Memory and SQLite
adapters prove two implementations behind the same `MovementStore` capacity.

Gate 1 is complete for the executable foundation. Typed multiline records, lazy conditionals, functional `fold`
loops, flow-to-flow `run` calls, enum `match`, `map`, `filter`, stable typed
`sort`, typed `group` and inferred multiline list literals are implemented.
`parallel` executes forked provider runtimes concurrently and preserves source
order. Idempotent `attempt` adds bounded retry and real deadlines; `race`
returns the first successful idempotent flow. The first Gate 3 slice is now
complete: typed skill configuration reaches the provider ABI and a SQLite path
survives independent runtime instances.

Gate 2 has started without closing Gate 1: `api` declarations now compile to
`axl-http/1` and execute through a generic Axum server. Exact JSON routes and
status mapping work. One process-local provider runtime is shared across
requests. A typed `auth bearer: Capacity = Provider` surface now protects whole
APIs and the built-in fixture proves 401/403/200 behavior. Secret references,
JWT/OAuth and response middleware remain. Scalar and composite body/path/query/
header/cookie assembly is executable; ordered request middleware is capacity-backed.
Typed events and multi-subscriber `emit` are executable. Durable/scheduled jobs with
retry, idempotency and replaceable JobStore providers are executable across
runtime recreate. Cache and observability remain. Gate 3
continues with transactions, migrations and additional database families.
