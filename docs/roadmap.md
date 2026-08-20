# Roadmap AXL

AXL procede per vertical slice: sorgente compatto → HIR/MIR → runtime → risultato osservabile.

## Stato

| Area | Stato |
|---|---|
| Compact Source 2 + RPN | funzionante |
| Writer canonico + `axl pack` | funzionante |
| Reference runtime Python | funzionante |
| AX-IR JSON 1.0/1.1 | funzionante |
| Funzioni, moduli, AM, agenti | funzionante, nucleo iniziale |
| Collezioni e tipi utente | prossimo |
| AX-HIR/AX-MIR | pianificato |
| Runtime/compiler Rust | pianificato |
| Native/VM/WASM | pianificato |
| Bridge piattaforma | pianificato |

## M1 — Compact Core

- stabilizzare opcode Source 2;
- source span per frame/token;
- liste, mappe, tuple;
- record/struct, enum;
- option/result, pattern matching;
- iterazione e mutabilità definite.

**Gate:** programmi general-purpose non banali in formato esclusivamente compatto.

## M2 — Moduli e package

- export/import compact;
- manifest e lockfile machine-first;
- resolver deterministico;
- package firmati e cache content-addressed.

**Gate:** applicazione multi-package riproducibile offline.

## M3 — AX-HIR, AX-MIR e ABI

- HIR tipizzata;
- lowering verso CFG MIR;
- ABI valori, effetti e capability;
- source mapping ai frame compact;
- equivalenza con reference runtime.

**Gate:** stesso corpus su tree-walk e MIR.

## M4 — Rust runtime e VM

- workspace Rust;
- decoder/validator;
- VM budgeted e cancellabile;
- AM, policy, approval e audit compatibili;
- adapter capability isolati.

**Gate:** suite Python/Rust senza divergenze.

## M5 — Backend e bridge base

- native e WASM/WASI;
- Rust/C ABI;
- filesystem, processi, clock, random e networking;
- sistema bridge versionato e discovery target.

**Gate:** stessa CLI su VM, native e WASM.

## M6 — Backend applicativo

- HTTP client/server;
- routing e middleware;
- database e transazioni;
- serialization, config e observability.

**Gate:** servizio backend deployabile scritto in AXL.

## M7 — Web

- DOM/Web APIs bridge;
- component model e stato;
- WASM/browser build;
- storage, worker e WebGPU.

**Gate:** web app full-stack principalmente AXL.

## M8 — Native, mobile e grafica

- windowing, input, audio;
- GPU/rendering;
- packaging desktop;
- bridge Android/iOS;
- asset pipeline.

**Gate:** app grafica multipiattaforma.

## M9 — Agent platform

- task, eventi e scheduler DAG;
- concorrenza strutturata;
- checkpoint/suspend/resume;
- model provider capability;
- memoria semantic/vector/graph;
- policy distribuite.

**Gate:** workflow durevole e auditabile.

## M10 — Ecosistema

- optimizer token/source e MIR;
- LSP machine-oriented;
- debugger, profiler e trace;
- registry package/bridge;
- self-hosting progressivo.

## Regola

Ogni feature richiede opcode canonico, test RED→GREEN, specifica, round-trip source/IR, esempio reale e compatibilità backend.
