# AXL auth-module autoloop (M27–M31)

Identity and access management in AXL: users, dynamic roles, permissions, admin UI, RBAC on Vendite.

## Milestones

| # | Goal | Status |
|---|---|---|
| 27 | `auth-domain.axl`: Utente/Ruolo/Permesso, register/login, seed admin+venditore | done |
| 28 | Login/register UI + session cookie (`sid`) on form redirect | done |
| 29 | Admin pages `/admin/utenti`, `/admin/ruoli`, CRUD ruoli/permessi | done |
| 30 | RBAC on Vendite (`PaginaClientiRbac`, `VerificaAuthorization`) | done |
| 31 | Dynamic roles (`CreaRuolo`, `AssegnaPermessoARuolo` eval gate) | done |

## Files

- `examples/apps/auth-domain.axl` — IAM domain (entities, stores, flows)
- `examples/apps/portal.axl` — unified portal (IAM + vendite, single `PortalUi`)
- `examples/apps/auth.axl` — auth domain check/eval entry (legacy)
- `scripts/verify-portal.sh` — full portal gate (auth + vendite)

## Steward primitives (open)

- `rust::axl::auth::password` — `hash` / `verify`
- `rust::axl::auth::jwt_sign` — `sign JwtClaims`
- `rust::axl::auth::jwt_decode` — `decode` (+ optional `Bearer ` prefix)
- `store.find_by` on memory/document stores
- UI pages: `from cookie.sid` / header / query bindings
- Form login redirect: `Set-Cookie: sid=<session_id>` when `LoginResult.session_id` is present

## Demo credentials

- Admin: `admin@example.com` / `admin123`
- Venditore: `venditore@example.com` / `vend123`

## UI routes

| Path | Descrizione |
|---|---|
| `/` | Home pubblica |
| `/home` | Home autenticata (cookie `sid`) |
| `/login` | Accesso |
| `/register` | Registrazione |
| `/password-dimenticata` | Richiesta reset password |
| `/reimposta-password` | Nuova password con token |
| `/admin/utenti`, `/admin/ruoli` | Pannello admin |

See `docs/agent-portal-framework.md` for modular shell layout (AuthUi / VenditeUi / session guards).
