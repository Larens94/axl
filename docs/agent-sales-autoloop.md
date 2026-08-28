# AXL sales-module autoloop

Evolves toward an Odoo-like **Vendite** slice: backend + frontend in AXL.

## Rule

- Domain logic in AXL (flows, entities, api, ui).
- Missing language/runtime → **open primitive** in Rust (capacity/skill, UI IR, HTTP adapter).
- Never implement sales rules in Rust.

## Cycle

1. **Steward** — implement next primitive from `steward_queue` in `.cursor/judge-loop-state.json`
2. **Verify** — `scripts/verify-libro-cassa.sh` (regression) + `scripts/verify-sales.sh` when exists
3. **Judge** — extend `examples/apps/sales*.axl` per `docs/agent-judge-prompt.md`
4. **Steward** — test judge output, queue next primitive from judge report E
5. Repeat — do not stop at libro-cassa; target sales milestones below

## Sales milestones

| # | Goal |
|---|---|
| 1 | `Cliente` CRUD: store, API, **form** + list UI — form primitive + serve GET shipped (`form-demo.axl`) |
| 2 | `Prodotto` + list/form |
| 3 | `Preventivo` + righe + totali |
| 4 | Workflow stati (`bozza` → `inviato` → `confermato`) |
| 5 | Durable SQLite + single `serve` demo |
| 6 | Search/filter on lists |

## Files

- `examples/apps/sales-domain.axl` — domain module
- `examples/apps/sales.axl` — app import + api + ui
- `examples/apps/form-demo.axl` — minimal form + POST api + serve GET (until sales slice lands)
- `scripts/verify-sales.sh` — sales gate

See also `docs/agent-judge-loop.md` (libro-cassa experiment).
