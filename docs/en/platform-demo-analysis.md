# CRM demonstration and UI coverage

> Verified state: August 26, 2026, release `0.1.0-alpha.1`.

The CRM is AXL's canonical vertical slice: one model generates persistence, APIs,
and UI. It demonstrates the application compiler; it does not claim that the
language is ready for production workloads.

## Functional coverage

| Area | Implementation |
|---|---|
| Domain | customer, lead, deal, activity, task, note |
| Database | 6 SQLite tables and a generated migration |
| API | 5 CRUD operations per resource, 30 total |
| Navigation | dashboard, 6 resources, reports, settings |
| CRUD UI | list, create, edit, and detail per resource |
| Tables | search, filters, sorting, pagination, density |
| Responsive | desktop drawer, mobile bottom nav, table-to-card |
| Feedback | loading, empty, error, confirmation, notification states |
| Visual | Material UI, Lucide icons, AXL tokens and theme |

## UI kit coverage

The demo covers the common foundations of an admin panel or CRM:

- app shell, sidebar, breadcrumb, top bar, and bottom navigation;
- KPI cards, progress, status, and summary dashboard;
- desktop data tables with typed columns and contextual actions;
- mobile card lists with information priority;
- server-side search, filter, sort, pagination, and page size;
- create/edit forms, detail view, dialogs, and notifications;
- avatars, chips/status, menus, buttons, icon buttons, and tooltips;
- empty, loading, and error states;
- responsive theme and breakpoints.

The 70% figure is a qualitative coverage target for administrative UI basics,
not a computed percentage against a universal catalog. Specialist components
such as advanced calendars, rich-text editors, uploads, drag-and-drop kanban, and
interactive charts remain future extensions.

## Compact UI Source 3

Important UI decisions live in the numeric sidecar instead of the template:

```axl
3;60|2;
  61|1000|64;
    62|1|"customers";
    62|3|#25;
    62|4|"comfortable";
    62|5|"cards";
    61|1001|65; 62|1|"name"; 62|2|"Name"; 62|3|"text"; 62|4|#1; 99;
    61|1002|65; 62|1|"email"; 62|2|"Email"; 62|3|"text"; 62|4|#2; 99;
  99;
99;
```

The compiler validates components, properties, types, priorities, and limits.
React and TanStack consume the resulting model; they do not define the language.

## Demo gate

The demo is accepted when:

1. `axl check` validates both source files;
2. `axl build` regenerates backend, frontend, and SQL;
3. Rust tests pass;
4. generated backend and frontend compile;
5. the smoke test exercises health, list, create, update, and delete;
6. desktop and mobile retain usable navigation and CRUD actions.

## Declared limitations

- authentication and authorization are demonstrative;
- SQLite is the local target, not the recommended production database;
- alpha versions do not yet guarantee stable migrations;
- accessibility and browser/device matrices require a dedicated E2E suite;
- observability, rate limiting, secret management, and hardening are roadmap work.
