# Agent testing handoff

This guide lets another agent verify the implemented AXL Open Block Protocol
without relying on conversation history.

## 1. Verify the repository

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
jq empty schema/axl-ir-4.0.schema.json
jq empty schema/axl-open-block-2.schema.json
jq empty schema/axl-flow-2.schema.json
jq empty schema/axl-http-1.schema.json
jq empty schema/axl-ui-1.schema.json
jq empty schema/axl-provider-1.schema.json
jq empty schema/axl-check-1.schema.json
```

The integration suite compiles ten valid documented programs, round-trips each
through Packed Graph IR, verifies twenty-seven intentionally invalid programs,
and proves multi-file import through `examples/apps/import-demo.axl`.

The foundation program `examples/catalog/software-foundation.axl` contains
fourteen primary open blueprint contracts and must compile as application
`SoftwareFoundation`.

## 2. Structured check diagnostics (`axl-check/1`)

`check` and `diagnose` are aliases. With `--json`, both success and failure
write a single JSON document to **stdout** using protocol `axl-check/1`
(schema: `schema/axl-check-1.schema.json`). Agents should parse stdout only.

The input path and `--json` may appear in either order:

```sh
cargo run -p axl-compiler -- check examples/apps/ledger.axl --json
cargo run -p axl-compiler -- check --json examples/apps/ledger.axl
```

Success example:

```json
{
  "protocol": "axl-check/1",
  "ok": true,
  "path": "examples/apps/cashflow-core.axl",
  "app": "CashflowCore",
  "schema": "ax-ir/4.0",
  "nodes": 87,
  "edges": 142
}
```

Failure example (parse, analyze, import and UI phases):

```json
{
  "protocol": "axl-check/1",
  "ok": false,
  "path": "examples/invalid/flow-calls.axl",
  "diagnostics": [
    {
      "code": "AXL-X817",
      "phase": "execution",
      "severity": "error",
      "message": "call 'store.find' receives the wrong argument type",
      "path": "examples/invalid/flow-calls.axl",
      "span": { "line": 39, "column": 1, "length": 35 },
      "expected": "uuid",
      "found": "Movement",
      "fix_safety": "manual"
    }
  ]
}
```

Each diagnostic carries a stable `code`, human `message`, optional source `path`,
1-based `span`, optional `expected`/`found`, optional `hint`, `fix_safety` and
`repairs` (connect/replace candidates). Phases: `parse`, `imports`, `names`,
`types`, `ports`, `execution`, `http`, `ui`, `compact`.

Quick probe:

```sh
cargo run -p axl-compiler -- \
  check examples/invalid/flow-calls.axl --json | jq '.protocol,.ok,(.diagnostics|length)'
```

Expected: `"axl-check/1"`, `false`, and a positive diagnostic count.

## 3. Compile the complete open block

```sh
cargo run -p axl-compiler -- \
  check examples/blocks/05-open-dataview.axl --json
```

The command must return `"ok": true` with `"protocol": "axl-check/1"` for
application `OpenDataViewBlock`.

## 4. Inspect the open surfaces

```sh
cargo run -p axl-compiler -- \
  blocks examples/blocks/05-open-dataview.axl

cargo run -p axl-compiler -- \
  experiment examples/blocks/05-open-dataview.axl /tmp/axl-open-block-test

jq '.protocol, .blocks[0].open_surface_count, .blocks[0].surfaces' \
  /tmp/axl-open-block-test/targets/blocks/open-blocks.json
```

Expected protocol: `axl-open-block/2`. The `CustomerDataView` block currently
contains twelve typed surfaces, nine of which are direct customization surfaces.
The other three are observable `state`, `event` and `error` surfaces.

## 5. Verify a typed instance override

```sh
cargo run -p axl-compiler -- \
  check examples/blocks/06-instance-override.axl --json

cargo run -p axl-compiler -- \
  blocks examples/blocks/06-instance-override.axl \
  | jq '.instances[0]'
```

The resolved manifest entry must name blueprint `CustomerList`, contain two
settings (`page_size`, `density`) and bind `table.row` to
`CompactCustomerRow`.

## 6. Execute the cashflow core

```sh
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
  eval examples/apps/cashflow-core.axl SaveAndAnnounce \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ScheduleDurableMovementPersist \
  examples/apps/inputs/movement-valid.json

