# Judge prompt — sales milestone 24 (listini UI)

Repo: AXL sales-module autoloop

Milestones 1–23 complete. Readiness **10/10**. See `docs/agent-sales-autoloop.md` and `.cursor/judge-loop-state.json`.

## Milestone 24: listini UI integration

1. Extend `/preventivi/new` (or dedicated form) to build righe via `CreaPreventivoConListino` / listino picker
2. Listini list UI: row links to detail or inline righe table on `/listini/{id}` if path template exists
3. If nested form/list primitive is missing → **stop**, report exact gap in steward queue
4. Extend `verify-sales.sh` with listino form render + HTTP smoke
5. Never implement pricing rules in Rust beyond open primitives

Report A–F. Run `verify-sales.sh` + `verify-libro-cassa.sh`.
