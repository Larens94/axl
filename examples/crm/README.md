# AXL CRM

Esempio full-stack definito da [`crm.axl`](crm.axl). AXL è il sorgente; il
compilatore genera backend Rust/Axum/SeaORM, frontend React/Refine/MUI e schema
SQLite.

Il layout della dashboard non usa più i vecchi blocchi `ui { ... }`: è
programmato in [`crm.ui.axl`](crm.ui.axl) tramite Compact Source v3. Gli opcode
`60`, `61`, `62`, `63` e `99` descrivono rispettivamente view, nodi, proprietà,
eventi e chiusure. I componenti numerici `64` e `65` definiscono data table e
colonne: page size, densità, modalità mobile, tipo visuale, priorità responsive e
larghezza vengono validati dal compilatore. I binding come `$customers.total`
diventano query React reali.

## Build

La demo completa parte con un comando (genera Rust, React e SQL, installa le
dipendenze frontend al primo avvio e mantiene entrambi i server attivi):

```bash
cargo build -p axl-cli
target/debug/axl dev examples/crm/crm.axl
```

Aprire `http://localhost:5173/register`. I record dimostrativi dichiarati con
`seed` in `crm.axl` vengono caricati in modo idempotente.

Per separare build e avvio:

```bash
cargo build -p axl-cli
target/debug/axl check examples/crm/crm.axl
target/debug/axl build examples/crm/crm.axl -o build/crm
```

Backend:

```bash
cp build/crm/backend/.env.example build/crm/backend/.env
# sostituire JWT_SECRET con un valore casuale
cargo run --manifest-path build/crm/backend/Cargo.toml
```

Frontend, in un secondo terminale:

```bash
cd build/crm/frontend
npm install
npm run dev
```

Aprire `http://localhost:5173/register`, creare il primo utente e accedere al
CRM. Vite inoltra `/api` al backend sulla porta 3000.

## Verifica end-to-end

Dopo aver avviato lo stack:

```bash
build/crm/smoke-test.sh
```

Il test verifica il rifiuto delle API senza token, registrazione, JWT e lettura
dei dati CRM seed. Verifica inoltre pagination, ricerca full-text sui campi
testuali, filtri esatti e ordinamento server-side prodotti dalla primitiva
`query page ... max ... sort ...`. Le API CRUD sono protette dal middleware `jwt_required`
dichiarato in AXL; il client React allega automaticamente il Bearer token.

## Sezioni

- dashboard con KPI, pipeline, attività e task;
- clienti con lifecycle e owner;
- lead con scoring, owner e prossima azione;
- trattative con stage, probabilità, owner, prossima azione e data di chiusura;
- attività;
- task collegati ai record CRM;
- note;
- registrazione e login Argon2/JWT;
- pagine dichiarative Reports e Settings nella specifica UI.

## Copertura UI kit admin

La baseline comprende 20 primitive necessarie a un pannello amministrativo.
Il CRM le copre tutte, quindi la copertura è **100%** (oltre il gate del 70%).

| Categoria | Componenti coperti |
|---|---|
| Layout | app shell, sidebar, bottom navigation, bottom sheet, topbar |
| Dati | stat card, card mobile, TanStack data table, pagination, badge, progress |
| Controlli | search, select/filter, text input, button |
| Feedback | alert, tabs, skeleton, empty state, snackbar/notification provider |

Dialog, checkbox, textarea e date input completano inoltre i flussi di form. Le
icone sono token semantici AXL tradotti dal renderer nel set Lucide; il sorgente
non contiene nomi di componenti React. Le preferenze di colonne e densità sono
persistenti per risorsa.
Le pagine sono caricate on demand; il bundle framework condiviso viene separato
dal piccolo entrypoint e dalle route CRM.

## Perché questa demo conta

Il file AXL non è una configurazione decorativa: definisce dominio, tipi,
default, API, autenticazione, middleware, dati demo e superfici UI. Rust/Axum,
SeaORM/SQLite e React/Refine/MUI sono target sostituibili sotto il linguaggio.
Un agente può quindi leggere e modificare un unico contratto compatto e lasciare
al compilatore il lavoro ripetitivo, coerente e verificabile.

## Flusso CRM dimostrato

La demo conserva il contesto dal primo contatto all'esecuzione commerciale:
un lead ha score, valore, responsabile e prossimo passo; una trattativa rende
espliciti stage, probabilità, chiusura prevista e azione successiva; clienti e
task mantengono rispettivamente lifecycle/owner e collegamento al record. Questo
permette di mostrare non solo CRUD, ma una pipeline sulla quale un agente può
ragionare e agire.

Le prossime primitive di linguaggio ad alto valore sono `relation`, `enum` o
`stage`, KPI `computed`, transizioni `workflow` e scadenze `overdue`: renderebbero
relazioni, pipeline e automazioni semanticamente verificabili dal compilatore,
anziché convenzioni affidate a stringhe.
