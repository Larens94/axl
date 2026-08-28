# Judge prompt — sales milestone 25 (listino durable + form primitive)

Repo: AXL sales-module autoloop

Milestones 1–24 complete. Readiness **10/10**. See `docs/agent-sales-autoloop.md` and `.cursor/judge-loop-state.json`.

## Milestone 25: listino durable SQLite + dynamic righe form

1. Extend `verify-sales.sh` with listino durable HTTP restart (mirror preventivi/ordini pattern)
2. Document or prototype `ui form` nested `List<>` field rendering (repeatable righe rows) — if missing, report exact gap in steward queue E
3. Optional: listino picker (`select` from seeded list) on preventivo listino form
4. Never implement pricing rules in Rust beyond open primitives

Report A–F. Run `verify-sales.sh` + `verify-libro-cassa.sh`.
