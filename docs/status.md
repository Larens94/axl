# AXL 4 implementation status

This table is the short source of truth for the current experiment.

| Area | Implemented now | Not implemented yet |
|---|---|---|
| Source | multiline AXL, typed list literals, transforms, flows, enum match, `parallel`, idempotent `attempt` and `race` | mutable statement blocks intentionally deferred |
| Types | built-ins, entities, enums, capacities, recursive generics | tuples and record operation parameters |
| Blocks | open protocol, typed instances/overrides, foundation contracts (incl. transactions and migrations), relative file imports | cross-package overlays, package registry and lockfile |
| Contracts | `requires`, `ensures`, `invariant` stored in IR | expression type checking and execution |
| Safety | diagnostics (`axl-check/1` JSON envelope, file paths, repair candidates, safety levels) | automatic application of risky repairs |
| Policies | effects and capabilities validated and stored | runtime budgets and enforcement |
| Agents | belief/goal/plan graph model | planning and execution runtime |
| Runtime | records, transforms, `parallel`, `race`, retry/timeout, flow/capacity calls, typed `emit`/subscriptions, jobs (`enqueue`/`tick`), `Result` propagation, forkable configured provider ABI and HTTP | state; Gate 4 timeline polish |
| Storage | generic memory, SQLite, PostgreSQL and MySQL (store + tx + migrate + **SQL pushdown query**) and document/JSON-file providers (store + tx + migrate); typed durable paths; capacity-backed transactions; capacity-backed migrations; typed store `query` | document pooling; health/timeout |
| Backend | scalar and composite body/path/query/header/cookie binding, Axum, typed bearer auth (static + HS256 JWT skills), **OAuth demo provider** (`rust::axl::auth::oauth` — `authorize_url`/`exchange`; see `oauth-boundary.axl`), **HTTP redirect routes** (`redirect text` / `redirect LoginResult`), ordered request and response middleware, **per-route guards** (`session`/`can`/`guest` → AXL flows), capacity-backed rate-limit (`allow` → 429), capacity-backed CORS (`Access-Control-*` + OPTIONS preflight), typed events/subscriptions, capacity-backed jobs, Cache get/put/invalidate (memory + durable SQLite), Logger/Metrics/Tracer observability (memory) and durable SQLite; memory `EmailSender` send/list/**latest** and `PdfRenderer` render/get stubs; password reset via `EmailSender` (token only in mailbox); **portal production SQLite** (auth + vendite APIs); SQLite **`find_by`** on store capacities; **Gate 8 secret refs** (`secret("ENV")` → runtime env, redacted in IR/manifest) | OAuth HTTP redirect/callback routes; typed `OAuthStart`/`OAuthToken` records |
| Targets | Rust/React/SQL contracts plus agent, block, flow, HTTP, UI and provider manifests; **React routes/layouts/registry** from `axl-ui/1`; **Vite host** `hosts/portal-web` (same-origin cookie proxy) | executable full-stack application generation |
| IR | canonical JSON graph, packed opcode round-trip | stable compatibility guarantee |

## Evidence

- The compiler unit and integration tests validate parsing, semantics,
  diagnostics (`axl-check/1` envelope with stable codes and spans), IR
  determinism, packing and target adapters.
- Every sample in `examples/blocks`, the CRM graph, the executable cashflow
  core and `examples/apps/import-demo.axl` (multi-file import) is compiled by
  `documented_examples.rs`.
- `cargo clippy --workspace --all-targets -- -D warnings` is the lint gate.

The project remains an experiment. AXL flows call replaceable capacities; the
same cashflow graph executes against memory and SQLite. A runtime test saves to
a configured SQLite file, destroys the runtime and reads through a new runtime.
Typed events reach multiple subscribers. Capacity-backed jobs enqueue, claim and
retry through memory or durable SQLite stores (`axl-compiler tick`). Header and
cookie request bindings share the same open bind model as path/query/body.
Cache put/get/invalidate works through memory and durable SQLite skills; a
configured cache path survives runtime recreate. Logger write/list, Metrics
increment/get and Tracer start/finish/list work through memory skills; cashflow
proves two structured lines and counters via eval and HTTP. Capacity-backed
rate-limit middleware proves N allowed requests then HTTP 429 on the cashflow
demo route. Capacity-backed CORS middleware proves `Access-Control-Allow-Origin`
on `/cors/balance` and OPTIONS preflight 204. Capacity-backed HS256 JWT auth
(`axl::auth::jwt`) proves 401/403/200 on `/jwt/balance` with demo `secret`/
`issuer` config. True secret references (no plaintext in IR) remain Gate 8;
OAuth demo provider `rust::axl::auth::oauth` is executable (`authorize_url` /
`exchange` on `oauth-boundary.axl`); **HTTP redirect routes** (`redirect text`,
`redirect LoginResult` + session cookie) and portal `/auth/oauth/*` are proven.
Gate 2 auth-adapter slice is otherwise complete. Gate 3
transactions are executable: `TransactionManager` begin/commit/rollback with
memory and SQLite skills; SQLite commit survives runtime recreate and rollback
hides both writes. Gate 3 migrations are executable: `MigrationRunner`
up/down/status with memory and SQLite skills; SQLite history and version marker
tables survive runtime recreate; `down` rolls back one head version. Gate 3 typed
queries are executable: store `query` with filter/order/page over memory and
SQLite; durable SQLite pages survive runtime recreate. Gate 3 document/JSON-file
store is executable: the same `MovementStore` save/find/query contract runs
through `rust::axl::store::document` with durable `config path`; cashflow and
conformance tests switch Memory, Sqlite and Document by skill binding only.
Next Gate 3 target: MySQL and document tx/migrate behind the same
capacities. Gate 3 PostgreSQL and MySQL store/tx/migrate are executable with
`config url` (`AXL_POSTGRES_URL` / `AXL_MYSQL_URL`); document tx/migrate are
executable via `axl::tx::document` and `axl::migrate::document` with durable
`config path` sidecars; see `postgres-boundary.axl`, `mysql-boundary.axl`,
`document-tx-boundary.axl` and `sql-pushdown-boundary.axl`. Store `query` on
SQLite/PostgreSQL/MySQL uses SQL pushdown for equality `filter`, `order_by` and
`limit`/`offset` over JSON payload fields.
Gate 4 foundation is executable: `ui` / `page` / `form` / **`drawer`** / **`modal`** /
**`kpi`** / **`chart`** / **`slot`** lower to Graph IR, emit `axl-ui/1` (with `kit.slots` + shell), and
`render` / `serve` produce HTML with sidebar + mobile bottom nav, KPI dashboards,
bar charts, overlays and error/empty states (`balance-ui.axl`, `form-demo.axl`,
`drawer-boundary.axl`, `modal-boundary.axl`, `bottom-nav-boundary.axl`,
`kpi-registry-boundary.axl`, `chart-boundary.axl`). Form POST failures re-render HTML with
field errors (`form-validation-boundary.axl`); portal clienti use a detail **drawer**.
Remaining Gate 4 polish: timeline/activity widgets.