cargo run -p axl-compiler -- \
  tick examples/apps/cashflow-core.axl

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-id.json

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

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CacheBalanceSnapshotDurable \
  examples/apps/inputs/balance-cache.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl LoadCachedBalanceDurable \
  examples/apps/inputs/balance-cache-key.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl InvalidateCachedBalanceDurable \
  examples/apps/inputs/balance-cache-key.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl LoadCachedBalanceDurable \
  examples/apps/inputs/balance-cache-key.json
```

The first results must respectively contain an `ok` movement, the error
`amount_must_be_positive`, the integer `80000`, a view with direction `Entrata`
and a folded ledger balance of `80000`. The storage evaluations must return
movement `movement-001`; the composed flow must validate before saving.
`SaveAndAnnounce` saves then emits `MovementSaved` to two AXL listeners.
`ScheduleDurableMovementPersist` enqueues a durable job; `tick` in a fresh
process runs it through `SaveDurableMovement`, and the following
`FindDurableMovement` must still find `movement-001`. Durable cache put/get
returns `"80000"` for key `ledger:demo` across processes; invalidate returns
`true`, then get yields `cache_miss`. No application-specific Rust function
contains these rules. The durable movement commands run in independent
processes and must still find `movement-001`, proving that the configured
SQLite path survives a runtime restart. The same save/find pattern through
`SaveDurableDocumentMovement` / `FindDurableDocumentMovement` proves the
document JSON path survives independently.

Verify observability (memory skills; two writes listable in one eval):

```sh
cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl RecordTwoObservabilityLines \
  examples/apps/inputs/unit.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl ObserveMetricTwice \
  examples/apps/inputs/unit.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl TraceObservabilitySpan \
  examples/apps/inputs/unit.json
```

`RecordTwoObservabilityLines` returns
`{ "ok": ["ledger.balance", "ledger.balance"] }`. `ObserveMetricTwice` returns
`{ "ok": 2 }`. `TraceObservabilitySpan` returns
`{ "ok": ["CalculateLedgerBalance"] }`.

Verify transactions (SQLite commit survives recreate; rollback hides writes):

```sh
cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CommitTwoDurableMovements \
  examples/apps/inputs/movement-pair-commit.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-tx-c01.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-tx-c02.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl RollbackTwoDurableMovements \
  examples/apps/inputs/movement-pair-rollback.json

cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl FindDurableMovement \
  examples/apps/inputs/movement-tx-r01.json
```

`CommitTwoDurableMovements` returns the second movement. Fresh-process finds
for `movement-tx-c01` and `movement-tx-c02` must succeed. After
`RollbackTwoDurableMovements`, find for `movement-tx-r01` must be `not_found`.
Memory adapters prove the same contract in-process via `cargo test`.

Verify jobs (in-process memory enqueue is covered by `cargo test`; durable
cross-process proof is the `ScheduleDurableMovementPersist` / `tick` /
`FindDurableMovement` sequence above). Scheduled unit jobs use
`schedule "every <n>ms|s|m"`. In-process memory cache put/get/invalidate is
covered by `cargo test`; durable cache cross-process proof is the sequence
above.

## 6.1 Verify multi-file import

```sh
cargo run -p axl-compiler -- \
  check examples/apps/import-demo.axl --json

cargo run -p axl-compiler -- \
  eval examples/apps/import-demo.axl CalculateBalance \
  examples/apps/inputs/balance.json
```

The check command must return `ok: true` for application `ImportDemo`. The eval
command must return `80000` using the imported `CalculateBalance` flow from
`examples/modules/math-lib.axl`.

Missing import paths must report `AXL-P931`. Duplicate imported symbols must
report `AXL-N002`:

```sh
cargo run -p axl-compiler -- check examples/invalid/import-missing.axl
cargo run -p axl-compiler -- check examples/invalid/import-duplicate.axl
```

## 6.2 Verify the minimal UI slice

```sh
cargo run -p axl-compiler -- \
  check examples/apps/balance-ui.axl --json

cargo run -p axl-compiler -- \
  ui examples/apps/balance-ui.axl

cargo run -p axl-compiler -- \
  render examples/apps/balance-ui.axl /balance \
  examples/apps/inputs/balance.json > /tmp/balance-ui.html

cargo run -p axl-compiler -- \
  check examples/apps/form-demo.axl --json

cargo run -p axl-compiler -- \
  ui examples/apps/form-demo.axl

cargo run -p axl-compiler -- \
  serve examples/apps/form-demo.axl 127.0.0.1:8080
```

Open `http://127.0.0.1:8080/clienti/new` in a browser or curl it; the response
must be `text/html` with a form posting to `/clienti` and a dashboard sidebar
linking `/clienti` and `/clienti/new`. POST JSON to `/clienti` still goes
through the api route unchanged.

