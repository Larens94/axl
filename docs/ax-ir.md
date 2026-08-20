# AX-IR e compatibilità

## Definizione

**AX-IR** è la rappresentazione intermedia tipizzata e versionata di AXL. Disaccoppia sintassi, analisi statica e runtime.

## Envelope JSON

AX-IR 1.2 viene serializzata così:

```json
{
  "ir_version": "1.2",
  "program": {
    "type": "Program",
    "instructions": []
  }
}
```

I nodi usano un discriminatore `type` e campi chiusi. Il decoder rifiuta:

- versioni sconosciute;
- nodi o campi sconosciuti;
- chiavi JSON duplicate;
- tipi literal non AXL;
- espressioni nella posizione di istruzioni;
- collezioni malformate;
- identificatori e operatori invalidi;
- riferimenti o grafi workflow non validi;
- payload oltre il limite configurato.

## Versioni pubblicate

- **1.0:** agenti, workflow, memoria, tool e controllo di flusso base.
- **1.1:** funzioni tipizzate, `return`, chiamate funzione e annotazioni sui binding.
- **1.2:** `ListExpression` e tipi `list<T>` omogenei.

Il decoder 1.2 legge documenti 1.0 e 1.1 e applica upgrade controllati dove necessari. Ogni schema pubblicato resta immutato e vive in un file separato.

## Moduli

Gli import sono direttive del compilatore e non sopravvivono nell'IR. Il compilatore risolve i file, applica namespace qualificati e produce un singolo `Program` validato.

## Evoluzione futura

La IR JSON corrente è ad alto livello. Lo stack definitivo distinguerà:

### AX-HIR

- funzioni e tipi risolti;
- agenti, workflow, memoria e capability;
- semantica vicina al linguaggio;
- source mapping per diagnostica.

### AX-MIR

- blocchi di base e control-flow graph;
- chiamate e valori abbassati;
- layout, ownership e ABI;
- operazioni adatte a VM, native e WASM;
- ottimizzazioni verificabili.

Ogni lowering dovrà preservare output, effetti, errori, audit e limiti osservabili rispetto al reference runtime.

## Schemi

- [`../schema/axl-ir-1.0.schema.json`](../schema/axl-ir-1.0.schema.json)
- [`../schema/axl-ir-1.1.schema.json`](../schema/axl-ir-1.1.schema.json)
- [`../schema/axl-ir-1.2.schema.json`](../schema/axl-ir-1.2.schema.json)
