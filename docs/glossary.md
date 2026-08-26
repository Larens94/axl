# Glossario

- **AXL Compact Source:** stream sorgente canonico a opcode, ottimizzato per agenti e token.
- **AXL legacy frontend:** sintassi keyword-based temporanea per migrazione/debug.
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
- **Runtime di riferimento:** implementazione Rust che esegue Compact Source e AX-IR.
- **Compilatore applicativo:** frontend Rust che genera target Rust, React e SQL.
- **Scope memoria:** confine host-controlled che isola record persistenti.
- **Budget:** limite applicato a step, valori, output, tool, memoria o profondità.
- **Provider:** implementazione esterna di modello, memoria, database o servizio; non fa parte della grammatica.
