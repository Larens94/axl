# Roadmap AXL

La roadmap procede per vertical slice verificabili. Le piattaforme applicative arrivano dopo la stabilizzazione del core.

## Stato

| Area | Stato |
|---|---|
| Parser e runtime Python | funzionante |
| AX-IR JSON 1.0/1.1 | funzionante, 1.1 in sviluppo |
| Type-checker base | funzionante |
| Funzioni e moduli | funzionante, scope iniziale |
| AM memoria scoped | funzionante |
| Agenti/workflow/tool policy | funzionante, sequenziale |
| Collezioni e tipi utente | non iniziato |
| Runtime/compilatore Rust | non iniziato |
| Native/WASM | non iniziato |
| Framework applicativi | non iniziato |

## M1 — AXL Core

- source span e diagnostica strutturata;
- liste, mappe e tuple;
- record/struct ed enum;
- option/result e gestione errori;
- pattern matching;
- iterazione generalizzata;
- contratti precisi per scope e mutabilità.

**Gate:** programmi general-purpose non banali verificati end-to-end.

## M2 — Moduli e package

- export/import espliciti;
- manifest di progetto e lockfile;
- resolver deterministico;
- versionamento dipendenze;
- package firmati e cache locale.

**Gate:** applicazione multi-package riproducibile offline da lockfile.

## M3 — AX-HIR e AX-MIR

- HIR tipizzata con source mapping;
- lowering verso CFG MIR;
- ABI dei valori e capability;
- bytecode/serialization MIR;
- equivalenza osservazionale con reference runtime.

**Gate:** stesso corpus eseguito da tree-walk e MIR con risultati identici.

## M4 — Rust runtime e VM

- workspace Rust;
- decoder/validator AX-IR;
- VM con budget e cancellazione;
- AM, policy, approval e audit compatibili;
- sandbox e adapter capability.

**Gate:** suite di conformità Python/Rust senza divergenze.

## M5 — Target WASM e native

- WASM/WASI;
- backend native iniziale;
- FFI Rust/C ABI;
- filesystem/rete/clock/random come capability;
- profiling e ottimizzazione sicura.

**Gate:** stessa applicazione CLI su VM, native e WASM.

## M6 — Standard library e backend

- HTTP client/server;
- routing, middleware e serialization;
- database e transazioni;
- processi, filesystem e networking;
- osservabilità e configurazione.

**Gate:** servizio backend AXL completo e deployabile.

## M7 — Web frontend

- binding DOM e Web APIs;
- component model e stato;
- build WASM/browser;
- networking, storage e worker;
- WebGPU.

**Gate:** web app full-stack scritta principalmente in AXL.

## M8 — Desktop, mobile e grafica

- windowing/input/audio;
- rendering GPU;
- packaging desktop;
- bridge Android/iOS;
- asset pipeline.

**Gate:** applicazione grafica multipiattaforma dimostrativa.

## M9 — Agent platform completa

- task, eventi e scheduler DAG;
- parallelismo strutturato;
- checkpoint, sospensione e ripresa;
- model providers e context management;
- memoria semantica/vector/graph;
- policy distribuite e attestazione degli effetti.

**Gate:** workflow agentico durevole, riprendibile e auditabile.

## M10 — Developer experience

- formatter canonico;
- LSP e diagnostica incrementale;
- debugger e profiler;
- documentazione generata;
- package registry;
- self-hosting progressivo.

## Regola di rilascio

Ogni milestone richiede test, lint, build/install, esempio reale, specifica aggiornata e compatibilità delle IR pubblicate.
