# AXL agent work packages

This is the operational companion to `film.html` and
`film/axl-plan-film.mp4`. It lets specialized agents work from repository
evidence instead of interpreting the film as a specification.

## Non-negotiable protocol

Every package follows the same sequence:

1. add an AXL example that expresses the missing capability and initially fails;
2. implement a general language/runtime primitive, never application-specific
   Rust or React logic;
3. keep the primitive open through typed capacities, skills, slots, hooks,
   parameters or policies;
4. add stable negative diagnostics and repair information where useful;
5. prove readable AXL → AST → Graph IR → Packed IR round-trip;
6. add a manifest/schema when another runtime must consume the capability;
7. run formatting, unit, integration and Clippy gates;
8. perform a real end-to-end runtime check;
9. update the specification, status, testing handoff, presentation and film.

If AXL lacks a required primitive, stop at the AXL boundary and report the
missing syntax, type rule, IR node and runtime behavior. Do not hide the missing
feature in handwritten application Rust, React or another target language.

## Coordination order

| Wave | Work packages | Dependency |
|---|---|---|
| A | Language stewardship, backend completion | current Graph IR |
| B | data runtime, UI IR | stable provider/config ABI and request/event contracts |
| C | AI/vector, agent runtime, IoT runtime | package/provider conformance from waves A–B |
| D | security/ecosystem, reference applications, QA | all executable vertical slices |

Agents should work on separate branches or worktrees. Changes to `ast.rs`,
`parser.rs`, `analyzer.rs`, `packed.rs` and shared schemas must be integrated
serially by the language steward because these files define the common ABI.

## WP-01 — Language and Graph IR steward

Goal: preserve one coherent language while vertical agents add primitives.

Primary files:

- `runtime/axl-compiler/src/next/{ast,parser,analyzer,formatter,packed}.rs`;
- `schema/axl-ir-4.0.schema.json`;
- `SPEC-4.0.md`.

Deliverables:

- review and normalize proposed syntax;
- assign stable node kinds/opcodes and diagnostics;
- preserve canonical formatting and lossless Packed IR decoding;
- reject target-specific concepts that do not belong in the semantic graph;
- publish compatibility notes when a manifest shape changes.

Exit evidence: every documented valid program round-trips exactly, every new
invalid program exposes stable codes, and no runtime feature depends on parsing
source text after compilation.

## WP-02 — Backend runtime

Goal: complete Gate 2 without handwritten controllers.

Starting evidence: Axum routes, body/path/query composite binding, capacity-based
bearer auth, ordered request middleware, typed events/subscriptions,
durable/scheduled jobs with retry, and durable SQLite are executable.

Next deliverables:

- response-phase middleware and response header mutation;
- header and cookie request bindings;
- cache and invalidation capacities;
- tracing, metrics and structured logs;
- CORS, rate-limit and production auth adapters behind capacities.

Exit evidence: a server restart preserves durable state/jobs, middleware can be
replaced without changing routes, events reach two consumers, and all HTTP
status/header/body behavior is tested on a real listener.

## WP-03 — Data and multi-database runtime

Goal: complete Gate 3 with portable data semantics.

Deliverables:

- typed repository query/filter/order/page contracts;
- transaction block with commit and rollback proof;
- versioned migrations and schema history;
- SQLite, PostgreSQL and MySQL providers behind the same capacities;
- document/key-value provider contract;
- pooling, health, timeout and tenant/namespace configuration;
- deterministic test adapters independent of external infrastructure.

Exit evidence: the same AXL application switches providers only through skill
bindings/config, passes one conformance suite, and proves rollback plus migration
upgrade/downgrade behavior.

## WP-04 — UI IR and React runtime

Goal: complete Gate 4 and restore the CRM as an executable UI demonstration.

Deliverables:

- UI node model for layout, route, page, component, slot and interaction;
- state/query/mutation binding to typed capacities and flows;
- React renderer with replaceable component registry;
- responsive shell with desktop sidebar and mobile bottom navigation;
- forms, validation, tables, filters, pagination, detail, modal and drawer;
- KPI, chart, timeline, activity and loading/empty/error states;
- theme, accessibility, keyboard and density parameters;
- at least 70% coverage of the agreed admin/CRM UI foundation kit.

