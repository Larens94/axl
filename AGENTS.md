# AXL specialized agents

This repository is driven by work packages in `docs/agent-work-packages.md`.
Cursor rules under `.cursor/rules/` encode the same protocol.

## How to run a specialized agent

Paste this prompt (replace `<WP-ID>`):

```text
Work on AXL package <WP-ID> only. Read SPEC-4.0.md, docs/status.md,
docs/roadmap.md, docs/agent-testing.md and docs/agent-work-packages.md first.

Start from a failing AXL example. Implement only general open primitives. Do
not move application logic into Rust, React or another target. If the language
lacks something, stop and report the exact missing syntax, type rule, IR node,
manifest field and runtime behavior.

Before handoff, prove formatting, positive/negative tests, exact Graph/Packed IR
round-trip, schema validity and a real end-to-end scenario. Update the spec,
status, testing guide, presentation and film/mondo narration. Report changed
files, commands, outputs, remaining boundary and any ABI decision required from
WP-01.
```

## Current wave (A)

| Agent | Package | Next primitive |
|---|---|---|
| Language steward | WP-01 | review opcodes/diagnostics from vertical slices |
| Backend | WP-02 | cache / observability |
| Data | WP-03 | transactions / migrations (after jobs or in parallel on separate ABI surface) |

Later waves: WP-04 UI, WP-05 AI/vector, WP-06 agents, WP-07 IoT, WP-08 security, WP-09 QA.

## Autoloop

Each iteration advances one failing example → one open primitive → proofs → docs.
Do not skip gates by writing application logic in Rust or React.
