# Agenti, workflow, tool e memoria

## Agent-native non significa probabilistico

Le primitive agentiche fanno parte del modello del linguaggio, ma l'esecuzione del codice resta deterministica. Modelli AI, motori di ricerca e servizi esterni sono capability runtime.

## Agent

Un `agent` è un principal di sicurezza con:

- nome stabile;
- frame locale isolato;
- elenco esplicito di tool concessi;
- corpo eseguibile;
- futura identità, policy, budget e contesto dedicati.

```axl
agent researcher uses search
    let finding = call search("AXL")
    memory finding = finding meta confidence=90 source=researcher
    emit finding
end
```

Un tool registrato dall'host ma non presente in `uses` viene negato.

## Workflow

Un `workflow` compone agenti o altri workflow:

```axl
workflow research_and_publish
    run researcher
    run publisher
end
```

La versione corrente è sequenziale e rifiuta cicli statici. Sono futuri: DAG, parallelismo, retry, checkpoint, sospensione e ripresa.

## Tool e capability

Un tool host dichiara:

- nome;
- handler;
- effetto (`read`, `write`, `destructive` o altro);
- necessità di approvazione.

Esempio Python:

```python
from axl import Tool


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
```

I plugin Python sono infrastruttura fidata: il runtime limita quali tool possono essere chiamati, ma non sandboxa il codice interno del plugin.

## Approval fail-closed

Solo il valore booleano esatto `True` autorizza un effetto. Valori truthy come `"true"` o `1`, eccezioni del provider e assenza di provider vengono negati.

## Audit

Il runtime registra eventi quali:

- `approval_required`;
- `approved`;
- `denied`;
- `executed`;
- `failed`.

Gli audit non devono contenere segreti. I tool devono risolvere credenziali internamente.

## AM — memoria

**AM** è il modulo memoria di AXL, non il nome del linguaggio.

Proprietà correnti:

- interfaccia `MemoryStore` provider-agnostic;
- adapter in-memory e SQLite;
- scope host-controlled;
- valori tipizzati;
- confidence, source, versione e timestamp;
- TTL e scadenza automatica;
- cancellazione esplicita tramite `forget`.

La memoria non viene promossa automaticamente tra scope. Vector store, graph store e backend cloud dovranno implementare lo stesso contratto senza cambiare la semantica centrale.

## Budget

Ogni esecuzione può limitare:

- step di istruzioni/espressioni;
- profondità delle funzioni;
- byte dei valori;
- byte dell'output;
- numero di tool call;
- operazioni memoria.

Tool bloccanti o non fidati devono essere isolati dal processo host con timeout e sandbox esterni.
