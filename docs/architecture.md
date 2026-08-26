# Architettura

AXL separa la semantica dell'applicazione dalle tecnologie usate per eseguirla.
Il sorgente AXL resta il contratto principale; Rust, React e SQL sono target
generati e ispezionabili.

## Due frontend complementari

```text
crm.axl                 crm.ui.axl
dominio + API           UI numerica compatta
        \               /
         parser + analisi
                ↓
       modello semantico tipizzato
        ├─ Rust/Axum/SeaORM
        ├─ React/Refine/MUI
        └─ SQL migrations
```

`crm.axl` descrive applicazione, entità, campi e risorse CRUD. Il sidecar
`crm.ui.axl` descrive viste e componenti senza introdurre JSX, CSS o API di un
framework nel linguaggio.

## Workspace Rust

```text
runtime/
├── axl-core-rs/       # AX-IR, interprete, primitive, memoria, policy e AX-UI
├── axl-cli/           # comandi check, build, dev e fmt
└── axl-compiler/      # parser applicativo e generatori Rust/React/SQL
```

## Compilazione applicativa

1. Il parser legge il sorgente applicativo e il sidecar UI opzionale.
2. L'analisi verifica identificatori, tipi, risorse e contratti UI.
3. Un modello semantico unico alimenta tutti i generatori.
4. Il target Rust produce router Axum, modelli SeaORM e query server-side.
5. Il target React produce shell responsive, pagine CRUD e data table.
6. Il target SQL produce lo schema e le migrazioni SQLite.
7. Lo smoke test avvia backend e frontend generati e verifica le API reali.

## Runtime compatto

Il runtime in `axl-core-rs` rimane distinto dal compilatore applicativo. Esegue
Compact Source 2 e AX-IR per binding, flow, funzioni, agenti, workflow, memoria e
primitive. La separazione consente di evolvere il modello applicativo senza
accoppiare la sintassi alle librerie host.

## Confini tecnologici

| Livello | Contratto AXL | Implementazione corrente |
|---|---|---|
| Dati | entità, campi, relazioni | SQLite + SeaORM |
| Rete | risorse e operazioni CRUD | Axum + Tower HTTP |
| UI | viste, componenti, proprietà | React + Refine + MUI |
| Tabelle | colonne, priorità, densità | TanStack Table |
| Icone | significato semantico | Lucide React |
| Agenti | capability, memoria, workflow | runtime Rust |

Le librerie host sono sostituibili. AXL deve conservare semantica, tipi e
intenzione, non replicare direttamente le loro API.
