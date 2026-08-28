# Judge prompt — sales milestone 23 (listini)

Repo: AXL sales-module autoloop

Milestones 1–22 complete. Readiness **10/10**. See `docs/agent-sales-autoloop.md` and `.cursor/judge-loop-state.json`.

## Milestone 23: listini (price lists)

1. Model `Listino` + righe in AXL (`sales-domain.axl`) with store capacity
2. Wire listino lookup into `Prodotto` / preventivo righe pricing in AXL only
3. If store/query or money-composition primitive is missing → **stop**, report exact gap in steward queue
4. Extend `verify-sales.sh` with listino eval + HTTP smoke
5. Never implement pricing rules in Rust beyond open primitives

Report A–F. Run `verify-sales.sh` + `verify-libro-cassa.sh`.
