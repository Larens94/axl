# Comandi AXL CRM

## 1. Setup Environment
```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /Users/fabriziocorpora/Desktop/workspaces/axl
```

## 2. Pulisci generati precedenti
```bash
rm -rf generated
```

## 3. Compila AXL → Rust + React
```bash
cargo run -p axl-compiler -- examples/crm/crm.axl generated
```

## 4. Build Backend Rust
```bash
cd generated/backend
cargo build --release
```

## 5. Build Frontend React
```bash
cd /Users/fabriziocorpora/Desktop/workspaces/axl/generated/frontend
npm install
npm run build
```

## 6. Deploy
```bash
# Copia frontend build nella cartella statica
cp -r /Users/fabriziocorpora/Desktop/workspaces/axl/generated/frontend/dist/* /Users/fabriziocorpora/Desktop/workspaces/axl/build/crm/

# Avvia backend
cd /Users/fabriziocorpora/Desktop/workspaces/axl/generated/backend
./target/release/app
```

## 7. Test
- Backend API: http://localhost:3000/api/health
- Frontend: http://localhost:3000 (se serve static files)

## Struttura Generata
```
generated/
├── backend/          ← Rust (Axum + SeaORM)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── auth.rs
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── customer.rs
│   │   │   ├── lead.rs
│   │   │   ├── deal.rs
│   │   │   └── activity.rs
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── customer.rs
│   │       ├── lead.rs
│   │       ├── deal.rs
│   │       └── activity.rs
│   └── .env
└── frontend/         ← React (MUI + Refine)
    ├── package.json
    ├── src/
    │   ├── App.tsx
    │   └── pages/
    │       ├── dashboard.tsx
    │       ├── customer/
    │       │   ├── list.tsx
    │       │   ├── create.tsx
    │       │   ├── edit.tsx
    │       │   └── show.tsx
    │       ├── lead/
    │       ├── deal/
    │       └── activity/
    └── index.html
```

## Sorgente AXL
- `examples/crm/crm.axl` — Definizione completa dell'applicazione
