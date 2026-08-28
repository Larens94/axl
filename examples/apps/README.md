# Cashflow executable core

`balance-ui.axl` is the minimal Gate 4 slice: one `ui` page bound to
`CalculateBalance`, lowered to Graph IR, emitted as `axl-ui/1` and rendered to
HTML through `axl-compiler render`.

`form-demo.axl` extends Gate 4 with a `form` bound to a POST api route,
`render_form` HTML inputs from entity fields, a navigation shell across pages and
forms, and `serve` GET responses as `text/html`.

`cashflow-core.axl` is the first AXL example that executes application behavior
instead of stopping at contracts. It implements eight deliberately narrow flows:

- `ValidateMovement` checks a typed movement kind and positive amount;
- `CalculateBalance` subtracts expenses from income using `money` arithmetic.
- `BuildMovementView` constructs a typed record with conditional and exhaustive match values;
- `CalculateLedgerBalance` folds a list of movements into one balance;
- `StoreAndLoadMovement` calls a generic in-memory store provider;
- `StoreAndLoadMovementSqlite` runs the same calls through SQLite;
- `StoreAndLoadMovementDocument` runs the same calls through the document/JSON store.
- `SaveDurableMovement` and `FindDurableMovement` reopen a configured SQLite file.
- `SaveDurableDocumentMovement` and `FindDurableDocumentMovement` reopen a configured JSON file.
- `ValidateAndStoreMovement` composes validation and storage flows.
- `IncomeAmounts` filters and maps the movement collection.

Run from the repository root:

```sh
cargo run -p axl-compiler -- \
  check examples/apps/cashflow-core.axl --json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateMovement \
  examples/apps/inputs/movement-invalid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateBalance \
  examples/apps/inputs/balance.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl BuildMovementView \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateLedgerBalance \
  examples/apps/inputs/movement-batch.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl IncomeAmounts \
  examples/apps/inputs/movement-batch.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl StoreAndLoadMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl StoreAndLoadMovementSqlite \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl StoreAndLoadMovementDocument \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ValidateAndStoreMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl SaveDurableMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-id.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl SaveDurableDocumentMovement \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableDocumentMovement \
  examples/apps/inputs/movement-id.json
```

Expected results:

- the valid movement is returned inside `{ "ok": ... }`;
- the zero amount returns `{ "error": "amount_must_be_positive" }`;
- the balance input returns `80000`.
- the movement view returns direction `Entrata` and signed amount `125000`;
- the ledger fold returns `80000`;
- the filter/map pipeline returns `[125000]`;
- the typed sort returns `movement-002` before `movement-001`;
- grouping creates `consulting` and `software` buckets with typed movements;
- the multiline list literal creates the default category list;
- the parallel flow builds movement views concurrently in source order;
- resilient lookup uses bounded retry and timeout on an idempotent operation;
- raced lookup ignores failed candidates and returns the first successful record;
- both replaceable storage providers return movement `movement-001`.
- two independent durable evaluations return movement `movement-001` from the same file;
- the composed validation/storage flow returns the valid movement.

The implemented expression operators are `!`, unary `-`, `*`, `/`, `+`, `-`,
comparison operators, equality, `&&` and `||`, with normal precedence and
parentheses. It also supports lazy `if ... then ... else ...` expressions and
multiline `make name: Entity` record construction. Flow Runtime 2 adds typed `in` dependencies and
`call value = dependency.operation(argument)?` with `Result` propagation.
`fold` provides immutable collection aggregation and `run` composes flows.

The `CashflowApi` declaration exposes `/movements`, `/movement-by-id`,
`/balance`, `/income-amounts`, `/movements/sorted`, `/movements/grouped`,
`GET /categories`, `/movement-views`, `/movement-by-id/resilient` and
`/movement-first`, `/movements/durable` and `/movement-by-id/durable` through the generic Axum runtime:

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

The server shares one provider runtime across requests. Memory state is
process-local. The durable routes use the path declared on
`DurableSqliteMovements` and survive a server restart.

