# Guida al linguaggio

Questa guida descrive AXL `1.1-dev`, non tutte le funzionalità target.

## Valori e variabili

I valori correnti sono `int`, `string` e `bool`.

```axl
let count: int = 3
let name: string = "AXL"
let ready: bool = true
emit name
```

Una variabile senza annotazione riceve il tipo inferito dall'espressione quando possibile.

## Operatori

```axl
let total = 2 + 3 * 4
let label = "Agent " + "Language"
let enough = total >= 10
```

- `+` somma interi o concatena stringhe;
- `-`, `*`, `/` operano su interi;
- `/` accetta solo risultati interi;
- `==`, `!=` richiedono tipi uguali;
- `<`, `<=`, `>`, `>=` operano su interi.

## Controllo di flusso

```axl
if total >= 10
    emit "large"
else
    emit "small"
end

let index = 0
while index < 3
    emit index
    let index = index + 1
end
```

Le condizioni devono essere booleane. I cicli sono protetti dal budget di esecuzione.

## Funzioni tipizzate

```axl
fn add(a: int, b: int) -> int
    return a + b
end

fn describe(value: int) -> string
    if value >= 10
        return "large"
    else
        return "small"
    end
end

let total: int = add(7, 8)
emit describe(total)
```

Il type-checker verifica:

- nomi e parametri duplicati;
- arità delle chiamate;
- tipi degli argomenti;
- tipo di ritorno;
- presenza di un ritorno su tutti i percorsi richiesti.

Ogni chiamata usa uno scope locale isolato ed è soggetta a un limite di profondità.

## Moduli

`math.axl`:

```axl
fn add(a: int, b: int) -> int
    return a + b
end
```

`app.axl`:

```axl
import math from "math.axl"
emit math.add(20, 22)
```

Gli import:

- sono relativi al file importatore;
- richiedono alias espliciti;
- creano namespace qualificati;
- rifiutano alias duplicati e cicli;
- nella versione corrente esportano soltanto funzioni, senza effetti top-level.

## Output

```axl
emit "hello"
emit 42
```

`emit` aggiunge un valore all'output del programma. Il runtime applica un budget in byte.

## Memoria

```axl
memory finding = "AXL" meta confidence=95 ttl=3600 source=researcher
let saved = recall finding
emit saved
forget finding
```

Lo scope memoria viene deciso dall'host, non dal sorgente.

## Tool

```axl
let result = call search("AXL")
emit result
```

`call` invoca una capability registrata dall'host. Non è una normale funzione AXL e resta soggetta a grants, policy, approvazioni e audit.

## Agenti e workflow

```axl
agent researcher uses search
    emit call search("AXL")
end

workflow release
    run researcher
end

run release
```

Gli agenti hanno scope locale e grants espliciti. I workflow correnti sono sequenziali.

## Commenti e identificatori

```axl
# commento su riga intera
let valid_name = 1
```

Gli identificatori seguono `[A-Za-z_][A-Za-z0-9_]*`; le keyword sono riservate.

## Limiti attuali

Non sono ancora disponibili liste, mappe, record, enum, generics, pattern matching, eccezioni strutturate, async o metodi. La roadmap li introduce prima dei framework applicativi.
