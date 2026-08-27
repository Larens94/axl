# AXL 4 experiment

`crm.axl` is the first executable example of the semantic blueprint language.
It validates entities, capacities, skills, open ports, UI slots, lifecycle hooks,
contracts, effects, capabilities and an agent model.

Run from the repository root:

```sh
~/.cargo/bin/cargo run -p axl-compiler -- check examples/next/crm.axl --json
~/.cargo/bin/cargo run -p axl-compiler -- ir examples/next/crm.axl
~/.cargo/bin/cargo run -p axl-compiler -- pack examples/next/crm.axl --matrix
~/.cargo/bin/cargo run -p axl-compiler -- experiment examples/next/crm.axl build/axl4
```

The experiment output contains canonical readable AXL, JSON Graph IR, matrix-
formatted Packed Graph IR and initial Rust, React, SQL and agent target contracts.
See `SPEC-4.0.md` for the implemented boundary.

For smaller, independently verified construction examples, see
`examples/blocks`. The visual overview is available in `docs/index.html`, and
the simplified presentation is `presentation.html`.
