# Documentazione AXL

[Italiano](README.md) · [English](en/README.md)

**AXL — Agent eXecution Language** è il linguaggio sorgente principale per
descrivere dominio, dati, API, interfaccia e agenti. Rust, React/TypeScript e SQL
sono i primi target applicativi; il runtime Rust esegue inoltre Compact Source e
le primitive native.

> Release corrente: `0.1.0-alpha.1`. È una proof of concept funzionante, non una
> promessa di compatibilità stabile o di deployment production-ready.

[Apri la presentazione interattiva Claude × Apple](presentation.html)

## Cosa è disponibile

- parser, analisi e generazione applicativa in Rust;
- backend Axum + SeaORM con SQLite, migrazioni e API CRUD;
- frontend React + Refine + Material UI + TanStack Table + Lucide;
- Compact UI Source 3 con frame numerici per viste, componenti e proprietà;
- runtime Rust per Compact Source 2, primitive, memoria, policy e rendering web;
- demo CRM con 6 entità, 30 operazioni CRUD e 7 viste compatte;
- portale documentale bilingue e 43 test Rust verificati per la release.

## Pipeline

```text
crm.axl + crm.ui.axl
→ parser e modello semantico tipizzato
→ Rust/Axum/SeaORM + React/Refine/MUI + SQL
→ applicazione CRM eseguibile
```

## Indice

1. [AX: ecosistema e tassonomia](ax-ecosystem.md)
2. [Visione e obiettivi](overview.md)
3. [Architettura dello stack](architecture.md)
4. [Sorgente compatto](compact-syntax.md)
5. [Guida al linguaggio](language-guide.md)
6. [Agenti, workflow, tool e memoria](agent-runtime.md)
7. [AX-IR e compatibilità](ax-ir.md)
8. [Sicurezza e capability](security.md)
9. [Toolchain e utilizzo](toolchain.md)
10. [Roadmap](roadmap.md)
11. [CRM e copertura UI](platform-demo-analysis.md)
12. [Sviluppo e contribuzione](development.md)
13. [Glossario](glossary.md)

La specifica normativa rimane in [`../SPEC.md`](../SPEC.md). In caso di conflitto
sul comportamento implementato prevalgono codice, schema AX-IR e test.
