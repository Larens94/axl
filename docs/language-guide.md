# Guida al sorgente compatto

AXL Compact Source 2 è il formato canonico. Per la tabella normativa completa vedere [`compact-syntax.md`](compact-syntax.md).

## Struttura

```text
2;frame;frame;frame
```

- `2`: versione;
- `;`: separatore frame;
- `|`: separatore campi;
- `,`: separatore token RPN;
- `99`: fine blocco.

Le newline sono opzionali e non strutturali. La forma canonica prodotta da `axl pack` è una singola riga.

## Atomi

```text
#42       integer
"AXL"     string JSON
?1        true
?0        false
$x        variabile
@finding  memoria
```

## RPN

```text
#2,#3,#4,*,+      2 + 3 * 4
#8,#7,G           8 >= 7
"AX","L",+       "AXL"
#7,#8,^add/2      funzione add(7,8)
"AXL",!search/1  capability search("AXL")
```

Codici confronto: `=` (`==`), `!` (`!=`), `G` (`>=`), `L` (`<=`).

## Binding e output

```axl
2;10|count|#3|i;10|name|"AXL"|s;10|ready|?1|b;12|$name
```

Tipi: `i` integer, `s` string, `b` boolean.

## If e while

```axl
2;10|n|#0;32|$n,#3,<;30|$n,#1,=;12|"one";31;12|$n;99;10|n|$n,#1,+;99
```

- `30` apre if;
- `31` apre else;
- `32` apre while;
- `99` chiude il blocco.

## Funzioni

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99;10|n|#7,#8,^add/2|i;12|$n
```

`40|add|a:i,b:i|i` dichiara firma e apre il corpo; `11` ritorna; `^add/2` consuma due argomenti dallo stack RPN.

## Moduli

`m.axl`:

```axl
2;40|add|a:i,b:i|i;11|$a,$b,+;99
```

`app.axl`:

```axl
2;1|m|m.axl;12|#20,#22,^m.add/2
```

Gli import sono relativi, espliciti e namespaced. Sono ammessi solo file `.axl` entro il module root; path assoluti e componenti `..` vengono rifiutati. Cicli, profondità oltre 256, più di 1024 moduli, oltre 4 MiB aggregati ed effetti top-level nei moduli vengono rifiutati.

## Memoria AM

```axl
2;20|finding|"AXL"|95|3600|researcher;10|x|@finding;12|$x;21|finding
```

Lo scope è deciso dall'host. Metadata: confidence, TTL (`-` se assente), source.

## Agenti, tool e workflow

```axl
2;50|r|search;10|x|"AXL",!search/1;12|$x;99;51|w;52|r;99;52|w
```

- `50`: agente e grants;
- `!search/1`: tool call postfix;
- `51`: workflow;
- `52`: run;
- `99`: fine dichiarazione.

## Conversione legacy

```bash
axl pack readable.axl -o compact.axl
```

Il frontend keyword-based resta solo per migrazione e debug. Nuove feature devono avere una codifica compact canonica prima di essere considerate parte del linguaggio.
