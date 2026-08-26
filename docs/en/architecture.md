# Architecture

AXL separates application semantics from the technologies used to execute them.
AXL source remains the primary contract; Rust, React, and SQL are generated,
inspectable targets.

## Two complementary frontends

```text
crm.axl                 crm.ui.axl
domain + APIs           compact numeric UI
        \               /
         parser + analysis
                ↓
        typed semantic model
        ├─ Rust/Axum/SeaORM
        ├─ React/Refine/MUI
        └─ SQL migrations
```

`crm.axl` describes the application, entities, fields, and CRUD resources. The
`crm.ui.axl` sidecar describes views and components without embedding JSX, CSS,
or framework APIs in the language.

## Rust workspace

```text
runtime/
├── axl-core-rs/       # AX-IR, interpreter, primitives, memory, policy, AX-UI
├── axl-cli/           # check, build, dev, and fmt commands
└── axl-compiler/      # application parser and Rust/React/SQL generators
```

## Application compilation

1. The parser reads application source and the optional UI sidecar.
2. Analysis checks identifiers, types, resources, and UI contracts.
3. One semantic model feeds every generator.
4. The Rust target emits Axum routers, SeaORM models, and server-side queries.
5. The React target emits the responsive shell, CRUD pages, and data tables.
6. The SQL target emits the SQLite schema and migrations.
7. The smoke test starts generated backend and frontend and exercises real APIs.

## Compact runtime

The `axl-core-rs` runtime remains separate from the application compiler. It
executes Compact Source 2 and AX-IR for bindings, flow, functions, agents,
workflows, memory, and primitives. This separation lets the application model
evolve without coupling syntax to host libraries.

## Technology boundaries

| Layer | AXL contract | Current implementation |
|---|---|---|
| Data | entities, fields, relationships | SQLite + SeaORM |
| Network | resources and CRUD operations | Axum + Tower HTTP |
| UI | views, components, properties | React + Refine + MUI |
| Tables | columns, priority, density | TanStack Table |
| Icons | semantic meaning | Lucide React |
| Agents | capabilities, memory, workflows | Rust runtime |

Host libraries are replaceable. AXL preserves semantics, types, and intent
instead of copying their APIs into the language.
