# Visione AXL

## Solo per agenti

AXL — **Agent eXecution Language** — è un linguaggio general-purpose progettato esclusivamente per agenti software. Non cerca familiarità con Python, Rust o JavaScript. La priorità è ridurre token, ambiguità e costo di generazione.

Il sorgente canonico è un flusso versionato:

```axl
2;10|x|#2,#3,#4,*,+|i;12|$x
```

Non richiede newline, indentazione o parole chiave lunghe. Opcode numerici descrivono le istruzioni; le espressioni RPN eliminano parentesi e precedenza sintattica.

## Obiettivo general-purpose

Compatto non significa limitato. AXL deve esprimere:

- backend, API, servizi e applicazioni distribuite;
- browser e frontend tramite WASM/DOM;
- desktop e mobile native;
- CLI, automazione e software di sistema;
- GPU, grafica, audio e giochi;
- agenti, workflow, memoria, task, eventi e modelli;
- librerie e componenti riutilizzabili.

## Backend e bridge

Rust è il primo runtime/compiler di basso livello per sicurezza, performance e portabilità. AXL non è legato a Rust: AX-HIR/AX-MIR e un ABI versionato permetteranno backend e bridge multipli.

```text
AXL → AX-HIR → AX-MIR → Rust/native
                       → VM
                       → WASM/WASI
                       → C ABI
                       → DOM/WebGPU
                       → mobile/desktop/OS bridge
                       → backend futuri
```

Il sorgente AXL non cambia quando cambia il target.

## Principi

1. **Agent-only:** token-efficiency prima della leggibilità umana.
2. **General-purpose:** nessuna categoria di software esclusa dall'architettura.
3. **Determinismo:** parsing e compilazione non dipendono da LLM.
4. **Canonicalità:** una rappresentazione normale facilita hash, cache e firma.
5. **Capability security:** effetti esterni negati per default.
6. **IR versionata:** sorgente, HIR, MIR e ABI evolvono con contratti espliciti.
7. **Portabilità:** stessi risultati, effetti ed errori su backend diversi.
8. **Diagnostica macchina:** errori stabili, localizzabili e correggibili automaticamente.

## Stato reale

### Disponibile

- Compact Source 2 a singola riga;
- writer canonico e `axl pack`;
- parser deterministico e RPN;
- tipi base, funzioni e moduli;
- controllo di flusso;
- agenti, workflow e tool;
- AM scoped con SQLite, TTL e metadata;
- policy, approval, audit e budget;
- AX-IR JSON versionata;
- reference runtime Python e CLI.

### Da costruire

- collezioni, record, enum, option/result e pattern matching;
- task, eventi, async e concorrenza strutturata;
- AX-HIR/AX-MIR e ABI capability;
- runtime/compiler Rust;
- VM, native, WASM/WASI;
- bridge filesystem, rete, HTTP, database, DOM, GPU, desktop/mobile;
- package manager, LSP, debugger e profiler;
- framework applicativi.

La reference Python fissa semantica e conformità; non è il runtime finale.
