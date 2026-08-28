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

| # | Goal | Status |
|---|---|---|
| 1 | `Cliente` CRUD: store, API, **form** + list UI | done |
| 2 | `Prodotto` + list/form | done |
| 3 | `Preventivo` + righe + totali | done |
| 4 | Workflow stati (`bozza` → `inviato` → `confermato`) | done |
| 5 | Durable SQLite + single `serve` demo | done |
| 6 | Search/filter on lists | done |
| 7 | Form POST (`urlencoded`) + preventivo workflow via HTTP | done |
| 8 | Browser round-trip: form POST → live list refresh | done |
| 9 | Form POST redirect (`303` to list page) | done |
| 10 | Preventivo detail page + workflow forms in HTML | done |
| 11 | `ui action` primitive + table row links to detail | done |
| 12 | Live preventivo detail + workflow (non-demo routes) | done |
| 13 | UI page path templates `{id}` (`from path.id`) | done |
| 14 | README demo + render proofs for templated detail | done |
| 15 | Templated `ui action` submit (`POST /preventivi/{id}/invia`) | done |
| 16 | Dead `*Live*` flow cleanup + milestone doc | done |
| 17 | `Ordine` from confermato `Preventivo` (store, API, list UI, detail action) | done |
| 18 | Ordine workflow (`bozza`→`confermato`/`annullato`) + `/ordini/{id}` detail | done |
| 19 | Detail righe table on preventivo/ordine pages | done |
| 20 | Independent ordine ids (`uuid.v4` + dynamic verify) | done |

| 20 | Independent ordine ids (`uuid.v4`) | done |

Milestones 1–20 complete for the **Odoo Vendite slice** demo. Readiness **10/10**. Next wave: auth, PDF/listini — via open primitives.

## Files

- `examples/apps/sales-domain.axl` — domain module
- `examples/apps/sales.axl` — app import + api + ui
- `examples/apps/form-demo.axl` — minimal form + POST api + serve GET (until sales slice lands)
- `scripts/verify-sales.sh` — sales gate

See also `docs/agent-judge-loop.md` (libro-cassa experiment).
