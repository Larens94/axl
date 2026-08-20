# Architettura dello stack

## Pipeline stabile

```text
Compact Source 2
      │
      ▼
parser deterministico
      │
      ▼
AST + resolver + type-checker
      │
      ▼
AX-HIR ── primitive general-purpose + agentiche
      │
      ▼
AX-MIR ── CFG, tipi abbassati, effetti, capability ABI
      │
      ├── VM
      ├── Rust/native
      ├── WASM/WASI
      └── bridge piattaforma
             ├── filesystem/network/HTTP/database
             ├── DOM/browser/WebGPU
             ├── desktop/mobile/OS
             └── futuri backend
```

## Source layer

Il sorgente canonico è ottimizzato per agenti: opcode numerici, frame delimitati, espressioni RPN, nessuna indentazione. È compatto ma versionato e completamente deterministico.

Il frontend verbose esistente serve soltanto a migrazione, debug e conversione con `axl pack`.

## Frontend

Responsabilità:

- framing e parsing strict;
- diagnostica per frame/token;
- risoluzione moduli e namespace;
- type-check statico;
- costruzione di HIR valida;
- nessun effetto runtime.

Oggi il frontend/reference runtime è Python. Il corpus di test ne rende la semantica trasferibile.

## AX-IR, HIR e MIR

- **AX-IR JSON 1.x:** contratto interoperabile corrente.
- **AX-HIR:** funzioni, tipi, agenti, workflow, memoria ed effetti di alto livello.
- **AX-MIR:** basic block, controllo di flusso, layout valori, chiamate e capability abbassate.

Separare i livelli consente ottimizzazione e target multipli senza contaminare il sorgente.

## Runtime

Il runtime governa:

- esecuzione, scheduling e cancellazione;
- memoria AM e scope;
- capability, policy e approval;
- audit e budget;
- bridge con host e piattaforme.

Rust è la prima implementazione definitiva prevista per safety e performance. Non è parte della sintassi né l'unico backend ammesso.

## Capability ABI e bridge

Ogni bridge espone contratti tipizzati e versionati:

```text
capability id + ABI version + input/output types + effects + target + cancellation
```

Questo permette a uno stesso programma AXL di usare implementazioni diverse per Linux, Windows, macOS, browser, Android, iOS, GPU o cloud.

## Componenti correnti

```text
axl/compact.py        parser/writer Compact Source 2
axl/parser.py         dispatcher + frontend legacy
axl/compiler.py       moduli e namespace
axl/ir.py             IR tipizzata corrente
axl/typechecker.py    type-checker
axl/validation.py     validazione semantica
axl/serialization.py  AX-IR JSON
axl/interpreter.py    reference runtime
axl/memory.py         AM
axl/policy.py         capability policy/audit
axl/__main__.py       CLI
```

## Compatibilità

- source versionata;
- output canonico stabile;
- schema IR pubblicato immutabile;
- decoder legacy testato;
- equivalenza osservazionale tra reference, VM, native e WASM;
- bridge sostituibili senza cambiare il programma.
