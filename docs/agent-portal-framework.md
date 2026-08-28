# Portal framework (AXL app shell)

Modular portal aligned with the **ledger.axl** pattern: domain modules + thin shell + session middleware.

## Layers

| Layer | Path | Responsibility |
|---|---|---|
| **Shell** | `examples/apps/portal.axl` | `import`, cross-domain API glue, `api`/`ui` wiring only |
| **Portal core** | `examples/apps/domains/portal/core.axl` | Session/RBAC guards (`RequireSession`, `RequireSessionPermesso`) |
| **Portal handlers** | `examples/apps/domains/portal/vendite-pages.axl` | Session-gated UI page flows |
| **Auth domain** | `examples/apps/auth-domain.axl` | IAM entities, stores, flows |
| **Sales domain** | `examples/apps/sales-domain.axl` | Vendite entities, stores, flows |

Domain files must **not** declare `api` or `ui` (except isolated eval stubs).

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

## Permissions (seed)

- `admin.*` — IAM admin
- `vendite.clienti.read`, `vendite.prodotti.read`, `vendite.listini.read`
- `vendite.preventivi.read`, `vendite.preventivi.write`, `vendite.ordini.read`

## Known boundaries (steward queue)

- UI pages cannot bind **cookie + path** on the same page → detail `{id}` routes are not session-gated yet.
- Form POST and JSON API routes are not session-wrapped (HTTP middleware per-route auth TBD).
- Nested `List<>` form rows still use flat workaround forms.

## Verify

```sh
./scripts/verify-portal.sh
./scripts/demo-portal.sh
```

Demo login: `admin@example.com` / `admin123` → `/home` → `/clienti` (requires session).

Demo without login: `/clienti/demo` (seeded data for automated gates).