```sh
curl -s -X POST http://127.0.0.1:8080/clienti \
  -H 'content-type: application/json' \
  -d '{"nome":"Alice","email":"a@example.com","budget":1000,"stato":"attivo"}'
```

The manifest must use protocol `axl-ui/1` and bind `/balance` to
`CalculateBalance`. The rendered HTML must contain `80000`. Invalid UI
declarations must report stable codes (`AXL-P951`, `AXL-U904`, `AXL-U905`,
`AXL-P960`, `AXL-U908`).

## 7. Verify the HTTP backend

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

From another terminal:

```sh
curl -X POST http://127.0.0.1:8080/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response must be `80000`. Posting `movement-invalid.json` to `/movements`
must return HTTP 422 with `amount_must_be_positive`.

Verify state continuity while the server remains running:

```sh
curl -X POST http://127.0.0.1:8080/movements \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-valid.json

curl -X POST http://127.0.0.1:8080/movement-by-id \
  -H 'content-type: application/json' \
  --data-binary '"movement-001"'
```

Both responses must contain movement `movement-001`. The analogous `/durable`
routes use a configured SQLite file and remain readable after restarting the
server.

Verify the open bearer guard:

```sh
curl -X POST http://127.0.0.1:8080/secure/balance \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer axl-cashflow-demo' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response is `80000`. Omitting the authorization header returns 401; using a
different token returns 403. The token is deliberately a visible demo fixture,
not a production secret.

Verify the replaceable HS256 JWT guard (mint a demo token with the same secret
and issuer as `CashflowDemoJwt`):

```sh
# Helper shape used by compiler tests (HS256, claims sub + iss):
# encode_hs256_jwt("axl-cashflow-demo-jwt", {"sub":"alice","iss":"axl-cashflow"})
TOKEN=$(python3 - <<'PY'
import base64, hashlib, hmac, json
secret=b"axl-cashflow-demo-jwt"
def b64(data): return base64.urlsafe_b64encode(data).rstrip(b"=").decode()
header=b64(json.dumps({"alg":"HS256","typ":"JWT"},separators=(",",":")).encode())
payload=b64(json.dumps({"sub":"alice","iss":"axl-cashflow"},separators=(",",":")).encode())
sig=b64(hmac.new(secret,f"{header}.{payload}".encode(),hashlib.sha256).digest())
print(f"{header}.{payload}.{sig}")
PY
)

curl -X POST http://127.0.0.1:8080/jwt/balance \
  -H 'content-type: application/json' \
  -H "authorization: Bearer $TOKEN" \
  --data-binary @examples/apps/inputs/movement-batch.json
```

Missing bearer → 401; malformed / wrong-signature / wrong-`iss` / missing-`sub`
→ 403; valid HS256 JWT → `80000`. The HMAC `secret` and `issuer` are demo
skill config visible in the Graph/manifest — the same honesty rule as
`CashflowDemoBearer`. True secret references (no plaintext in IR) are Gate 8.

Verify the open request middleware gate:

```sh
curl -X POST http://127.0.0.1:8080/guarded/balance \
  -H 'content-type: application/json' \
  -H 'x-axl-client: cashflow-demo' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The response is `80000`. Omitting the client header or using another value
returns 403.

Verify the open response middleware gate:

```sh
curl -i -X POST http://127.0.0.1:8080/annotated/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The body is `80000` and the response includes `x-axl-middleware: ok`. The header
comes from capacity-backed response middleware, not Axum-only CORS logic.

Verify the open rate-limit middleware gate (limit 5 within 60s on this demo):

```sh
for i in 1 2 3 4 5; do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST http://127.0.0.1:8080/limited/balance \
    -H 'content-type: application/json' \
    --data-binary @examples/apps/inputs/movement-batch.json
done
curl -i -X POST http://127.0.0.1:8080/limited/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

The first five responses are HTTP 200 with body `80000`. The sixth is HTTP 429
with `rate_limit_exceeded`. The limiter is the replaceable `MemoryRateLimit`
skill behind capacity `RateLimit`.

Verify the open CORS middleware gate:

```sh
curl -i -X POST http://127.0.0.1:8080/cors/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json

