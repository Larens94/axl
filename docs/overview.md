# Visione AXL

## Linguaggio Agent-Native

AXL — **Agent eXecution Language** — è un linguaggio di programmazione per agenti che wrappa Rust e tutto quello che Rust può fare. Il focus è sul linguaggio, non su app demo.

AXL 3.0 è **agent-native**: l'agente è la prima classe, non la funzione. Il reasoning LLM è nativo, gli agenti sono event-driven, la memoria è semantica.

## Obiettivo

AXL deve permettere agli agenti di costruire **qualsiasi software**:

- backend, API, servizi e database
- frontend web e WebAssembly
- app desktop e mobile native
- CLI, sistemi e automazioni
- grafica, GPU, audio e giochi
- agenti, workflow, task ed eventi

Rust è il runtime nativo. Ogni capability di Rust è esponibile come primitiva AXL.

## Stato Attuale (v3.0)

### Funzionante

- Parser compact (Source 2/3)
- 90+ primitiva native Rust (io, text, collections, math, crypto, json, network, system)
- Interpreter con tool policy, memory, budget
- LLM backend trait + MockBackend
- CLI: run, compile, exec, pack, build, serve
- 28 test passanti

### In costruito

- Keyword parser (sorgente leggibile)
- Compiler con moduli/import
- LLM backend reali (OpenAI, Anthropic)
- Comunicazione inter-agenti completa
- Event-driven execution
- Semantic memory

## Principi

1. **Agent-native:** l'agente è la prima classe, non la funzione
2. **Rust-backed:** ogni primitiva mappa a codice Rust safe
3. **Componibile:** primitiva ritornano valori che altre consumano
4. **Determinismo:** parsing e compilazione non dipendono da LLM
5. **Sicurezza:** permission system, approval, audit
6. **Completo:** 500+ primitiva mappano ogni capability di Rust
