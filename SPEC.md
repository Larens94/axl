# AXL 2 compact-source development specification

## 1. Identità e obiettivo

AXL — Agent eXecution Language — è un linguaggio general-purpose progettato esclusivamente per agenti software. Non ottimizza la leggibilità umana: ottimizza token, determinismo, generazione, validazione, hashing e correzione automatica.

AXL deve poter esprimere qualsiasi categoria di software. Rust è il primo runtime/backend di basso livello, non il solo target. Bridge e backend versionati devono permettere native, server, browser/WASM, desktop, mobile, GPU, sistemi operativi e future piattaforme senza cambiare la semantica sorgente.

## 2. Pipeline

```text
AXL Compact Source 2
→ parser deterministico
→ AST/AX-HIR tipizzata
→ validazione + type-check
→ AX-MIR
→ runtime/compiler backend
→ Rust/native | VM | WASM | bridge piattaforma
```

La reference implementation Python definisce oggi la semantica e il corpus di conformità. Non è il runtime finale.

## 3. Sorgente canonico

```ebnf
source = "2", { ";", frame } ;
frame  = opcode, { "|", field } ;
```

Le righe e l'indentazione non hanno significato. `;`, `|` e `,` sono separatori strutturali fuori dalle stringhe JSON. Il formato completo è normato in [`docs/compact-syntax.md`](docs/compact-syntax.md).

### Opcode 2.0

| Opcode | Operazione |
|---:|---|
| 1 | import modulo |
| 10 | binding |
| 11 | return |
| 12 | emit |
| 20 | memory write |
| 21 | forget |
| 30 | if |
| 31 | else |
| 32 | while |
| 40 | function |
| 50 | agent |
| 51 | workflow |
| 52 | run |
| 99 | end block |

### Espressioni

Le espressioni sono RPN e delimitate da virgole. Atomi: `#int`, `"string"`, `?1`, `?0`, `$variable`, `@memory`. Operatori: `+ - * / = ! > < G L`. Chiamate postfix: `^function/arity` e `!tool/arity`. `~arity` costruisce una lista dai valori sullo stack.

Il formato legacy keyword-based è soltanto un frontend di migrazione/debug. `axl pack` produce il sorgente canonico.

## 4. Tipi e operatori

Valori correnti: string, integer, boolean e liste omogenee immutabili. Codici sorgente: `s`, `i`, `b`; il prefisso ricorsivo `l` forma `list<T>` (`li`, `ls`, `lb`, `lli`) fino a 16 livelli. Coercizioni host sono vietate.

- `+`: integer addition o string concatenation con tipi identici;
- `-`, `*`, `/`, ordering: interi;
- `/`: errore su zero o risultato frazionario;
- equality: tipi runtime identici;
- condizioni: booleani;
- risultati tool esterni al value algebra: errore.

Le funzioni dichiarano parametri e ritorno tipizzati, hanno frame locale isolato e profondità limitata. Arity, tipi, ritorni mancanti e riferimenti sconosciuti sono errori statici.

## 5. Moduli

`1|alias|relative-path` importa dichiarazioni funzione. I path sono relativi all'importatore. Alias duplicati, cicli, moduli mancanti ed effetti top-level nei moduli sono errori. I namespace qualificano le chiamate (`^math.add/2`).

## 6. Agenti e workflow

Un agente è un principal con grants tool espliciti e scope locale. Un workflow è un blocco sequenziale che esegue agenti/workflow. Riferimenti sconosciuti e cicli vengono rifiutati prima degli effetti.

Una tool call riesce solo se:

1. l'host registra la capability;
2. l'agente la dichiara nei grants;
3. la policy autorizza l'effetto;
4. l'eventuale approvazione restituisce esattamente `True`;
5. i budget consentono la chiamata;
6. il risultato appartiene al value algebra.

## 7. AM — memoria

AM è provider-agnostic. Lo scope è host-owned. Ogni record contiene chiave, scope, valore tipizzato, versione, confidence, source, timestamp e TTL opzionale. Expiry è verificata in lettura. `forget` è idempotente e scoped.

## 8. Capability e bridge

Filesystem, network, HTTP, database, modelli, UI, GPU e API OS non sono keyword vendor-specific. Sono contratti capability tipizzati abbassati verso adapter/bridge host.

Un bridge dichiara:

- ABI e versione;
- tipi accettati/prodotti;
- effetti e capability richieste;
- target supportati;
- limiti, cancellazione e comportamento d'errore.

Il sorgente AXL resta indipendente dall'implementazione Rust, C ABI, WASI, JVM, JavaScript host o futuro backend.

## 9. Policy, audit e budget

Tool deny-by-default. Decisioni approval-required, approved, denied, executed e failed sono auditabili. Segreti non devono comparire in source, IR, output o audit arguments.

Budget positivi limitano step/espressioni, profondità chiamate, byte e nodi dei valori intermedi, profondità delle collezioni, byte della serializzazione canonica dei valori in output (escluso il delimitatore di riga), tool call e operazioni memoria. Plugin host bloccanti richiedono isolamento e timeout esterni.

## 10. AX-IR 1.2

Envelope:

```json
{"ir_version":"1.2","program":{"type":"Program","instructions":[]}}
```

Decoder strict: versioni, nodi, campi, tipi, placement, riferimenti e cicli sono validati. AX-IR 1.0 e 1.1 restano leggibili. Schemi pubblicati sono immutabili.

## 11. Compatibilità

- stesso Compact Source 2 → stessa semantica;
- writer canonico → rappresentazione stabile;
- reference runtime e backend ottimizzati → equivalenza osservazionale;
- nuovi target → stessi effetti, errori e confini capability;
- cambi incompatibili al source o all'IR → nuova versione esplicita.