Exit evidence: desktop and mobile visual tests, keyboard/accessibility checks,
real backend integration and a CRM whose application behavior remains in AXL.

## WP-05 — AI and vector runtime

Goal: complete Gate 5 with provider-neutral knowledge workflows.

Deliverables:

- model, embedding and vector-store capacities;
- document ingestion, chunking, metadata and namespace contracts;
- upsert/delete/search with typed filters, top-k and score;
- streaming generation and structured output schema;
- RAG block with observable retrieval/generation trace;
- cache, budget, retry, fallback and model routing policies;
- in-memory deterministic vector adapter plus at least one external adapter.

Exit evidence: the same AXL RAG application swaps embedding, vector and model
providers independently, and retrieval quality plus failure paths are evaluated.

## WP-06 — Executable agent runtime

Goal: turn belief/goal/plan graph nodes into Gate 6 execution.

Deliverables:

- typed tool invocation through capacities;
- short/long-term memory providers;
- goal, plan, step and handoff state machine;
- approval gates and resumable runs;
- token/time/cost/tool budgets;
- trace, replay and deterministic test policy;
- multi-agent delegation without implicit privilege expansion.

Exit evidence: an agent completes a bounded workflow, pauses for approval,
resumes after restart, delegates one typed task and produces a replayable trace.

## WP-07 — IoT and edge runtime

Goal: complete Gate 7 with one portable device graph.

Deliverables:

- device model, registry, credential and digital-twin state;
- telemetry ingestion and typed time-series events;
- command/acknowledgement lifecycle;
- MQTT, HTTP and WebSocket provider adapters;
- offline queue, reconnect, deduplication and idempotency;
- edge rule, threshold, window and alert primitives;
- simulated device laboratory for deterministic tests.

Exit evidence: simulated devices stream telemetry, an AXL rule issues a command,
disconnect/reconnect preserves delivery semantics and the dashboard updates live.

## WP-08 — Packages, security and deployment

Goal: complete Gate 8 so openness remains enforceable in production.

Deliverables:

- package manifest, semantic versions, dependency resolution and lockfile;
- signed provider packages and registry metadata;
- secret references that never enter Graph/manifest plaintext;
- runtime enforcement for effects and capabilities;
- provider sandbox and network/filesystem policies;
- deployment manifest, health/readiness and rollback;
- conformance SDK for third-party providers.

Exit evidence: an external sample provider passes the SDK, cannot exceed granted
capabilities, resolves secrets only at runtime and deploys with a reversible plan.

## WP-09 — QA, documentation and reference applications

Goal: keep every claim coupled to executable evidence and complete Gate 9.

Deliverables:

- conformance matrix covering compiler, runtime and every provider family;
- CRM, cashflow, AI knowledge and IoT control-center applications;
- token/line/latency/size benchmarks against explicit Rust/React baselines;
- mobile/desktop visual regression and API end-to-end suites;
- versioned docs, agent handoff, presentation and narrated film;
- release checklist that distinguishes experimental and production-ready gates.

Exit evidence: one command builds and verifies all reference applications; every
presentation claim links to a test, manifest or runtime recording.

## Prompt template for a specialized agent

```text
Work on AXL package <WP-ID> only. Read SPEC-4.0.md, docs/status.md,
docs/roadmap.md, docs/agent-testing.md and docs/agent-work-packages.md first.

Start from a failing AXL example. Implement only general open primitives. Do
not move application logic into Rust, React or another target. If the language
lacks something, stop and report the exact missing syntax, type rule, IR node,
manifest field and runtime behavior.

Before handoff, prove formatting, positive/negative tests, exact Graph/Packed IR
round-trip, schema validity and a real end-to-end scenario. Update the spec,
status, testing guide, presentation and film narration. Report changed files,
commands, outputs, remaining boundary and any ABI decision required from WP-01.
```
