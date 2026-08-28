# AXL Portal React host

Consumes `axl-ui/1` codegen (`axl_routes.tsx`, layouts, registry) from
`examples/apps/portal.axl`. **No portal IAM/vendite logic lives here** — only
layout slots and a same-origin proxy so `sid` cookies work with
`axl-compiler serve`.

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
