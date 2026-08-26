# AXL documentation

**AXL — Agent eXecution Language** is the primary source language for describing
domain, data, APIs, UI, and agents. Rust, React/TypeScript, and SQL are the first
application targets; the Rust runtime also executes Compact Source and native
primitives.

> Current release: `0.1.0-alpha.1`. This is a working proof of concept, not a
> promise of stable compatibility or production-ready deployment.

[Open the interactive Claude × Apple presentation](../presentation.html)

## What is available

- Rust parser, analysis, and application generation;
- Axum + SeaORM backend with SQLite, migrations, and CRUD APIs;
- React + Refine + Material UI + TanStack Table + Lucide frontend;
- Compact UI Source 3 numeric frames for views, components, and properties;
- Rust runtime for Compact Source 2, primitives, memory, policy, and web rendering;
- CRM demo with 6 entities, 30 CRUD operations, and 7 compact views;
- bilingual documentation portal and 43 Rust tests verified for this release.

## Pipeline

```text
crm.axl + crm.ui.axl
→ parser and typed semantic model
→ Rust/Axum/SeaORM + React/Refine/MUI + SQL
→ runnable CRM application
```

## Contents

1. [AX ecosystem and taxonomy](ax-ecosystem.md)
2. [Vision and goals](overview.md)
3. [Stack architecture](architecture.md)
4. [Compact source](compact-syntax.md)
5. [Language guide](language-guide.md)
6. [Agents, workflows, tools, and memory](agent-runtime.md)
7. [AX-IR and compatibility](ax-ir.md)
8. [Security and capabilities](security.md)
9. [Toolchain and usage](toolchain.md)
10. [Roadmap](roadmap.md)
11. [CRM and UI coverage](platform-demo-analysis.md)
12. [Development and contribution](development.md)
13. [Glossary](glossary.md)

The normative specification remains [`../../SPEC.en.md`](../../SPEC.en.md). When
implemented behavior conflicts with prose, code, AX-IR schemas, and tests prevail.
