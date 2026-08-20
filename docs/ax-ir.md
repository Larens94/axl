# AX-IR e compatibilità

## Definizione

**AX-IR** è la rappresentazione intermedia tipizzata e versionata di AXL. Disaccoppia sintassi, analisi statica e runtime.

## Envelope JSON

AX-IR 1.1 viene serializzata così:

```json
{
  "ir_version": "1.1",
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

Il decoder 1.1 legge documenti 1.0 e applica un upgrade controllato. Lo schema 1.0 resta immutato; lo schema 1.1 vive in un file separato.

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
