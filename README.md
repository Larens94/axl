# AXL

AXL is an experimental agent-native semantic blueprint language.

The current compiler implements one focused pipeline:

```text
readable AXL
  -> typed AST
  -> Semantic Graph IR
  -> Packed Graph IR
  -> Flow runtime + replaceable providers + target contracts
```

AXL describes software through entities, capacities, skills and blueprints with
typed open surfaces. The compiler rejects closed blueprints. Rust and React are
target implementations rather than the source language.

## Try the experiment

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  check examples/next/crm.axl --json

~/.cargo/bin/cargo run -p axl-compiler -- \
  experiment examples/next/crm.axl build/axl4
```

The second command generates canonical AXL, JSON Graph IR, Packed IR and initial
target contracts.

Execute behavior written in AXL:

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  eval examples/apps/cashflow-core.axl CalculateBalance \
  examples/apps/inputs/balance.json
```

This returns `80000`. The balance rule is an AXL flow, not an
application-specific Rust function.

Serve routes declared in AXL through the generic Axum runtime:

```sh
~/.cargo/bin/cargo run -p axl-compiler -- \
  serve examples/apps/cashflow-core.axl 127.0.0.1:8080
```

## Portal gestionale (sviluppo locale)

Prerequisiti: [Rust](https://rustup.rs) (toolchain recente, workspace edition 2024).
Opzionale: Node.js 20+ per l’host React.

Aggiorna il clone e avvia il CRM portal (auth + vendite in AXL):

```sh
git checkout main
git pull origin main

cargo build -p axl-compiler

# Terminale 1 — HTML + API AXL su http://127.0.0.1:8080
./scripts/demo-portal.sh
```

| URL | Uso |
|---|---|
| http://127.0.0.1:8080/login | Login sessione (`admin@example.com` / `admin123`) |
| http://127.0.0.1:8080/home | Dashboard dopo login |
| http://127.0.0.1:8080/clienti | Clienti (drawer dettaglio) |
| http://127.0.0.1:8080/clienti/demo | Demo senza login (gate automatici) |

Persistenza SQLite locale: `./build/portal-auth.db`, `./build/vendite.db` (create al primo avvio).

Host React opzionale (proxy same-origin per cookie `sid`):

```sh
# Terminale 2 — Vite su http://127.0.0.1:5173
cd hosts/portal-web
npm install
AXL_PROXY_TARGET=http://127.0.0.1:8080 npm run dev
```

Verifica end-to-end: `./scripts/verify-portal.sh`

Dettaglio app, curl e flussi vendite: [`examples/apps/README.md`](examples/apps/README.md).

## Project map

- `SPEC-4.0.md` — implemented language boundary.
- `docs/index.html` — concise browser documentation.
- `docs/blocks.md` — verified guide to open block construction.
- `docs/agent-testing.md` — repeatable test handoff for another agent.
- `docs/executable-flows.md` — executable Flow Runtime 2 semantics.
- `docs/runtime-abi.md` — replaceable native provider contract.
- `docs/backend-http.md` — executable Axum route adapter and current boundary.
- `docs/roadmap.md` — executable autoloop gates and current position.
- `presentation.html` — simplified, responsive project presentation.
- `mondo.html` — 3D kid-simple marketing world: truthful status as building blocks.
- `film.html` — autoplaying cinematic explanation of the complete plan.
- `film/axl-plan-film.mp4` — narrated 1080p Italian export.
- `docs/agent-work-packages.md` — nine executable work packages for specialized agents.
- `docs/agent-autoloop.md` — research autoloop roster and iteration checklist.
- `AGENTS.md` — how to launch a specialized WP agent.
- `examples/blocks` — small examples compiled by the test suite.
- `examples/catalog/software-foundation.axl` — fourteen open foundation blocks.
- `examples/apps/cashflow-core.axl` — executable validation and balance flows.
- `examples/next/crm.axl` — semantic CRM experiment.
- `runtime/axl-compiler/src/next` — parser, analyzer, IR and target adapters.
- `schema/axl-ir-4.0.schema.json` — Graph IR JSON schema.
- `schema/axl-open-block-2.schema.json` — block and instance manifest schema.
- `schema/axl-flow-2.schema.json` — executable flow and capacity-call manifest schema.
- `schema/axl-http-1.schema.json` — checked HTTP route manifest schema.
- `schema/axl-provider-1.schema.json` — typed provider configuration manifest schema.

## Build an open block

```axl
capacity CustomerRow
  op render Customer -> UI

skill DefaultCustomerRow provides CustomerRow
  native react crm::DefaultCustomerRow

blueprint CustomerList
  slot table.row: CustomerRow = DefaultCustomerRow
```

This exact example is compiled from `examples/blocks/02-ui-slot.axl`. See the
[block guide](docs/blocks.md) for backend ports, hooks and agent declarations.

The complete protocol example is `examples/blocks/05-open-dataview.axl`. It
uses typed parameters, state, events, actions, errors, policies, slots and hooks
without modifying generated target files.

`examples/blocks/06-instance-override.axl` derives a configured instance with
`set` and `use`; the original blueprint and generated Rust/React remain untouched.

The foundation catalog adds fourteen compiler-verified contracts covering data,
commands, API, UI, events, jobs, observability, agent tools and scenarios.

## Verification

```sh
~/.cargo/bin/cargo test --workspace
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Status: experiment. Typed flows, replaceable configured providers, durable
SQLite paths, composite body/path/query/header/cookie routes, capacity-backed
bearer auth (static token + HS256 JWT), capacity-backed transactions
(begin/commit/rollback), migrations and typed store queries (filter/order/page)
execute. True secret references (Gate 8), OAuth, multi-database adapters, richer
backend services and React runtime generation are not implemented yet.
