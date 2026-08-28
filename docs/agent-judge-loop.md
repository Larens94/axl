# AXL judge autoloop

Steward agent improves the **language**; judge agent tries to build a **product**
in AXL only. The steward tests judge output and queues the next language fix.

## Roles

| Role | Does | Does not |
|---|---|---|
| **Steward** (main autoloop) | open primitives, CLI/docs, run `scripts/verify-libro-cassa.sh`, read judge report, update state | copy application logic into Rust/React |
| **Judge** (delegated) | author/extend `examples/apps/ledger*.axl`, iterate on `axl-check/1`, eval/serve/render proofs | implement missing language in Rust; must STOP and report gaps |

## Product target

**Libro cassa** (entrate/uscite): typed domain module + HTTP API + minimal UI, multi-file,
memory and durable SQLite behind the same capacity. Not a fork of `cashflow-core.axl`.

## State file

`.cursor/judge-loop-state.json` tracks:

- `iteration` — loop count
- `milestone` — current product goal for the judge
- `steward_queue` — language fixes before next judge run
- `last_verification` — steward test output summary
- `last_judge_summary` — short report from judge

## Cycle (each tick)

1. Read state + this doc.
2. If `steward_queue` is non-empty → implement smallest item, prove fmt/test/clippy,
   run `scripts/verify-libro-cassa.sh`, dequeue, update state. **Do not commit.**
3. Else if no judge subagent is running → launch judge with `judge_prompt` from state.
4. When judge completes → steward runs verification, records summary, fills
   `steward_queue` from judge blockers, advances `milestone`, increments `iteration`.
5. Skip launching a second judge while one is active.

## Judge prompt template

See `docs/agent-judge-prompt.md` (filled per iteration by steward).

## Verification gate

```sh
./scripts/verify-libro-cassa.sh
```

Must pass before accepting a judge iteration as complete.

## Milestones (product)

| # | Goal |
|---|---|
| 1 | Domain + API + saldo UI (done manually) |
| 2 | UI elenco voci (query page) + verify script green |
| 3 | Durable SQLite round-trip via HTTP only |
| 4 | Broken-program repair drill (`axl-check/1` on errors) |
| 5 | Single-command demo entry in README |

Steward advances language when judge reports a **missing primitive**, not when the
judge wants app-specific Rust.
