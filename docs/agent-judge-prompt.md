# Judge prompt — sales milestone 21 (auth stub)

Repo: AXL sales-module autoloop

Milestones 1–20 complete. Readiness **10/10**. See `docs/agent-sales-autoloop.md` and `.cursor/judge-loop-state.json`.

## Milestone 21: auth on VenditeApi

1. If **auth capacity** exists (bearer/jwt/header gate) → wire `VenditeApi` routes with minimal auth policy in AXL only
2. If missing → **stop**, report exact gap in steward queue (syntax, IR, runtime)
3. Extend `verify-sales.sh` with positive/negative auth smoke if wired
4. Never implement auth rules in Rust beyond open primitives

Report A–F. Run `verify-sales.sh` + `verify-libro-cassa.sh`. No commit unless user asks.
