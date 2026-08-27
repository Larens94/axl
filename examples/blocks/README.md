# Verified block examples

These examples are compiled by `runtime/axl-compiler/tests/documented_examples.rs`.
They demonstrate only syntax and semantics implemented by the AXL 4 experiment.

- `01-store.axl` — a capacity, its Rust skill and a blueprint binding.
- `02-ui-slot.axl` — a replaceable React UI slot.
- `03-hook.axl` — a replaceable lifecycle hook with recorded contracts.
- `04-agent.axl` — the current belief/goal/plan agent model.
- `../next/crm.axl` — all the building blocks composed into one CRM graph.

Validate every example from the repository root:

```sh
for file in examples/blocks/*.axl; do
  ~/.cargo/bin/cargo run -q -p axl-compiler -- check "$file"
done
```

The examples prove parsing, semantic analysis and Graph IR generation. They do
not imply executable Rust, React or agent behavior that the compiler does not
yet generate.
