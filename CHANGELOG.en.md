**Languages:** [Italiano](CHANGELOG.md) · [English](CHANGELOG.en.md)

# Changelog

## 2.0.0 — in development

- Added Compact Source 2 as the canonical syntax intended exclusively for agents.
- Added numeric instruction frames, RPN expressions, and block opcodes.
- Added functions, modules, memory, agents, workflows, and tool calls in compact format.
- Added the canonical compact writer and the `axl pack` migration command.
- Redefined Rust as the first runtime/backend within a multi-bridge and multi-backend architecture.
- Redesigned the documentation and GitHub Pages around the compact language model.
- Added homogeneous immutable `list<T>` values with compact `~arity` construction.
- Published AX-IR 1.2 while retaining decoding support for AX-IR 1.0 and 1.1.
- Added list transport through functions, tool capabilities, SQLite AM, and canonical JSON CLI output.
- Added the typed `map<K,V>` tracer for Compact Source and the reference runtime.
- Published an always-light bilingual static documentation portal with sidebar navigation, search, page outlines, and Italian/English links.

## 1.1.0 — in development

- Added typed functions, parameters, returns, and calls in expressions.
- Added static type checking for function contracts and typed variables.
- Added isolated function scopes and bounded recursion depth.
- Extended the IR and JSON schema with function nodes.
- Added relative module imports, aliases, namespaced functions, and cycle detection.
- Published AX-IR 1.1 while retaining verified decoding of AX-IR 1.0.
- Added comprehensive architecture, language, runtime, security, toolchain, and roadmap documentation under `docs/`.
- Added the Apache 2.0 license text for the public open-source repository.

## 1.0.0

- Added typed and validated JSON IR 1.0 and the `compile`/`exec` CLI commands.
- Added agents, tool permissions, workflows, and static cycle detection.
- Added tool effects, explicit approvals, and audit events.
- Added scoped memory, metadata, TTL, versioning, and deletion.
- Added SQLite schema migration and a provider-independent memory interface.
- Added budgets for expressions, intermediate values, output, tool calls, and memory operations.
- Strengthened runtime types, CLI errors, reserved identifiers, and IR validation.

## 0.3.0

- Added bounded loops and persistent SQLite memory.

## 0.2.0

- Added typed expressions, conditions, and explicit tools.

## 0.1.0

- First release of the parser, IR, interpreter, and CLI.