`POST /secure/balance` belongs to a separate guarded API. It requires
`Authorization: Bearer axl-cashflow-demo`; missing and invalid credentials return
401 and 403. The credential is intentionally visible test data.

`POST /jwt/balance` uses the replaceable `CashflowDemoJwt` skill
(`axl::auth::jwt`). It validates an HS256 JWT with claims `sub` and
`iss=axl-cashflow` signed with demo secret `axl-cashflow-demo-jwt`. Missing
bearer → 401; bad token → 403; valid JWT → `80000`. Demo `secret`/`issuer` are
plaintext skill config (same honesty rule as the static bearer). True secret
references are Gate 8; OAuth is not implemented.

`POST /guarded/balance` uses ordered request middleware. It requires
`x-axl-client: cashflow-demo` and returns 403 when the header is missing or
wrong. The header-gate skill is a replaceable capacity, not route-specific Rust.

`POST /annotated/balance` uses ordered response middleware. It returns the same
balance body and sets `x-axl-middleware: ok` through the replaceable
`axl::middleware::response_headers` skill.

`POST /limited/balance` uses capacity-backed rate-limit middleware
(`RateLimit` / `MemoryRateLimit`, limit 5 per 60s). The first five requests
return `80000`; the sixth returns HTTP 429 with `rate_limit_exceeded`.

`POST /cors/balance` uses capacity-backed CORS middleware
(`CashflowCorsOrigin` / `CashflowCorsHeaders` via `axl::middleware::cors`).
Responses include `access-control-allow-origin: *` and configured allow-methods
/ allow-headers. `OPTIONS /cors/balance` returns 204 with the same headers
without running the balance flow.

The durable lookup is also exposed as `GET /movements/{id}` and
`GET /movements/find?id=...`. These routes bind a path or query string directly
to the typed `uuid` flow input; exact `/movements/find` matching takes precedence
over the `{id}` template.

`POST /accounts/{account}/movement-preview?dry_run=true` assembles a
`MovementPreviewRequest`: `account` comes from the path, `movement` is the full
JSON body and `dry_run` comes from the query. No Rust request DTO or extractor is
specific to this route.

`GET /me` and `GET /session` bind `header.x-user` and `cookie.sid` into a typed
`text` flow. `POST /client-preview` assembles `ClientSessionRequest` from
`header.x-user`, `cookie.sid` and the JSON body through the same open bind model.

`CacheBalanceSnapshot` stores a ledger balance key through the open `Cache`
capacity (`MemoryCache` / `DurableCache`). `POST /cache/balance`,
`/cache/balance/get` and `/cache/balance/invalidate` expose the memory skill;
durable put/get/invalidate survives process recreate via the configured SQLite
path. Values are typed text envelopes (`CacheEntry`), not cashflow-specific Rust.

`RecordTwoObservabilityLines`, `ObserveMetricTwice` and `TraceObservabilitySpan`
use open `Logger`, `Metrics` and `Tracer` capacities with memory skills. Two
structured log lines are listable; a named counter reaches `2`; a finished span
name is listable. HTTP routes under `/observability/*` share the process runtime.
No application-specific Rust logging is required.

`CommitTwoDurableMovements` / `RollbackTwoDurableMovements` use open
`TransactionManager` begin/commit/rollback with the same SQLite path as
`DurableSqliteMovements`. Commit survives a new runtime; rollback leaves both
ids `not_found`. Memory skills prove the same contract in-process.

`ApplyDurableMigration` / `RollbackDurableMigration` / `DurableMigrationStatus`
use open `MigrationRunner` up/down/status with the same SQLite path. Applying
`v1` then `v2` advances history; status survives runtime recreate; rolling back
`v2` returns head to `v1`. Memory skills prove the same contract in-process.

`QueryDurableMovements` / `QueryMovements` use store `query` with a typed
`MovementQuery` (filter map, order_by, direction, limit, offset) returning
`MovementPage`. Save N movements, filter by kind/category, order and page; the
durable SQLite path survives runtime recreate. Memory skills prove the same
contract in-process.

