# AXL Compact Source 2

AXL Compact Source 2 è la sintassi canonica del linguaggio. È progettata esclusivamente per agenti e sistemi automatici: minima quantità di token, parsing deterministico, nessuna indentazione e nessuna dipendenza dalle righe.

## Forma

```text
2;frame;frame;frame
```

- `2` è la versione del formato sorgente;
- `;` separa i frame;
- `|` separa i campi;
- `,` separa i token di un'espressione RPN;
- `99` chiude un blocco;
- spazi, indentazione e newline non hanno significato strutturale;
- le stringhe usano JSON escaping, quindi possono contenere separatori.

Esempio completo su una riga:

```axl
2;10|x|#2,#3,#4,*,+|i;12|$x
```

Equivale semanticamente a: calcola `2 + 3 * 4`, assegna a `x: int`, emette `x`.

## Espressioni RPN

Le espressioni usano Reverse Polish Notation. Non servono parentesi né regole di precedenza.

| Token | Significato |
|---|---|
| `#42` | intero |
| `"AXL"` | stringa JSON |
| `?1`, `?0` | booleani true/false |
| `$x` | variabile locale |
| `@k` | recall memoria |
| `+ - * /` | aritmetica |
| `= ! > < G L` | `== != > < >= <=` |
| `^f/2` | chiama funzione `f` con 2 valori dallo stack |
| `!t/1` | chiama tool `t` con 1 valore dallo stack |
| `~3` | costruisce una lista con 3 valori dallo stack |

```text
#2,#3,#4,*,+      → 2 + (3 * 4)
"AX","L",+       → "AXL"
#7,#8,^add/2      → add(7,8)
"AXL",!search/1  → tool search("AXL")
#1,#2,#3,~3       → list<int>[1,2,3]
```

## Tipi

| Codice | Tipo |
|---|---|
| `i` | int |
| `s` | string |
| `b` | bool |
| `li` | list&lt;int&gt; |
| `ls` | list&lt;string&gt; |
| `lb` | list&lt;bool&gt; |

Il prefisso `l` è ricorsivo fino a 16 livelli: `lli` rappresenta `list<list<int>>`. Le liste sono immutabili e omogenee; `~0` crea una lista vuota il cui tipo elemento viene determinato dal contesto dichiarato.

## Opcode

| Opcode | Frame | Semantica |
|---:|---|---|
| `1` | `1|alias|path` | import `.axl` relativo e top-level, confinato al module root |
| `10` | `10|name|expr[|type]` | binding locale |
| `11` | `11|expr` | return |
| `12` | `12|expr` | emit |
| `20` | `20|key|expr[|confidence|ttl|source]` | write memoria |
| `21` | `21|key` | forget |
| `30` | `30|condition` | apre if |
| `31` | `31` | else |
| `32` | `32|condition` | apre while |
| `40` | `40|name|a:i,b:i|i` | apre funzione tipizzata |
| `50` | `50|name[|tool,tool]` | apre agente con grants |
| `51` | `51|name` | apre workflow |
| `52` | `52|name` | run agente/workflow |
| `99` | `99` | chiude il blocco corrente |

### Memoria con metadata

```axl
2;20|finding|"AXL"|95|3600|researcher;12|@finding;21|finding
```

Per TTL assente si usa `-`:

```axl
2;20|k|"v"|100|-|runtime
```

### Funzione

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99;12|#20,#22,^add/2
```

### Controllo di flusso

```axl
2;10|n|#0;32|$n,#3,<;30|$n,#1,=;12|"one";31;12|$n;99;10|n|$n,#1,+;99
```

### Agente e workflow

```axl
2;50|r|search;10|x|"AXL",!search/1;12|$x;99;51|w;52|r;99;52|w
```

## Canonicalizzazione

Il comando `pack` converte la sintassi legacy nella rappresentazione canonica:

```bash
axl pack legacy.axl -o packed.axl
```

Il writer canonico garantisce una sola rappresentazione normale per il programma supportato. Questo facilita hashing, cache, deduplicazione, firma e confronto automatico.

## Formato legacy

Il parser testuale leggibile (`let`, `fn`, `agent`, indentazione visuale) resta temporaneamente disponibile come frontend di migrazione e debug. Non è la sintassi primaria e non definisce il design futuro di AXL.

## Evoluzione

Nuovi opcode possono essere aggiunti senza rendere verbose il linguaggio. Le famiglie future includeranno collezioni, record, enum, task, eventi, concorrenza, capability standard e binding piattaforma. La versione iniziale impedisce interpretazioni ambigue; cambi incompatibili richiedono un nuovo header sorgente.
