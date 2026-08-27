# AXL research autoloop

Operational notes for the experimentation phase. Pairs with `AGENTS.md` and
`docs/agent-work-packages.md`.

## Active focus

Gate 2 remainder → Gate 3 data → Gate 4 UI → Gates 5–9.

Immediate next executable slice: **cache / observability** (WP-02), then Gate 3
data.

## Autoloop

A local session loop wakes about every **5 minutes** to advance the next failing
example → open primitive → proofs → docs. Prefer one ABI-touching package at a
time (serial on `ast`/`parser`/`analyzer`/`packed`).

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
