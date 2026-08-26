# Visione AXL

## Il sorgente principale

AXL — **Agent eXecution Language** — vuole essere il linguaggio in cui un agente
descrive un sistema intero. Rust, React, SQL e i futuri bridge non devono
sostituire AXL nella sorgente: devono implementarne il modello semantico.

```text
intenzione AXL
→ analisi deterministica
→ modello tipizzato
→ backend · frontend · dati · runtime · bridge
```

## Perché compatto

La sintassi numerica elimina keyword ripetitive, precedenza implicita e
strutture decorative. Compatto non significa su una riga: il formatter dispone i
frame su più righe per diff e revisione, mentre la forma minificata serve a
cache, hashing, firma e trasporto.

Non dichiariamo percentuali di token o moltiplicatori di velocità senza un
benchmark riproducibile. Il vantaggio dimostrato oggi è strutturale: un contratto
AXL genera più target coerenti.

## Obiettivo generale

L'architettura deve poter coprire:

- backend, API, servizi, database e networking;
- frontend web, desktop e mobile;
- CLI, automazione, sistemi e IoT;
- AI, agenti, workflow, memoria, tool ed eventi;
- grafica, GPU, audio e applicazioni specialistiche.

## Stato `0.1.0-alpha.1`

### Disponibile

- runtime e CLI Rust;
- Compact Source 2, RPN, AX-IR, funzioni, memoria, agenti e workflow;
- compilatore applicativo per entità, API, auth e seed;
- Compact UI Source 3 e registry componenti/proprietà;
- target Rust/Axum/SeaORM, React/Refine/MUI/TanStack e SQL/SQLite;
- CRM full-stack responsive e testato.

### Non ancora garantito

- compatibilità stabile del formato tra release alpha;
- hardening e deployment production-ready;
- scheduler async e concorrenza strutturata;
- target WASM/native mobile/desktop;
- package manager, LSP, debugger e SDK pubblici;
- backend LLM reali e sandbox di capability non fidate.

## Principi

1. **AXL-first:** la semantica vive nel sorgente AXL.
2. **Agent-native:** forma deterministica, diagnostica machine-readable.
3. **Target standard:** il codice generato usa ecosistemi reali e ispezionabili.
4. **Capability security:** gli effetti esterni sono espliciti e limitabili.
5. **Portabilità:** il linguaggio non copia le API di un singolo framework.
6. **Evidenza:** demo, test e benchmark devono essere riproducibili.