This is not yet the complete cashflow application. There are no
multi-database providers, state mutation or rendered UI.
Those missing capabilities must be added to AXL rather than implemented inside
this application with handwritten Rust or React.

## Libro cassa (`ledger.axl`)

Small income/expense book: domain module (`ledger-domain.axl`), HTTP API and UI.
Memory and durable SQLite routes share the same `ArchivioVoci` capacity.

`PaginaVociDemoUnit` seeds two demo voci with inline `make` (typed text→uuid,
int→money, text→datetime) and queries them in one `unit`-input flow — no JSON
pair payload:

```sh
cargo run -p axl-compiler -- \
  render examples/apps/ledger.axl /voci/demo examples/apps/inputs/unit.json

cargo run -p axl-compiler -- \
  eval examples/apps/ledger.axl PaginaVociDemoUnit examples/apps/inputs/unit.json

cargo run -p axl-compiler -- \
  render examples/apps/ledger.axl /saldo examples/apps/inputs/ledger-saldo.json

./scripts/verify-libro-cassa.sh
```

The JSON pair variant (`PaginaVociDemo` on `/voci`) remains for comparison:

```sh
cargo run -p axl-compiler -- \
  render examples/apps/ledger.axl /voci examples/apps/inputs/ledger-voci-demo.json
```

Expected results:

- `/voci/demo` render includes a `<table>` with `voce-001` (entrata 150000) and
  `voce-002` (uscita 42000), ordered by `registrato_il desc`, with `null` unit
  input only;
- `PaginaVociDemoUnit` eval returns `{ "ok": { "total": 2, "items": [...] } }`;
- `/saldo` render shows `108000`;
- verify script passes check, eval, render and UI manifest gates.

Durable SQLite routes (`/voci/durable`, `/voci/durable/query`) persist across
process restarts via `./build/libro-cassa.db`. The UI demo uses in-memory
`MemoriaVoci` only; each eval/render starts a fresh store unless the seed flow
registers voci in the same evaluation (as `PaginaVociDemo` does).

## Vendite (`sales.axl`)

Odoo-like sales slice: domain module (`sales-domain.axl`), HTTP API and UI for
`Cliente`, `Prodotto`, and `Preventivo` with workflow transitions. Memory routes
are process-local; durable SQLite routes share `./build/vendite.db` and survive
process restarts.

```sh
./scripts/demo-sales.sh

cargo run -p axl-compiler -- \
  eval examples/apps/sales.axl CreaCliente examples/apps/inputs/sales-cliente.json

cargo run -p axl-compiler -- \
  render examples/apps/sales.axl /clienti/demo examples/apps/inputs/unit.json

# Full browser demo (one serve session; UI GET shares the API memory store):
#
# 1. Create cliente (form POST → 303 to /clienti)
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8080/clienti \
  -H 'content-type: application/x-www-form-urlencoded' \
  -H 'accept: text/html' \
  --data-urlencode 'id=cliente-form-001' \
  --data-urlencode 'nome=Form Client' \
  --data-urlencode 'email=form@example.com' \
  --data-urlencode 'budget=5000' \
  --data-urlencode 'stato=attivo' | grep -i '^location: /clienti'
curl -s http://127.0.0.1:8080/clienti | grep 'Form Client'

# 2. Create preventivo (JSON POST)
curl -s -X POST http://127.0.0.1:8080/preventivi \
  -H 'content-type: application/json' \
  -d @examples/apps/inputs/sales-preventivo.json
# → {"ok":{"id":"preventivo-002",...,"stato":"bozza",...}}

# 3. Open templated detail page (list links href="/preventivi/{id}")
curl -s -H 'accept: text/html' http://127.0.0.1:8080/preventivi/preventivo-002 | grep 'preventivo-002'

# 4. Invia from detail UI (ui action → POST /preventivi/{id}/invia → 303 /preventivi/{id})
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8080/preventivi/preventivo-002/invia \
  -H 'content-type: application/x-www-form-urlencoded' \
  -H 'accept: text/html' \
  --data-urlencode 'id=preventivo-002' | grep -i '^location: /preventivi/preventivo-002'
curl -s -H 'accept: text/html' http://127.0.0.1:8080/preventivi/preventivo-002 | grep 'inviato'

# 5. Conferma (templated action submit)
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8080/preventivi/preventivo-002/conferma \
  -H 'content-type: application/x-www-form-urlencoded' \
  -H 'accept: text/html' \
  --data-urlencode 'id=preventivo-002' | grep -i '^location: /preventivi/preventivo-002'
curl -s -H 'accept: text/html' http://127.0.0.1:8080/preventivi/preventivo-002 | grep 'confermato'

# JSON API workflow (same session, path-param routes):
curl -s -X POST http://127.0.0.1:8080/preventivi/preventivo-002/invia | jq '.ok.stato'
curl -s http://127.0.0.1:8080/preventivi/preventivo-002 | jq '.ok.stato'

# Seeded demo list + templated detail (eval/render companion):
cargo run -p axl-compiler -- \
  eval examples/apps/sales.axl RenderDettaglioPreventivoDemoUnit examples/apps/inputs/unit.json
cargo run -p axl-compiler -- \
  render examples/apps/sales.axl /preventivi/preventivo-001 null
# render CLI uses a fresh store (no row data); serve GET after /preventivi/demo seeds in-session.

# Durable workflow survives restart:
curl -s -X POST http://127.0.0.1:8080/preventivi/durable \
  -H 'content-type: application/json' \
  -d @examples/apps/inputs/sales-preventivo.json
curl -s -X POST http://127.0.0.1:8080/preventivi/durable/preventivo-002/invia

./scripts/verify-sales.sh
```

