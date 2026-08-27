# AXL HTTP Runtime 1

AXL HTTP Runtime 1 exposes typed flows through a generic Axum server. An
application declares routes in AXL; it does not require application-specific
Rust controllers.

```axl
api CashflowApi
  post /movements Movement -> Result<Movement> = ValidateAndStoreMovement
  post /movement-by-id uuid -> Result<Movement> = FindMovement
  post /balance MovementBatch -> money = CalculateLedgerBalance
```

The compiler verifies:

- supported methods: `get`, `post`, `put`, `patch`, `delete`;
- absolute exact paths;
- unique routes locally and across APIs;
- known input/output types;
- a declared target flow with exactly the same signature.

Routes become `api` and `route` nodes plus `dispatch` edges in Graph IR and
Packed IR. `targets/http/routes.json` uses protocol `axl-http/1`.

## Run

```sh
cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080

curl -X POST http://127.0.0.1:8080/balance \
  -H 'content-type: application/json' \
  --data-binary @examples/apps/inputs/movement-batch.json
```

Expected response: `80000`.

The server owns one provider runtime. This makes state available to consecutive
requests while the process is running. For example, post
`movement-valid.json` to `/movements`, then post the JSON string
`"movement-001"` to `/movement-by-id`: the second response contains the saved
movement.

Status mapping:

| Condition | Status |
|---|---:|
| successful flow value | 200 |
| AXL `Result` error | 422 |
| invalid JSON or runtime input | 400 |
| unknown method/path | 404 |

## Current boundary

Paths are exact: parameters and query decoding are not implemented. The shared
runtime is process-local; memory and the current in-memory SQLite connection are
lost on restart. Durable paths and connection configuration belong to the data
gate. Authentication, middleware, CORS, streaming, events, jobs, cache, rate
limits and observability remain later backend gates.