curl -i -X OPTIONS http://127.0.0.1:8080/cors/balance
```

The POST body is `80000` and includes `access-control-allow-origin: *` plus
allow-methods/headers from the replaceable `axl::middleware::cors` skill.
OPTIONS returns HTTP 204 with the same CORS headers and does not run the flow.

After saving the durable movement, verify both request bindings:

```sh
curl http://127.0.0.1:8080/movements/movement-001
curl 'http://127.0.0.1:8080/movements/find?id=movement-001'
```

Both return the movement. The second URL proves that the exact `/movements/find`
route wins over the `/movements/{id}` template.

Verify composite request assembly:

```sh
curl -X POST \
  'http://127.0.0.1:8080/accounts/account-1/movement-preview?dry_run=true' \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-valid.json
```

The response contains the validated movement. The flow input was assembled as
`{ account, movement, dry_run }` from path, complete body and query.

Verify header and cookie request bindings:

```sh
curl http://127.0.0.1:8080/me -H 'x-user: alice'
curl http://127.0.0.1:8080/session -H 'cookie: sid=session-42'

curl -X POST http://127.0.0.1:8080/client-preview \
  -H 'content-type: application/json' \
  -H 'x-user: alice' \
  -H 'cookie: sid=session-42' \
  --data-binary @examples/apps/inputs/movement-valid.json
```

The first two responses are `"alice"` and `"session-42"`. The composite route
returns the validated movement after assembling `{ user, sid, movement }` from
header, cookie and body.

Verify memory cache routes (same server process):

```sh
curl -X POST http://127.0.0.1:8080/cache/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/balance-cache.json

curl -X POST http://127.0.0.1:8080/cache/balance/get \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/balance-cache-key.json

curl -X POST http://127.0.0.1:8080/cache/balance/invalidate \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/balance-cache-key.json
```

Put/get return `"80000"` inside `{ "ok": ... }`. Invalidate returns
`{ "ok": true }`. A following get returns HTTP 422 with `cache_miss`.

Verify observability routes (same server process):

```sh
curl -X POST http://127.0.0.1:8080/observability/log \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/observability-line.json

curl -X POST http://127.0.0.1:8080/observability/log \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/observability-line.json

curl -X POST http://127.0.0.1:8080/observability/logs \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/unit.json
```

The list response contains two `"ledger.balance"` lines inside `{ "ok": ... }`.

## 8. Verify invalid programs

These commands are expected to fail:

```sh
cargo run -p axl-compiler -- \
  check examples/invalid/closed-blueprint.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/wrong-parameter.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/instance-overrides.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-types.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-calls.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-records.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-folds.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-runs.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-matches.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-transforms.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-parallel.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-attempts.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-races.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-routes.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/provider-config-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/provider-configs.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-auth-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-auth.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-request-bindings.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-middleware-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/http-middleware.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-events-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-events.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-jobs-syntax.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-jobs.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-transactions.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-migrations.axl --json

cargo run -p axl-compiler -- \
  check examples/invalid/flow-queries.axl --json
```

The first diagnostic set must include `AXL-O401`; the second must include
`AXL-V403`; the third must include `AXL-I605`, `AXL-I607` and `AXL-P405`; the
fourth must include `AXL-X803` and `AXL-X806`; the fifth must include every code
from `AXL-X816` through `AXL-X821`; the sixth must include every code from
`AXL-X831` through `AXL-X835`; the seventh must include `AXL-N805` and
`AXL-X841` through `AXL-X843`; the eighth must include `AXL-X851` through
`AXL-X856`; the ninth must include `AXL-X861` through `AXL-X865`.
The tenth must include `AXL-X802`, `AXL-N806`, `AXL-X871` through `AXL-X879` and
`AXL-X881` through `AXL-X884`.
The eleventh must include every code from `AXL-X891` through `AXL-X895`.
The twelfth must include every code from `AXL-X901` through `AXL-X907`.
The thirteenth must include every code from `AXL-X911` through `AXL-X916`.
The fourteenth must include every code from `AXL-H901` through `AXL-H907`.
The fifteenth must include `AXL-P313` and `AXL-P314`. The sixteenth must include
`AXL-N303`, `AXL-N304` and `AXL-V305`.
The seventeenth must include every code from `AXL-P913` through `AXL-P917`. The eighteenth must
include every code from `AXL-H908` through `AXL-H912`.
The nineteenth must include every code from `AXL-H913` through `AXL-H917`.
The twentieth must include `AXL-P918`. The twenty-first must include every code
from `AXL-H918` through `AXL-H922`. The twenty-second must include `AXL-P920`.
The twenty-third must include every code from `AXL-E901` through `AXL-E906`.
The twenty-fourth must include `AXL-P921`. The twenty-fifth must include every
code from `AXL-J901` through `AXL-J908`. The twenty-sixth must include
`AXL-D901`. The twenty-seventh must include `AXL-D902`. The twenty-eighth must
include `AXL-D903`. File-based import invalid examples must report `AXL-P931`
(`import-missing.axl`) and `AXL-N002` (`import-duplicate.axl`).

## 9. Verify canonical formatting and transport

```sh
cargo run -p axl-compiler -- \
  fmt examples/blocks/05-open-dataview.axl

