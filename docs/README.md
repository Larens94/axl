# Documentazione AXL

**AXL — Agent eXecution Language** è un linguaggio di programmazione general-purpose agent-native in sviluppo.

Questa documentazione distingue sempre:

- **stato attuale**: frontend e runtime di riferimento Python, versione `1.1.0.dev0`;
- **architettura target**: compilatore/runtime Rust, AX-IR multilivello, target native e WebAssembly;
- **piattaforme future**: backend, browser, desktop/mobile e grafica.

## Indice

1. [Visione e obiettivi](overview.md)
2. [Architettura dello stack](architecture.md)
3. [Guida al linguaggio](language-guide.md)
4. [Agenti, workflow, tool e memoria](agent-runtime.md)
5. [AX-IR e compatibilità](ax-ir.md)
6. [Sicurezza e modello delle capability](security.md)
7. [Toolchain e utilizzo](toolchain.md)
8. [Roadmap](roadmap.md)
9. [Sviluppo e contribuzione](development.md)
10. [Glossario](glossary.md)

La specifica normativa corrente rimane in [`../SPEC.md`](../SPEC.md). Questa cartella descrive il progetto e il suo disegno complessivo in modo leggibile; in caso di conflitto sul comportamento implementato prevalgono specifica, schema AX-IR e test.
