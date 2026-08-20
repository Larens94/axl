# AX — Agent eXecution

## Ruolo

**AX** è l’ecosistema ombrello per definire, compilare ed eseguire software agent-native. Il repository AXL contiene oggi il core eseguibile di riferimento: Compact Source 2, parser, validazione, type-check, AX-IR e runtime Python.

Le superfici specializzate elencate qui definiscono la tassonomia target. Non sono dichiarate implementate finché non dispongono di parser, lowering e test di conformità propri.

## Linguaggi e moduli

| Sigla | Nome | Responsabilità |
|---|---|---|
| AA | Agent Definition | identità operativa, obiettivi, grants e lifecycle degli agenti |
| AM | Memory | memoria, scope, routing, retention e provider |
| AW | Workflow | orchestrazione, dipendenze, task e compensazioni |
| AP | Policy | capability, decisioni, approval e vincoli |
| AT | Tools | contratti tool, MCP, effetti e adapter |
| AE | Events | eventi, stream, subscription e delivery |
| AS | State | stato applicativo, transizioni e sincronizzazione |
| AD | Data | modelli, schema, query, storage e migrazioni |
| AI | Identity | principal, autenticazione, autorizzazione e credenziali indirette |

`AI` significa **Identity** nel namespace AX. La documentazione deve scrivere il nome esteso quando il contesto può confonderlo con “Artificial Intelligence”.

## Core condiviso

- AX-IR, con evoluzione separata verso AX-HIR e AX-MIR;
- compiler e parser;
- runtime e scheduler;
- sandbox e Capability ABI;
- Context Engine;
- Memory Router;
- Model Router;
- Policy Engine;
- Tool/MCP Router.

## Servizi di piattaforma

- secrets tramite riferimenti host, mai valori nel sorgente o nell’IR;
- audit, tracing ed evals;
- versioning e compatibilità;
- registry e package manager;
- SDK Python e TypeScript;
- core Rust e target WASM;
- CLI e API;
- adapter web, mobile, desktop e server.

## Pipeline

```text
AA | AM | AW | AP | AT | AE | AS | AD | AI
                    ↓
                  AX-IR
                    ↓
       runtime + scheduler + sandbox
                    ↓
        adapter/provider versionati
                    ↓
 web | mobile | desktop | server | sistemi esistenti
```

I linguaggi specializzati non accedono direttamente all’host. Ogni effetto passa attraverso IR tipizzata, policy, capability e adapter versionati.

## Stato implementato

Alla versione di sviluppo corrente sono reali e testati:

- AXL Compact Source 2;
- AX-IR 1.0, 1.1 e 1.2;
- runtime Python sincrono di riferimento;
- tipi scalari e `list<T>` omogenee;
- funzioni, moduli, AM iniziale, agenti, workflow, tool e policy.

Scheduler async, runtime Rust/WASM, router completi, registry, SDK e adapter multipiattaforma restano milestone successive.
