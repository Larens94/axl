# AXL Portal React host

Consumes `axl-ui/1` codegen (`axl_routes.tsx`, layouts, registry) from
`examples/apps/portal.axl`. **No portal IAM/vendite logic lives here** — only
layout slots and a same-origin proxy so `sid` cookies work with
`axl-compiler serve`.

Full portal docs: [docs/agent-portal-framework.md](../../docs/agent-portal-framework.md)
([GitHub](https://github.com/Larens94/axl/blob/main/docs/agent-portal-framework.md)).

## What is composed from AXL?

| Layer | Source |
|---|---|
| API routes, auth, RBAC, vendite flows | `examples/apps/portal.axl` + `domains/*` |
| UI pages, forms, drawer, KPI, chart | `ui` blocks in `portal.axl` → HTML via `serve` |
| React route table + layouts | `axl-compiler experiment` → `src/generated/*` |
| Visible page content in this host | **HTML fetched from AXL** (`AxlSurface.tsx`) |

Rust (`serve`) and this React app are **targets**, not the product language.

## Run

```sh
# terminal 1 — AXL HTML/API (port 8080)
./scripts/demo-portal.sh

# terminal 2 — Vite host (port 5173, proxies to 8080)
cd hosts/portal-web
npm install
AXL_PROXY_TARGET=http://127.0.0.1:8080 npm run dev
```

Open http://127.0.0.1:5173/login — forms and APIs hit Vite, which forwards to
AXL; `Set-Cookie: sid=…` stays on the Vite origin.

## Sync

`npm run sync` re-runs `axl-compiler experiment` and copies
`targets/react/*` into `src/generated/`.
