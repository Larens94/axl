# Toolchain e utilizzo

## Requisiti

- Rust e Cargo stabili;
- Node.js e npm per compilare il frontend generato;
- SQLite per l'esecuzione locale del CRM, incorporato dal backend Rust.

## Costruire la CLI

```bash
cargo build -p axl-cli
cargo test --workspace
```

Il binario risultante è `target/debug/axl`.

## Controllo e formattazione

```bash
target/debug/axl check examples/crm/crm.axl
target/debug/axl fmt examples/crm/crm.ui.axl --width 100 --check
```

`fmt` mantiene il sorgente compatto multilinea: newline e indentazione non hanno
semantica, ma rendono diff e revisione più affidabili. La forma su una riga è
riservata a hashing, cache e trasporto.

## Generare il CRM

```bash
target/debug/axl build examples/crm/crm.axl -o build/crm
```

La build legge automaticamente `examples/crm/crm.ui.axl` e produce:

```text
build/crm/
├── backend/       # crate Rust Axum + SeaORM
├── frontend/      # app React + Vite + Refine + MUI
└── migrations/    # schema SQL SQLite
```

## Avvio sviluppo

```bash
target/debug/axl dev examples/crm/crm.axl -o build/crm
```

In alternativa i target possono essere avviati separatamente:

```bash
cargo run --manifest-path build/crm/backend/Cargo.toml
npm install --prefix build/crm/frontend
npm run dev --prefix build/crm/frontend
```

Il backend ascolta su `http://127.0.0.1:3000`; Vite pubblica il frontend su
`http://localhost:5173` e inoltra `/api` al backend.

## Verifica proporzionata alla release

```bash
cargo test --workspace
cargo check --manifest-path build/crm/backend/Cargo.toml
npm run build --prefix build/crm/frontend
build/crm/smoke-test.sh
python3 docs/build_docs.py
git diff --check
```

Il frontend generato è intenzionalmente ispezionabile. Rust, React e SQL non sono
una seconda sorgente da mantenere manualmente: sono artefatti rigenerabili dal
contratto AXL.
