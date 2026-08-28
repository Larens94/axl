# AXL research autoloop

Operational notes for the experimentation phase. Pairs with `AGENTS.md` and
`docs/agent-work-packages.md`.

## Active focus

Gate 2 remainder → Gate 3 data → Gate 4 UI → Gates 5–9.

Immediate next executable slice: **Gate 3 PostgreSQL/MySQL or document tx/migrate**
(WP-03). Document/JSON-file store (`rust::axl::store::document`) shares
save/find/query with memory and SQLite. Gate 3 typed queries, transactions and
migrations are executable. Gate 2 auth adapters (static bearer + HS256 JWT) are
complete; true secret references are Gate 8; OAuth remains optional WP-02
remainder.

## Autoloop

Two loops may run locally:

1. **Gate autoloop** — failing example → open primitive → proofs (see checklist below).
2. **Judge autoloop** — steward improves language → judge agent builds libro-cassa in AXL →
   steward verifies (`scripts/verify-libro-cassa.sh`). See `docs/agent-judge-loop.md`.

A local session loop wakes about every **5 minutes** to advance the active loop.
Prefer one ABI-touching package at a time (serial on `ast`/`parser`/`analyzer`/`packed`).

## Agent roster

| Id | Role | Owns |
|---|---|---|
| WP-01 | Language steward | AST, parser, analyzer, Packed IR, IR schema |
| WP-02 | Backend | HTTP, middleware, events, jobs, observability |
| WP-03 | Data | stores, transactions, migrations, multi-DB |
| WP-04 | Frontend | UI IR, React renderer |
| WP-05 | AI / vector | embeddings, RAG, streaming |
| WP-06 | Agents | tools, plans, approval, traces |
| WP-07 | IoT | devices, telemetry, commands |
| WP-08 | Security / packages | secrets, deploy, conformance |
| WP-09 | QA / docs | evidence, film, mondo, release checklist |

Only WP-01 may merge conflicting edits to shared ABI files. Vertical agents
propose syntax; the steward assigns opcodes and diagnostic ranges.

## Iteration checklist

1. Pick the next row from `docs/status.md` / roadmap.
2. Author a red AXL example under `examples/`.
3. Implement the smallest open primitive.
4. Negative diagnostics + IR round-trip + e2e proof.
5. Update SPEC, status, testing, presentation, film/mondo.
6. Leave an ABI note for WP-01 when kinds/opcodes change.

Protocol steps **1–5** are satisfied for the agent authoring base: failing
examples, open primitives, typed capacities, stable `axl-check/1` diagnostics
(parse/analyze/import/UI) and IR round-trip proofs through
`documented_examples.rs`.
