# CRM dimostrativo e copertura UI

> Stato verificato: 26 agosto 2026, release `0.1.0-alpha.1`.

Il CRM è la vertical slice canonica di AXL: lo stesso modello genera persistenza,
API e interfaccia. Serve a dimostrare il compilatore applicativo, non a dichiarare
che il linguaggio sia già pronto per carichi di produzione.

## Copertura funzionale

| Area | Implementazione |
|---|---|
| Dominio | customer, lead, deal, activity, task, note |
| Database | 6 tabelle SQLite e migrazione generata |
| API | 5 operazioni CRUD per risorsa, 30 in totale |
| Navigazione | dashboard, 6 risorse, report, impostazioni |
| CRUD UI | lista, creazione, modifica e dettaglio per risorsa |
| Tabelle | ricerca, filtri, sorting, paginazione e densità |
| Responsive | drawer desktop, menu bottom mobile, table-to-card |
| Feedback | loading, empty state, errori, conferme e notifiche |
| Visual | Material UI, icone Lucide, token e tema AXL |

## UI kit coperto

La demo copre le fondamenta tipiche di un admin panel/CRM:

- app shell, sidebar, breadcrumb, top bar e bottom navigation;
- KPI card, progress, stato e dashboard riepilogativa;
- data table desktop con colonne tipizzate e azioni contestuali;
- card list mobile con priorità delle informazioni;
- search, filter, sort, pagination e page-size server-side;
- form create/edit, detail view, dialog e notification;
- avatar, chip/status, menu, button, icon button e tooltip;
- empty, loading ed error state;
- tema responsive e breakpoints.

Il 70% è un obiettivo di copertura qualitativa del kit amministrativo, non una
percentuale calcolata contro un catalogo universale. Componenti specialistici
come calendario avanzato, editor rich-text, upload, kanban drag-and-drop e chart
interattivi restano estensioni successive.

## Compact UI Source 3

Le decisioni UI importanti sono nel sidecar numerico, non hardcoded nel template:

```axl
3;60|2;
  61|1000|64;
    62|1|"customers";
    62|3|#25;
    62|4|"comfortable";
    62|5|"cards";
    61|1001|65; 62|1|"name"; 62|2|"Nome"; 62|3|"text"; 62|4|#1; 99;
    61|1002|65; 62|1|"email"; 62|2|"Email"; 62|3|"text"; 62|4|#2; 99;
  99;
99;
```

Il compilatore valida componenti, proprietà, tipi, priorità e limiti. React e
TanStack interpretano il modello risultante; non definiscono il linguaggio.

## Gate della demo

La demo è accettata quando:

1. `axl check` valida entrambi i sorgenti;
2. `axl build` rigenera backend, frontend e SQL;
3. i test Rust passano;
4. backend e frontend generati compilano;
5. lo smoke test esercita health, list, create, update e delete;
6. desktop e mobile mantengono navigazione e azioni CRUD utilizzabili.

## Limiti dichiarati

- autenticazione e autorizzazione sono dimostrative;
- SQLite è il target locale, non il database production consigliato;
- non esistono ancora garanzie di migrazione stabile tra versioni alpha;
- accessibilità e browser/device matrix richiedono una suite E2E dedicata;
- osservabilità, rate limit, secret management e hardening sono roadmap.
