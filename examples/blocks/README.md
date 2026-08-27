# Verified block examples

These examples are compiled by `runtime/axl-compiler/tests/documented_examples.rs`.
They demonstrate only syntax and semantics implemented by the AXL 4 experiment.

- `01-store.axl` — a capacity, its Rust skill and a blueprint binding.
- `02-ui-slot.axl` — a replaceable React UI slot.
- `03-hook.axl` — a replaceable lifecycle hook with recorded contracts.
- `04-agent.axl` — the current belief/goal/plan agent model.
- `05-open-dataview.axl` — the complete open-block surface protocol.
- `06-instance-override.axl` — parameter and provider overrides in pure AXL.
- `../next/crm.axl` — all the building blocks composed into one CRM graph.
- `../catalog/software-foundation.axl` — fourteen primary open blueprint contracts.

Validate every example from the repository root:

```sh
for file in examples/blocks/*.axl; do
  ~/.cargo/bin/cargo run -q -p axl-compiler -- check "$file"
done
```

The examples prove parsing, semantic analysis and Graph IR generation. They do
not imply executable Rust, React or agent behavior that the compiler does not
yet generate.
