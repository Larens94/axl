# Glossario

- **AXL — Agent eXecution Language:** linguaggio general-purpose agent-native completo.
- **AX-IR:** famiglia di rappresentazioni intermedie tipizzate e versionate di AXL.
- **AX-HIR:** futura IR di alto livello, vicina alla semantica AXL.
- **AX-MIR:** futura IR abbassata per VM, ottimizzazione e code generation.
- **AM:** modulo memoria provider-agnostic, scoped e persistente.
- **Agent:** principal eseguibile con scope e capability esplicite.
- **Workflow:** composizione di agenti o altri workflow.
- **Tool:** capability implementata dall'host e invocabile tramite `call`.
- **Capability:** autorizzazione limitata a compiere una classe di effetti.
- **Approval:** consenso pre-effetto richiesto da una policy.
- **Audit:** traccia delle decisioni e degli esiti di capability/approval.
- **Reference runtime:** implementazione Python usata per fissare la semantica.
- **Runtime Rust:** implementazione target per VM, native, WASM e piattaforme.
- **Scope memoria:** confine host-controlled che isola record persistenti.
- **Budget:** limite applicato a step, valori, output, tool, memoria o profondità.
- **Provider:** implementazione esterna di modello, memoria, database o servizio; non fa parte della grammatica.
