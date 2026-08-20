# AXL Documentation

[Italiano](../README.md)

**AX — Agent eXecution** is the agent-native ecosystem. AXL is its current executable core, with a compact and deterministic canonical source format.

This documentation always distinguishes between:

- **current state**: Compact Source 2 and the Python reference runtime, version `2.0.0.dev0`;
- **target architecture**: AX-HIR/AX-MIR, the Rust runtime, native backends/bridges, VM, WASM, and platform;
- **future platforms**: backend, browser, desktop/mobile, and graphics.

## Contents

1. [AX: ecosystem and taxonomy](../ax-ecosystem.md)
2. [Vision and goals](overview.md)
3. [Stack architecture](architecture.md)
4. [Compact Source 2](../compact-syntax.md)
5. [Language guide](language-guide.md)
6. [Agents, workflows, tools, and memory](agent-runtime.md)
7. [AX-IR and compatibility](ax-ir.md)
8. [Security and the capability model](security.md)
9. [Toolchain and usage](toolchain.md)
10. [Roadmap](../roadmap.md)
11. [Application demos and platforms](../platform-demo-analysis.md)
12. [Development and contribution](../development.md)
13. [Glossary](glossary.md)

The current normative specification remains in [`../../SPEC.en.md`](../../SPEC.en.md). This directory describes the project and its overall design in a readable way; in the event of a conflict regarding implemented behavior, the specification, AX-IR schema, and tests take precedence.
