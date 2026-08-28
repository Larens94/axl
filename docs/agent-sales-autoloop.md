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
| 21 | Auth stub: `VenditeSecureApi` bearer + `VenditeJwtApi` HS256 JWT | done |
| 22 | PDF + email on `InviaPreventivo` (`PdfRenderer` + `EmailSender` capacities) | done |
| 23 | `Listino` + righe store + `RisolviPrezzo` / `CreaPreventivoConListino` pricing hook | done |
| 24 | Listini UI: `/listini/{id}` detail righe table + `/preventivi/new-listino` form via `CreaPreventivoConListino` | done |
| 25 | Listino durable SQLite HTTP restart + eval gates; nested `List<>` form **blocked** (steward queue) | done |
| 26 | Nested `List<>` form + dynamic listino picker | **deferred** (steward queue; flat listino form workaround remains) |

Milestones 1–25 complete. Readiness **10/10** for Odoo Vendite slice. **M26 closed deferred.** Auth IAM M27–M31: see `docs/agent-auth-autoloop.md` and `scripts/verify-auth.sh`.

## Files

- `examples/apps/sales-domain.axl` — domain module
- `examples/apps/portal.axl` — unified portal (IAM + vendite, single `PortalUi`)
- `examples/apps/form-demo.axl` — minimal form + POST api + serve GET (until sales slice lands)
- `scripts/verify-portal.sh` — portal gate (auth + vendite)

See also `docs/agent-judge-loop.md` (libro-cassa experiment).
