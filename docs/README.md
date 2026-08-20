# Documentazione AXL

**AXL — Agent eXecution Language** è un linguaggio general-purpose esclusivamente agent-native, con sorgente canonico compatto e deterministico.

Questa documentazione distingue sempre:

- **stato attuale**: Compact Source 2 e runtime di riferimento Python, versione `2.0.0.dev0`;
- **architettura target**: AX-HIR/AX-MIR, runtime Rust e backend/bridge native, VM, WASM e piattaforma;
- **piattaforme future**: backend, browser, desktop/mobile e grafica.

## Indice

1. [Visione e obiettivi](overview.md)
2. [Architettura dello stack](architecture.md)
3. [Compact Source 2](compact-syntax.md)
4. [Guida al linguaggio](language-guide.md)
5. [Agenti, workflow, tool e memoria](agent-runtime.md)
6. [AX-IR e compatibilità](ax-ir.md)
7. [Sicurezza e modello delle capability](security.md)
8. [Toolchain e utilizzo](toolchain.md)
9. [Roadmap](roadmap.md)
10. [Sviluppo e contribuzione](development.md)
11. [Glossario](glossary.md)

La specifica normativa corrente rimane in [`../SPEC.md`](../SPEC.md). Questa cartella descrive il progetto e il suo disegno complessivo in modo leggibile; in caso di conflitto sul comportamento implementato prevalgono specifica, schema AX-IR e test.
