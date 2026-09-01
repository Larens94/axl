# AXL 4 documentation

AXL 4 is an executable language experiment for describing typed software graphs.
This documentation deliberately separates verified behavior from the roadmap.

**Repository:** [github.com/Larens94/axl](https://github.com/Larens94/axl)

**Sito documentazione (GitBook-style):** [larens94.github.io/axl/docs/](https://larens94.github.io/axl/docs/)

## Start here

- [Building blocks](blocks.md) — how capacities, skills, ports, slots and hooks
  compose software without closing its extension points.
- [Toolchain](toolchain.md) — commands, representations and generated files.
- [Implementation status](status.md) — what works now and what does not.
- [Executable flows](executable-flows.md) — implemented expression and runtime semantics.
- [Provider runtime ABI](runtime-abi.md) — replaceable native integration contract.
- [HTTP backend](backend-http.md) — typed routes and the executable Axum adapter.
- [Portal framework](agent-portal-framework.md) — gestionale CRM, layers, `hosts/portal-web`, verify.
- [Autoloop roadmap](roadmap.md) — executable gates and current position.
- [Agent testing guide](agent-testing.md) — repeatable verification commands.
- [Language specification](../SPEC-4.0.md) — the complete implemented boundary.
- [Browser documentation](index.html) — sidebar GitBook-style, capitoli Markdown renderizzati.
- [Presentation](../presentation.html) — a keyboard-navigable explanation.

## Portal gestionale (quick links)

| Topic | Doc |
|---|---|
| App shell + domains | [agent-portal-framework.md](agent-portal-framework.md) |
| React host (`hosts/portal-web`) | [../hosts/portal-web/README.md](../hosts/portal-web/README.md) |
| Runnable examples + curl | [../examples/apps/README.md](../examples/apps/README.md) |

Run locally: `./scripts/demo-portal.sh` → http://127.0.0.1:8080/login
(`admin@example.com` / `admin123` after `bootstrap BootstrapPortalProd`).

## Architecture: AXL vs targets

```text
examples/apps/portal.axl          ← product logic (API, UI, auth, vendite)
        │
        ├─► axl-compiler serve    ← Rust target: generic HTTP + HTML renderer
        │       :8080
        │
        └─► axl-compiler experiment → axl-ui/1 codegen
                │
                └─► hosts/portal-web   ← React target: routes + proxy + layout shell
                        :5173  (fetches HTML from :8080)
```

**Rule:** IAM, vendite, admin and UI behavior live **only in `.axl`**. Rust and
React provide open primitives (dispatch, providers, HTML shell, manifest →
`axl_routes.tsx`). Never implement CRM rules in Rust or TypeScript.

## Verified examples

Every `.axl` file listed below is part of the Rust test suite:

- [Store block](../examples/blocks/01-store.axl)
- [UI slot](../examples/blocks/02-ui-slot.axl)
- [Lifecycle hook](../examples/blocks/03-hook.axl)
- [Agent model](../examples/blocks/04-agent.axl)
- [Complete open DataView](../examples/blocks/05-open-dataview.axl)
- [Typed instance override](../examples/blocks/06-instance-override.axl)
- [Software foundation catalog](../examples/catalog/software-foundation.axl)
- [Executable cashflow core](../examples/apps/cashflow-core.axl)
- [Portal CRM](../examples/apps/portal.axl)
- [Composed CRM graph](../examples/next/crm.axl)

Run `cargo test --workspace` to verify that the documented programs compile and
that their Packed IR decodes to the same canonical Semantic Graph IR.
