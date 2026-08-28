# Judge prompt — sales milestone 7 (UI actions + form POST)

Repo: `/Users/fabriziocorpora/Desktop/workspaces/axl`

Judge agent. Milestones 1–6 done.

## Milestone 7

If steward added **form POST** handling in `serve` (application/x-www-form-urlencoded → JSON entity), prove:
- Browser-style POST to `/clienti` from form fields creates cliente via HTTP in same server session
- List page shows new record after POST+redirect or GET refresh

Add to sales:
- `page /preventivi/{id}` **only if** UI path templates exist — else add **action links** on list HTML via steward `ui action` primitive if available
- Or: flows `DettaglioPreventivo` + render page with buttons linking to POST `/preventivi/durable/{id}/invia` (document curl)

Steward may have added `serve` form POST decode — use it.

Extend verify-sales.sh with form POST smoke if supported.

Report A–F. No commit.
