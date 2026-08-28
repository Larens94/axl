# Judge prompt — sales milestone 14

Repo: `/Users/fabriziocorpora/Desktop/workspaces/axl`

UI path templates work. Sales still uses `/preventivi/live/invia` for actions because `ui action` submit must be static.

## Milestone 14

1. If steward allows action submit with path template OR hidden id field on detail page — wire `POST /preventivi/{id}/invia` from detail UI
2. Else: document live/* pattern in README; add eval proof `render /preventivi/preventivo-001` after seed flow in same eval (new DemoUnit)
3. Update `examples/apps/README.md` with full browser demo: create cliente → create preventivo → open `/preventivi/{id}` → invia → conferma
4. Remove obsolete `/preventivi/detail` if still present in domain

verify-sales.sh. Report A–F. No commit.
