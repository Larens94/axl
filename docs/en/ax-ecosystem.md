# AX — Agent eXecution

[Italiano](../ax-ecosystem.md)

## Role

**AX** is the umbrella ecosystem for defining, compiling, and executing agent-native software. Today, the AXL repository contains the executable reference core: Compact Source 2, parser, validation, type-checking, AX-IR, and the Python runtime.

The specialized surfaces listed here define the target taxonomy. They are not declared implemented until they have their own parser, lowering, and conformance tests.

## Languages and Modules

| Abbreviation | Name | Responsibility |
|---|---|---|
| AA | Agent Definition | agent operational identity, goals, grants, and lifecycle |
| AM | Memory | memory, scope, routing, retention, and providers |
| AW | Workflow | orchestration, dependencies, tasks, and compensations |
| AP | Policy | capabilities, decisions, approval, and constraints |
| AT | Tools | tool contracts, MCP, effects, and adapters |
| AE | Events | events, streams, subscriptions, and delivery |
| AS | State | application state, transitions, and synchronization |
| AD | Data | models, schemas, queries, storage, and migrations |
| AI | Identity | principals, authentication, authorization, and indirect credentials |

`AI` means **Identity** in the AX namespace. Documentation must spell out the full name when the context could confuse it with “Artificial Intelligence.”

## Shared Core

- AX-IR, with separate evolution toward AX-HIR and AX-MIR;
- compiler and parser;
- runtime and scheduler;
- sandbox and Capability ABI;
- Context Engine;
- Memory Router;
- Model Router;
- Policy Engine;
- Tool/MCP Router.

## Platform Services

- secrets through host references, never values in source or IR;
- audit, tracing, and evals;
- versioning and compatibility;
- registry and package manager;
- Python and TypeScript SDKs;
- Rust core and WASM target;
- CLI and APIs;
- web, mobile, desktop, and server adapters.

## Pipeline

```text
AA | AM | AW | AP | AT | AE | AS | AD | AI
                    ↓
                  AX-IR
                    ↓
       runtime + scheduler + sandbox
                    ↓
          versioned adapters/providers
                    ↓
web | mobile | desktop | server | existing systems
```

Specialized languages do not access the host directly. Every effect passes through typed IR, policies, capabilities, and versioned adapters.

## Implemented Status

In the current development version, the following are real and tested:

- AXL Compact Source 2;
- AX-IR 1.0, 1.1, and 1.2;
- synchronous Python reference runtime;
- scalar types and homogeneous `list<T>`;
- functions, modules, initial AM, agents, workflows, tools, and policies.

The async scheduler, Rust/WASM runtime, complete routers, registry, SDKs, and cross-platform adapters remain later milestones.
