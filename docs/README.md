# AXL 4 documentation

AXL 4 is an executable language experiment for describing typed software graphs.
This documentation deliberately separates verified behavior from the roadmap.

## Start here

- [Building blocks](blocks.md) — how capacities, skills, ports, slots and hooks
  compose software without closing its extension points.
- [Toolchain](toolchain.md) — commands, representations and generated files.
- [Implementation status](status.md) — what works now and what does not.
- [Agent testing guide](agent-testing.md) — repeatable verification commands.
- [Language specification](../SPEC-4.0.md) — the complete implemented boundary.
- [Browser documentation](index.html) — the concise visual guide.
- [Presentation](../presentation.html) — a keyboard-navigable explanation.

## Verified examples

Every `.axl` file listed below is part of the Rust test suite:

- [Store block](../examples/blocks/01-store.axl)
- [UI slot](../examples/blocks/02-ui-slot.axl)
- [Lifecycle hook](../examples/blocks/03-hook.axl)
- [Agent model](../examples/blocks/04-agent.axl)
- [Complete open DataView](../examples/blocks/05-open-dataview.axl)
- [Software foundation catalog](../examples/catalog/software-foundation.axl)
- [Composed CRM graph](../examples/next/crm.axl)

Run `cargo test --workspace` to verify that the documented programs compile and
that their Packed IR decodes to the same canonical Semantic Graph IR.
