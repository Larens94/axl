# AXL Compact Source 2

AXL Compact Source 2 è la sintassi compatta del linguaggio. Mantiene pochi token
e parsing deterministico, ma nei file viene normalmente disposta su più righe.

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

Esempio formattato:

```axl
2;
10|x|#2,#3,#4,*,+|i; 12|$x;
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
| `%2` | costruisce una mappa con 2 coppie chiave/valore dallo stack |

```text
#2,#3,#4,*,+      → 2 + (3 * 4)
"AX","L",+       → "AXL"
#7,#8,^add/2      → add(7,8)
"AXL",!search/1  → tool search("AXL")
#1,#2,#3,~3       → list<int>[1,2,3]
"a",#1,"b",#2,%2 → map<string,int>{"a":1,"b":2}
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
| `msi` | map&lt;string,int&gt; |
| `msli` | map&lt;string,list&lt;int&gt;&gt; |

I codici collezione sono prefissi e auto-delimitanti: `lT` indica `list<T>`,
mentre `mKV` indica `map<K,V>`. Le mappe sono immutabili, omogenee per chiavi
e valori e richiedono chiavi scalari uniche. La CLI usa sempre il wrapper
canonico non ambiguo `{\"$ax.map\":[[key,value],...]}`. In questa tranche il supporto
mappe riguarda Compact Source e reference runtime; AX-IR e persistenza SQLite
restano milestone successive.

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

## Compact UI Source 3

La UI applicativa usa un sidecar `.ui.axl` con header `3`. Conserva gli stessi
separatori e la stessa regola multilinea, ma ha frame dedicati al semantic UI tree.

| Opcode | Frame | Semantica |
|---:|---|---|
| `60` | `60|view_id` | apre una vista |
| `61` | `61|node_id|component_id` | apre un componente |
| `62` | `62|property_id|value` | assegna una proprietà |
| `63` | `63|event_id|action` | collega un evento |
| `99` | `99` | chiude componente o vista |

I componenti `64` e `65` definiscono rispettivamente una data table e una
colonna. Il contratto della table usa: risorsa (`1`), label entità (`2`), page
size (`3`), densità (`4`) e modalità mobile (`5`). Il contratto column usa:
campo (`1`), label (`2`), tipo visuale (`3`), priorità responsive da 1 a 3 (`4`)
e larghezza minima da 80 a 600 pixel (`5`).

```axl
3;60|2;
  61|1000|64;
    62|1|"customers";
    62|2|"Clienti";
    62|3|#20;
    62|4|"comfortable";
    62|5|"cards";
    61|1001|65; 62|1|"name"; 62|2|"Nome"; 62|3|"text"; 62|4|#1; 62|5|#180; 99;
    61|1002|65; 62|1|"status"; 62|2|"Stato"; 62|3|"status"; 62|4|#2; 99;
  99;
99;
```

Il compilatore rifiuta componenti sconosciuti, proprietà fuori contratto, tipi
errati e colonne senza campo. Questo mantiene il formato compatto senza perdere
validazione semantica.

## Canonicalizzazione

Il comando `pack` converte la sintassi legacy nella rappresentazione canonica:

```bash
axl pack legacy.axl -o packed.axl
```

`pack` produce il formato multilinea standard con larghezza 100. `axl fmt`
riformatta un file esistente; `--width N` sceglie la larghezza e `--check`
verifica il formato senza modificarlo.

Il compilatore può derivare separatamente una rappresentazione minificata
canonica per hashing, cache, deduplicazione, firma e trasporto. La minificazione
non è il formato raccomandato per i file sorgente versionati in Git.

## Formato legacy

Il parser testuale leggibile (`let`, `fn`, `agent`, indentazione visuale) resta temporaneamente disponibile come frontend di migrazione e debug. Non è la sintassi primaria e non definisce il design futuro di AXL.

## Evoluzione

Nuovi opcode possono essere aggiunti senza rendere verbose il linguaggio. Le famiglie future includeranno collezioni, record, enum, task, eventi, concorrenza, capability standard e binding piattaforma. La versione iniziale impedisce interpretazioni ambigue; cambi incompatibili richiedono un nuovo header sorgente.
