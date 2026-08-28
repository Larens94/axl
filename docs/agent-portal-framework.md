# Portal framework (AXL app shell)

Modular portal aligned with the **ledger.axl** pattern: domain modules + thin shell + session middleware.

## Layers

| Layer | Path | Responsibility |
|---|---|---|
| **Shell** | `examples/apps/portal.axl` | `import`, cross-domain API glue, `api`/`ui` wiring only |
| **Portal core** | `examples/apps/domains/portal/core.axl` | Session/RBAC guards (`RequireSession`, `RequireSessionPermesso`) |
| **Portal handlers** | `examples/apps/domains/portal/vendite-pages.axl` | Session-gated UI page flows |
| **Auth domain** | `examples/apps/auth-domain.axl` | IAM entities, stores, flows |
| **Sales shell** | `examples/apps/sales-domain.axl` | Aggregate imports only |
| **Sales aggregates** | `examples/apps/domains/sales/{cliente,prodotto,listino,preventivo,ordine,shared}.axl` | Vendite domain modules |

Domain files must **not** declare `api` or `ui` (except isolated eval stubs).

## Rule: AXL only for product logic

Portal IAM, vendite, admin and UI behavior are expressed **only in `.axl`**
(`auth-domain.axl`, `domains/sales/*`, `domains/portal/*`, `portal.axl`).
Rust (`ui.rs`, `http.rs`, `targets.rs`) and React are **targets**: bindings, middleware slots,
HTML shell primitives, manifest → codegen (`axl_routes.tsx`, layouts, registry).
Never implement login, RBAC, CRUD or page rules directly in Rust or React — add an
AXL primitive and bind it from flows.

## API surfaces (portal.axl)

| Block | Prefix | Purpose |
|---|---|---|
| `AuthApi` | `/auth/*` | Register, login, reset, admin IAM |
| `VenditeApi` | `/clienti`, `/preventivi`, … | In-memory CRUD + workflow |
| `VenditeDurableApi` | `/*/durable/*` | SQLite persistence |
| `PortalDemoApi` | `/secure/*` | Bearer auth demo |
| `PortalJwtDemoApi` | `/jwt/*` | JWT auth demo |
| `PortalRbacDemoApi` | `/rbac/*` | Header RBAC demo |

## UI surfaces

| Block | Routes | Auth |
|---|---|---|
| `AuthUi` | `/`, `/home`, `/login`, `/admin/*` | Public + cookie session on protected pages |
| `VenditeUi` | `/clienti`, `/prodotti`, … | **Session + permission** via `*Sessione` flows |
| `VenditeDemoUi` | `/*/demo` | Seeded eval/demo pages (no session, for gates) |

## Middleware pattern (Rust-style guards)

Session UI pages still use AXL flows:

```axl
flow PaginaClientiSessione uuid -> Result<ClientePage>
  make gate: SessionePermessoInput
    session_id = input
    permesso = "vendite.clienti.read"
  run _ = RequireSessionPermesso(gate)?
  // delegate to domain store query ...
  return pagina
```

UI binding: `page /clienti uuid -> ... = PaginaClientiSessione from cookie.sid`

**HTTP per-route guards** (open primitive) protect Form/JSON POST without
duplicating rules in Rust:

```axl
post /clienti Cliente -> Result<Cliente> = CreaCliente
  guard session RequireSession from cookie.sid
  guard can RequireSessionPermesso "vendite.clienti.read" from cookie.sid
```

Kinds: `session` (401), `can` (403 + permesso), `guest` (reject if already authenticated).
Guard flows are declared in AXL (`RequireSession`, `RequireSessionPermesso`).

## Permissions (seed)

- `admin.*` — IAM admin
- `vendite.clienti.read`, `vendite.prodotti.read`, `vendite.listini.read`
- `vendite.preventivi.read`, `vendite.preventivi.write`, `vendite.ordini.read`

## Known boundaries (steward queue)

- UI **composite page binding** (`cookie.sid` + `path.id`) is available for session-gated detail routes.
- **Per-route HTTP guards** (`session` / `can` / `guest`) protect Auth admin and VenditeApi mutations.
- **React codegen** emits routes/layouts/registry from `axl-ui/1` (`experiment` → `targets/react/`).
- **Gate 8 secret refs** (`secret("ENV")`) used by portal auth/vendite demo skills.
- Nested `List<>` form rows still use flat workaround forms.
- Full React host (Vite + shared cookie origin) remains a host integration, not AXL product logic.
- OAuth and package registry remain open Gate 8 items.

## Verify

```sh
./scripts/verify-portal.sh
./scripts/demo-portal.sh
```

Demo login: `admin@example.com` / `admin123` → `/home` → `/clienti` (requires session).

Demo without login: `/clienti/demo` (seeded data for automated gates).
