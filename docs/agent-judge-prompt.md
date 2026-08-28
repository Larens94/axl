# Judge prompt — iteration 5 (unit seed flow)

Repo: `/Users/fabriziocorpora/Desktop/workspaces/axl`

You are the **judge agent**.

## Steward change (new)

Analyzer now allows typed `make` assignments:
- `text` literal → `uuid`, `datetime`, `email`, `duration`
- `int` literal → `money`, `float`

Runtime already accepts these at eval time.

## Milestone 5

Add **`PaginaVociDemoUnit unit -> Result<VocePage>`** in `ledger-domain.axl` that:
1. Builds two demo voci with inline `make` (no JSON pair input)
2. Saves both via store in the same flow
3. Queries with a default filter/order inside the flow

Wire a **second UI page** `/voci/demo` to this flow OR switch `/voci` demo to accept `unit` input if the UI binder supports it.

Prove:

```sh
cargo run -p axl-compiler -- check examples/apps/ledger.axl --json
cargo run -p axl-compiler -- eval examples/apps/ledger.axl PaginaVociDemoUnit null
cargo run -p axl-compiler -- render examples/apps/ledger.axl /voci/demo null
./scripts/verify-libro-cassa.sh
```

Update README one-liner if the demo command changes.

If UI cannot bind `unit` input, report the exact missing syntax and STOP.

Report A–F. Do not commit.