**Preventivo detail UI** uses `page /preventivi/{id}` with `DettaglioPreventivo from path.id`.
List tables link each row `id` to `/preventivi/{id}`. Workflow buttons on the detail
page are `ui action` forms with templated submit paths (`POST /preventivi/{id}/invia`
and `/preventivi/{id}/conferma`), resolved at render time from the current page context
with hidden `id` inputs; `serve` returns `303` to `/preventivi/{id}` after success.

**Righe on detail pages:** `List<RigaPreventivo>` fields render as nested HTML tables
(`<table class="nested-table">` with `prodotto_id`, `quantita`, `prezzo_unitario`, `importo`
columns) on `/preventivi/{id}` and `/ordini/{id}` detail pages.

Expected results:

- `./scripts/demo-sales.sh` serves `examples/apps/sales.axl` on `127.0.0.1:8080`;
- memory eval/render gates pass for clienti, prodotti, preventivi (seeded pages at `/clienti/demo`, `/prodotti/demo`, `/preventivi/demo`);
- live list pages (`/clienti`, `/prodotti`, `/preventivi`) query the memory store; `GET /clienti` after form POST shows the new row in HTML (shared `BuiltinRuntime` in serve);
- form POST (`application/x-www-form-urlencoded`) creates `Cliente` via HTTP; `GET /clienti` and JSON query confirm the record in the same session;
- preventivo detail at `/preventivi/{id}` (templated path); seeded eval `RenderDettaglioPreventivoDemoUnit` + serve GET after `/preventivi/demo` show `preventivo-001`;
- workflow actions on detail POST to `/preventivi/{id}/invia` and `/preventivi/{id}/conferma` with `303` redirect to `/preventivi/{id}` (`accept: text/html`);
- list pages link `id` uuid fields to `/preventivi/{id}` (path template substitution);
- preventivo workflow (`bozza` → `inviato` → `confermato`) via JSON POST after create on memory and durable routes;
- `InviaPreventivo` renders a PDF stub and sends email via open `PdfRenderer` / `EmailSender` capacities (memory skills);
- `GET /secure/clienti` requires `Authorization: Bearer axl-vendite-demo` (401/403/200); `GET /jwt/preventivi/{id}` validates HS256 JWT (`iss=axl-vendite`);
- verify script passes check, eval, render, UI manifest, serve GET, auth smoke and durable gates.
