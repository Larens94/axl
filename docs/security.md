# Sicurezza e capability

## Threat model

Sono considerati non fidati:

- sorgente AXL;
- documenti AX-IR JSON;
- output di modelli e tool;
- contenuti recuperati dalla memoria;
- input provenienti da rete o utenti.

Sono considerati infrastruttura fidata nella reference implementation:

- processo host;
- plugin Python caricati esplicitamente;
- callback di approvazione;
- adapter memoria configurati dall'host.

## Regole fondamentali

1. **Deny-by-default:** un tool non registrato non può essere eseguito.
2. **Least privilege:** un agente può usare solo i tool elencati in `uses`.
3. **Approval pre-effect:** l'approvazione avviene prima dell'handler.
4. **Fail-closed:** soltanto `approved is True` autorizza.
5. **Validazione prima degli effetti:** AX-IR viene validata prima dell'esecuzione.
6. **Scope host-owned:** il sorgente non sceglie arbitrariamente lo scope memoria.
7. **Budget:** loop, output, valori, tool e memoria sono limitati.
8. **Segreti fuori dal linguaggio:** credenziali mai in sorgente, IR, output o audit.
9. **Module root:** gli import accettano solo path relativi `.axl`, top-level e confinati alla directory autorizzata; path assoluti e `..` sono rifiutati.
10. **Budget import:** profondità, numero di moduli e byte sorgente aggregati hanno limiti fail-closed.

## Confini non coperti

La reference implementation non fornisce ancora:

- sandbox del codice Python dei plugin;
- preemption di handler bloccanti;
- isolamento OS/container;
- limiti CPU/RAM imposti dal kernel;
- firma dei package;
- attestazione delle build;
- rete filtrata per capability a livello OS.

Questi controlli devono essere aggiunti dal deployment host e, in seguito, dal runtime Rust.

## Modello target

Il runtime Rust dovrà associare ogni effetto a una capability non falsificabile:

```text
principal agente
  + capability
  + scope risorsa
  + policy/versione
  + budget/scadenza
  + eventuale approval
  = effetto autorizzato
```

File, rete, database, shell, GPU, modelli e memoria saranno capability distinte. Le capability non dovranno essere rappresentate come stringhe liberamente costruibili dal programma.

## Errori e audit

Gli errori devono rimanere espliciti e machine-readable. L'audit registra decisioni, non segreti. In produzione ogni approvazione dovrebbe essere legata a run, principal, hash degli argomenti, policy, timestamp e scadenza.
