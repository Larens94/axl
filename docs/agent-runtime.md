# Agenti, workflow, capability e AM

## Deterministico, non probabilistico

Le primitive agentiche appartengono al linguaggio, ma il loro parsing e controllo sono deterministici. Modelli AI, ricerca, filesystem, rete e servizi sono capability runtime tipizzate.

## Agente

Un agente è un principal con nome, frame isolato, grants espliciti e corpo eseguibile.

```axl
2;50|researcher|search;10|finding|"AXL",!search/1;20|finding|$finding|90|-|researcher;12|$finding;99
```

- `50|researcher|search` apre l'agente e concede solo `search`;
- `!search/1` consuma un argomento e invoca la capability;
- `99` chiude il corpo.

Un tool registrato dall'host ma non concesso viene negato.

## Workflow

```axl
2;51|release;52|researcher;52|publisher;99;52|release
```

`51` apre il workflow, `52` esegue un runnable, `99` chiude. Il runtime corrente è sequenziale e rifiuta cicli. DAG, parallelismo, retry e checkpoint sono roadmap.

## Capability host

Una capability dichiara nome, ABI, input/output, effetto, target e policy. Oggi il bridge Python usa `Tool`:

```python
from axl import Tool


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
```

I futuri bridge Rust/C/WASI/DOM/GPU implementeranno lo stesso modello. Le capability non diventano keyword vendor-specific nel sorgente.

## Approval fail-closed

Solo il booleano esatto `True` autorizza. Stringhe truthy, numeri, eccezioni o provider assente vengono negati.

## Audit

Eventi: `approval_required`, `approved`, `denied`, `executed`, `failed`. Segreti e credenziali devono essere risolti dentro il bridge e non apparire in source, IR, output o audit.

## AM — memoria

AM è il modulo memoria, non il nome del linguaggio.

```axl
2;20|finding|"result"|95|3600|researcher;12|@finding;21|finding
```

Proprietà:

- protocollo provider-agnostic;
- adapter in-memory e SQLite;
- scope controllato dall'host;
- valori tipizzati;
- confidence, source, versione, timestamp e TTL;
- cancellazione esplicita.

Nessuna promozione automatica tra scope. Backend vector, graph o cloud devono preservare il contratto AM.

## Budget

Limiti su step/espressioni, profondità chiamate, valori, output, tool call e memoria. Capability bloccanti o non fidate richiedono isolamento, timeout e cancellazione host.
