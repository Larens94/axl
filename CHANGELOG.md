# Changelog

## 2.0.0 — development

- Added Compact Source 2 as the canonical agent-only syntax.
- Added numeric instruction frames, RPN expressions and block opcodes.
- Added compact functions, modules, memory, agents, workflows and tool calls.
- Added canonical compact writer and `axl pack` migration command.
- Reframed Rust as the first runtime/backend within a multi-backend bridge architecture.
- Redesigned documentation and GitHub Pages around the compact language model.
- Added homogeneous immutable `list<T>` values with compact `~arity` construction.
- Published AX-IR 1.2 while preserving AX-IR 1.0/1.1 decoding.
- Added list transport through functions, tool capabilities, SQLite AM and canonical CLI JSON output.

## 1.1.0 — development

- Added typed functions, parameters, returns and expression calls.
- Added static type checking for function contracts and typed variables.
- Added isolated function scopes and bounded recursion depth.
- Extended JSON IR/schema with function nodes.
- Added relative module imports, aliases, namespaced functions and cycle detection.
- Published AX-IR 1.1 while preserving tested AX-IR 1.0 decoding.
- Added complete architecture, language, runtime, security, toolchain and roadmap documentation under `docs/`.
- Added the Apache License 2.0 text for the public open-source repository.

## 1.0.0

- Added typed, validated JSON IR 1.0 and compile/exec CLI.
- Added agents, tool grants, workflows and static cycle detection.
- Added tool effects, explicit approvals and audit events.
- Added scoped memory, metadata, TTL, versioning and forgetting.
- Added SQLite schema migration and provider-neutral memory interface.
- Added expression, intermediate-value, output, tool-call and memory-operation budgets.
- Hardened runtime typing, CLI errors, reserved identifiers and IR validation.

## 0.3.0

- Added bounded loops and persistent SQLite memory.

## 0.2.0

- Added typed expressions, conditions and explicit tools.

## 0.1.0

- Initial parser, IR, interpreter and CLI.
