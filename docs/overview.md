# Visione e obiettivi

## Cos'è AXL

AXL significa **Agent eXecution Language**. L'obiettivo è un linguaggio general-purpose pensato perché agenti AI e sviluppatori possano creare, verificare ed eseguire software completo con semantica deterministica.

AXL non deve dipendere da un modello linguistico per interpretare il codice: lexer, parser, type-checker e compilazione devono produrre sempre lo stesso risultato per lo stesso input.

## Cosa vuole esprimere

A regime AXL dovrà supportare:

- backend, API, servizi e applicazioni distribuite;
- frontend web tramite WebAssembly e binding DOM/WebGPU;
- applicazioni desktop e mobile;
- CLI, automazioni e integrazioni di sistema;
- software grafico, GPU e giochi;
- agenti, workflow, memoria, eventi, task e modelli;
- librerie, package e componenti riutilizzabili;
- interoperabilità controllata con Rust, C ABI e piattaforme host.

## Principi

1. **General-purpose e agent-native.** Funzioni, moduli, tipi e strutture dati convivono con agenti, memoria e capability.
2. **Determinismo.** Nessun LLM è necessario per fare parsing o type checking.
3. **Capability security.** File, rete, shell, database e servizi sono negati per default.
4. **Effetti espliciti.** Tool sensibili richiedono policy e, quando previsto, approvazione.
5. **IR versionata.** Il sorgente viene trasformato in AX-IR tipizzata e validabile.
6. **Portabilità.** La semantica deve essere preservata tra reference runtime, Rust, native e WASM.
7. **Diagnostica per agenti.** Gli errori devono essere precisi, strutturati e correggibili automaticamente.

## Stato reale

### Disponibile oggi

- parser deterministico;
- valori `int`, `string`, `bool`;
- espressioni, condizioni e cicli limitati;
- funzioni tipizzate, parametri, ritorni e scope locali;
- import relativi con alias e namespace;
- agenti e workflow sequenziali;
- tool deny-by-default, approvazioni fail-closed e audit;
- memoria scoped in-memory/SQLite con TTL e metadata;
- AX-IR JSON versionata;
- CLI `run`, `compile`, `exec`;
- budget multidimensionali e test di conformità.

### Non ancora disponibile

- compilatore/runtime Rust;
- collezioni e tipi definiti dall'utente;
- enum, pattern matching, option/result ed error handling completo;
- async, task, eventi, parallelismo e scheduler DAG;
- standard library filesystem/rete/HTTP/database;
- backend native/WASM, DOM e grafica;
- package manager, formatter, LSP e debugger;
- framework backend/frontend/mobile/desktop.

L'implementazione Python è una **reference implementation**, non il runtime finale.
