# Documentazione AXL

[Italiano](README.md) · [English](en/README.md)

**AX — Agent eXecution** è l'ecosistema agent-native. AXL è il core eseguibile corrente, con sorgente canonico compatto e deterministico.

Questa documentazione distingue sempre:

- **stato attuale**: Compact Source 2 e runtime di riferimento Python, versione `2.0.0.dev0`;
- **architettura target**: AX-HIR/AX-MIR, runtime Rust e backend/bridge native, VM, WASM e piattaforma;
- **piattaforme future**: backend, browser, desktop/mobile e grafica.

## Indice

1. [AX: ecosistema e tassonomia](ax-ecosystem.md)
2. [Visione e obiettivi](overview.md)
3. [Architettura dello stack](architecture.md)
4. [Compact Source 2](compact-syntax.md)
5. [Guida al linguaggio](language-guide.md)
6. [Agenti, workflow, tool e memoria](agent-runtime.md)
7. [AX-IR e compatibilità](ax-ir.md)
8. [Sicurezza e modello delle capability](security.md)
9. [Toolchain e utilizzo](toolchain.md)
10. [Roadmap](roadmap.md)
11. [Demo applicative e piattaforme](platform-demo-analysis.md)
12. [Sviluppo e contribuzione](development.md)
13. [Glossario](glossary.md)

La specifica normativa corrente rimane in [`../SPEC.md`](../SPEC.md). Questa cartella descrive il progetto e il suo disegno complessivo in modo leggibile; in caso di conflitto sul comportamento implementato prevalgono specifica, schema AX-IR e test.