cargo run -p axl-compiler -- \
  pack examples/blocks/05-open-dataview.axl --matrix
```

`cargo test --workspace` performs the stronger check: decoding the matrix form
must reconstruct exactly the same canonical Semantic Graph IR.

## What this proves

- the new surfaces are parsed, type checked and lowered to Graph IR;
- compatible providers are checked for input, action, policy, slot and hook;
- closed blueprints and invalid scalar defaults are rejected;
- the open surface is machine-discoverable through a generated manifest;
- instance settings and provider overrides survive the Packed IR round-trip;
- enums and ordered flow statements survive the Packed IR round-trip;
- typed flows validate input and evaluate expressions at runtime;
- capacity dependencies, calls and provider bindings are statically checked;
- conditional expressions and multiline records are typed and executable;
- folds and composed flow runs survive formatting and Packed IR round-trips;
- enum matches are exhaustive and executable;
- map/filter transforms are scoped, typed and executable;
- stable ascending/descending sort is typed and executable;
- grouping produces a checked `Map<K,List<T>>` without handwritten Rust;
- non-empty list literals infer a common type and retain multiline formatting;
- `parallel` uses concurrent provider forks and preserves source order;
- `attempt` enforces idempotency, bounded retry and real deadlines;
- `race` returns the first successful idempotent worker;
- HTTP routes dispatch through the generic Axum runtime;
- consecutive HTTP requests share one process-local provider runtime;
- memory and SQLite providers execute through the same replaceable ABI;
- typed provider config survives Graph/Packed IR and `axl-provider/1` generation;
- configured SQLite data survives destruction and recreation of the runtime;
- API auth is capacity-backed (static bearer and HS256 JWT) and proves missing,
  denied and accepted requests;
- ordered request middleware is capacity-backed over typed envelopes;
- rate-limit request middleware uses `RateLimit.allow` and returns HTTP 429;
- ordered response middleware mutates response headers through typed envelopes;
- CORS middleware adds `Access-Control-*` headers and serves OPTIONS preflight;
- typed events reach multiple subscribers through `emit`;
- capacity-backed jobs enqueue, tick, retry and survive SQLite runtime recreate;
- Cache get/put/invalidate works through memory and durable SQLite skills;
- Logger write/list, Metrics increment/get and Tracer start/finish/list are
  executable through memory skills;
- rate-limit request middleware returns HTTP 429 after the configured budget;
- scalar path/query/header/cookie bindings are checked, decoded and exact-route-safe;
- composite request entities are assembled from checked body/path/query/header/cookie nodes;
- documentation examples remain coupled to compiler tests;
- capacity-backed transactions begin/commit/rollback through memory and SQLite;
- SQLite commit survives runtime recreate; rollback leaves neither record visible;
- capacity-backed migrations up/down/status through memory and SQLite;
- SQLite schema history survives runtime recreate; `down` rolls back one head version;
- typed store `query` filter/order/page through memory, SQLite and document;
- durable SQLite and document query pages survive runtime recreate;
- document/JSON-file store (`rust::axl::store::document`) shares save/find/query
  with memory and SQLite; cashflow switches by skill binding only;
- `ui` / `page` nodes lower to Graph IR and emit `axl-ui/1`;
- `render` evaluates a bound flow and displays typed scalar/entity fields in HTML.

It does not prove PostgreSQL/MySQL, document tx/migrate, routing shell, component
registry, forms, tables or responsive admin UI (full Gate 4).
HTTP execution, process-local memory, restart-durable configured SQLite and
document JSON, typed multi-subscriber events, durable jobs, Cache get/put/invalidate,
Logger/Metrics/Tracer observability, transaction commit/rollback, migration
upgrade/downgrade, typed store queries and the minimal UI page slice are proven.
Generated target files are not yet a deployable app. Capacity-backed rate-limit
and CORS middleware are proven. Capacity-backed HS256 JWT auth is proven with
demo config secrets. True secret references remain Gate 8; OAuth remains.
Gate 2 auth adapters are otherwise complete. Next Gate 3 target: PostgreSQL/
MySQL and document tx/migrate behind the same capacities. Next Gate 4 target:
routing shell, component registry and admin UI kit.
