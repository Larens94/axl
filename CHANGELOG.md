# Registro delle modifiche

[Italiano](CHANGELOG.md) · [English](CHANGELOG.en.md)

## 0.1.0-alpha.1 — 26 agosto 2026

- Aggiunto il compilatore applicativo AXL con generazione di backend Rust/Axum/SeaORM, frontend React/Refine/MUI e migrazioni SQL.
- Pubblicata la demo CRM full-stack con 6 entità, CRUD REST, dashboard, report, impostazioni e 7 viste Compact UI.
- Aggiunti i frame Compact UI Source 3 per viste, componenti, proprietà, eventi, tabelle dati e colonne tipizzate.
- Aggiunti layout responsive, icone semantiche, menu mobile inferiore, tabelle desktop e card adattive.
- Aggiunti query server-side per ricerca, filtri, ordinamento e paginazione.
- Aggiornati documentazione bilingue, GitHub Pages e presentazione interattiva Claude × Apple.
- Allineate le versioni dei crate alla pre-release e verificati 43 test Rust.
- Rimossa una credenziale MiMo hardcoded: il backend ora richiede `MIMO_API_KEY` dall'ambiente e fallisce in modo sicuro se assente.

## 2.0.0 — in sviluppo

- Aggiunto Compact Source 2 come sintassi canonica destinata esclusivamente agli agenti.
- Aggiunti frame numerici per le istruzioni, espressioni RPN e opcode per i blocchi.
- Aggiunti funzioni, moduli, memoria, agenti, workflow e chiamate ai tool in formato compatto.
- Aggiunti il writer compatto canonico e il comando di migrazione `axl pack`.
- Ridefinito Rust come primo runtime/backend all'interno di un'architettura a bridge e backend multipli.
- Ridisegnate la documentazione e le GitHub Pages attorno al modello di linguaggio compatto.
- Aggiunti valori `list<T>` omogenei e immutabili con costruzione compatta `~arity`.
- Pubblicato AX-IR 1.2 mantenendo la decodifica di AX-IR 1.0 e 1.1.
- Aggiunto il trasporto delle liste attraverso funzioni, capability dei tool, AM SQLite e output JSON canonico della CLI.
- Aggiunto il tracer `map<K,V>` tipizzato per Compact Source e runtime di riferimento.
- Pubblicato un portale documentale statico bilingue, sempre light, con navigazione laterale, ricerca, indice pagina e collegamenti italiano/inglese.

## 1.1.0 — in sviluppo

- Aggiunti funzioni tipizzate, parametri, ritorni e chiamate nelle espressioni.
- Aggiunto il controllo statico dei tipi per i contratti delle funzioni e le variabili tipizzate.
- Aggiunti scope isolati per le funzioni e profondità di ricorsione limitata.
- Estesi IR e schema JSON con i nodi funzione.
- Aggiunti import relativi dei moduli, alias, funzioni con namespace e rilevamento dei cicli.
- Pubblicato AX-IR 1.1 mantenendo la decodifica verificata di AX-IR 1.0.
- Aggiunta in `docs/` la documentazione completa su architettura, linguaggio, runtime, sicurezza, toolchain e roadmap.
- Aggiunto il testo della licenza Apache 2.0 per il repository open source pubblico.

## 1.0.0

- Aggiunti IR JSON 1.0 tipizzati e validati e i comandi CLI `compile`/`exec`.
- Aggiunti agenti, autorizzazioni ai tool, workflow e rilevamento statico dei cicli.
- Aggiunti effetti dei tool, approvazioni esplicite ed eventi di audit.
- Aggiunti memoria con scope, metadati, TTL, versionamento e cancellazione.
- Aggiunti migrazione dello schema SQLite e interfaccia di memoria indipendente dal provider.
- Aggiunti budget per espressioni, valori intermedi, output, chiamate ai tool e operazioni di memoria.
- Rafforzati tipi runtime, errori CLI, identificatori riservati e validazione IR.

## 0.3.0

- Aggiunti cicli limitati e memoria persistente SQLite.

## 0.2.0

- Aggiunti espressioni tipizzate, condizioni e tool espliciti.

## 0.1.0

- Prima versione di parser, IR, interprete e CLI.
