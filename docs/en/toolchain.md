# Toolchain and usage

## Requirements

- stable Rust and Cargo;
- Node.js and npm to compile the generated frontend;
- SQLite for local CRM execution, embedded by the Rust backend.

## Build the CLI

```bash
cargo build -p axl-cli
cargo test --workspace
```

The resulting binary is `target/debug/axl`.

## Check and format

```bash
target/debug/axl check examples/crm/crm.axl
target/debug/axl fmt examples/crm/crm.ui.axl --width 100 --check
```

`fmt` keeps compact source multiline: whitespace is not semantic, but readable
layout makes diffs and review more reliable. Single-line form is reserved for
hashing, caching, and transport.

## Generate the CRM

```bash
target/debug/axl build examples/crm/crm.axl -o build/crm
```

The build automatically reads `examples/crm/crm.ui.axl` and emits:

```text
build/crm/
├── backend/       # Axum + SeaORM Rust crate
├── frontend/      # React + Vite + Refine + MUI app
└── migrations/    # SQLite SQL schema
```

## Development server

```bash
target/debug/axl dev examples/crm/crm.axl -o build/crm
```

The targets can also be started separately:

```bash
cargo run --manifest-path build/crm/backend/Cargo.toml
npm install --prefix build/crm/frontend
npm run dev --prefix build/crm/frontend
```

The backend listens on `http://127.0.0.1:3000`; Vite serves the frontend at
`http://localhost:5173` and proxies `/api` to the backend.

## Release-level verification

```bash
cargo test --workspace
cargo check --manifest-path build/crm/backend/Cargo.toml
npm run build --prefix build/crm/frontend
build/crm/smoke-test.sh
python3 docs/build_docs.py
git diff --check
```

Generated frontend code is intentionally inspectable. Rust, React, and SQL are
not a second source to maintain manually; they are reproducible AXL artifacts.
