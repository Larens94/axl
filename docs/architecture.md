# Architettura dello stack

## Pipeline target

```text
Sorgente .axl
    │
    ▼
Lexer + parser deterministico ──► diagnostica con source span
    │
    ▼
AST
    │
    ▼
Resolver moduli + type-checker
    │
    ▼
AX-HIR (semantica di alto livello)
    │
    ├──► primitive agentiche: agent, workflow, memory, capability
    ▼
AX-MIR (controllo di flusso, chiamate, tipi abbassati)
    │
    ├──► interprete/VM
    ├──► backend native
    └──► backend WebAssembly
          │
          ├── browser/DOM/WebGPU
          └── runtime embedded
```

## Livelli

### 1. AXL Source

Sintassi leggibile e stabile. Contiene dichiarazioni, espressioni e primitive agentiche, ma non dettagli specifici di provider o sistema operativo.

### 2. Frontend

Responsabilità:

- tokenizzazione e parsing;
- source span e diagnostica;
- risoluzione di moduli e namespace;
- controllo statico dei tipi;
- costruzione di una rappresentazione semantica valida.

Oggi questo livello è implementato in Python come riferimento.

### 3. AX-IR

**AX-IR** è la famiglia di rappresentazioni intermedie tipizzate e versionate.

- L'attuale IR JSON è un contratto interoperabile e ad alto livello.
- La futura **AX-HIR** conserverà concetti come funzioni, agenti, workflow e capability.
- La futura **AX-MIR** rappresenterà blocchi di base, controllo di flusso, layout dei valori e chiamate abbassate.

Separare HIR e MIR evita di usare la sintassi come formato runtime e consente ottimizzazione e backend multipli.

### 4. Runtime

Il runtime governa:

- esecuzione e scheduling;
- memoria e scope;
- capability, policy e approvazioni;
- audit, budget e cancellazione;
- binding con filesystem, rete, database, modelli e piattaforme.

Il runtime Python corrente dimostra la semantica. Il runtime definitivo sarà principalmente Rust.

### 5. Backend e piattaforme

Target previsti:

- **native**: CLI, server, desktop, servizi e integrazioni di sistema;
- **WASM**: browser, edge ed embedding sandboxed;
- **binding di piattaforma**: DOM, WebGPU, mobile, desktop, GPU e API OS;
- **FFI**: Rust/C ABI per funzionalità di basso livello.

## Componenti correnti del repository

```text
axl/parser.py          parser sorgente
axl/compiler.py        import e namespace
axl/ir.py              IR tipizzata corrente
axl/typechecker.py     type-checker statico
axl/validation.py      validazione strutturale/semantica
axl/serialization.py   AX-IR JSON e compatibilità
axl/interpreter.py     reference runtime
axl/memory.py          AM e adapter memoria
axl/policy.py          tool, effetti, approval e audit
axl/__main__.py        CLI
schema/                schemi JSON AX-IR pubblicati
tests/                 corpus di conformità
```

## Regola di compatibilità

Una versione AX-IR pubblicata è immutabile. Nuovi nodi o campi obbligatori richiedono una nuova versione, un nuovo schema e decoder legacy testati.
