# Sviluppo e contribuzione

## Filosofia

AXL evolve tramite vertical slice complete:

```text
sorgente → parser → IR → validazione/type-check → runtime → output osservabile
```

Una modifica alla grammatica non è completa senza IR, semantica, diagnostica, test, esempio e documentazione coerenti.

## Flusso di lavoro TDD

Per ogni comportamento:

1. scrivere un test focalizzato;
2. eseguirlo e verificare il fallimento atteso;
3. implementare il minimo necessario;
4. rieseguire test focalizzato e suite completa;
5. rifattorizzare mantenendo i test verdi.

## Gate locali

```bash
python3 -m unittest discover -s tests -q
python3 -m ruff check .
python3 -m ruff format --check .
python3 -m compileall -q axl tests examples
python3 -m json.tool schema/axl-ir-1.0.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.1.schema.json >/dev/null
python3 -m json.tool schema/axl-ir-1.2.schema.json >/dev/null
git diff --check
```

Verificare anche almeno un programma sorgente e lo stesso programma via `compile`→`exec`.

## Modifiche al linguaggio

Una proposta deve specificare:

- problema e casi d'uso;
- sintassi proposta;
- regole di tipo;
- nodi HIR/IR;
- semantica runtime ed effetti;
- diagnostica;
- impatto su sicurezza e capability;
- compatibilità e migrazione;
- test di conformità.

## Compatibilità AX-IR

Non modificare uno schema pubblicato. Una modifica incompatibile richiede:

1. incremento versione AX-IR;
2. nuovo file schema;
3. decoder della nuova versione;
4. upgrade legacy o errore esplicito;
5. test di round-trip e compatibilità;
6. aggiornamento di specifica e changelog.

## Separazione dei livelli

- il parser non esegue effetti;
- l'IR non contiene client/provider specifici;
- il runtime non interpreta sintassi sorgente;
- i plugin implementano capability host;
- segreti e credenziali non entrano in sorgente, IR o audit.

## Segnalazioni di sicurezza

Non pubblicare vulnerabilità sfruttabili o credenziali in issue pubbliche. Contattare privatamente il maintainer del repository fino alla definizione di un processo formale di security reporting.
