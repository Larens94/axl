# ERP layout reference (Odoo-style, original implementation)

Design memory for building an ERP-style web layout in AXL. This describes the
**functional anatomy** (structure + behavior) of a modern modular-ERP web client
as general UX guidance. It is written from scratch as a specification for our own
**original** layout: we do not copy Odoo's trademark, logo, visual identity, CSS,
assets, or source code (Odoo Community is LGPLv3, but its brand/trade-dress are
protected and its code is not vendored here).

Use this file to drive autoloop iterations that close the gap between our current
portal shell and a full ERP layout. All product logic stays in AXL; the shell and
new widgets are added as open UI primitives (syntax → IR → manifest → renderer).

---

## 1. Global shell (chrome around every screen)

Top navbar (fixed), left → right:
- **Apps launcher button** — opens the grid of installed apps (our `/apps`).
- **Current app name** + the app's **menu bar** (the app's top-level menus /
  sections). Switching app swaps the menu bar.
- **Systray** (right side): global search entry, activities/notifications,
  messaging, and a **user menu** (profile, preferences, log out).

Main area below the navbar:
- Optional **left panel** (search categories / sub-menu), then the **control
  panel**, then the **view** (the record data).

Responsive: on small screens the menus collapse into a hamburger; the control
panel wraps; views become single-column.

Theme: light surfaces, one brand **accent color**, rounded controls, subtle
shadows, generous whitespace. (Our current theme = "dashboard-apple" CSS vars;
keep it original.)

## 2. Control panel (per action, above the view)

Two rows, conceptually:
- **Row 1 (left):** breadcrumb `App › Model › Record`, plus the record/action
  title. **(right):** the **view switcher** (icons/tabs: List, Kanban, Form,
  Calendar, Pivot, Graph, Activity…), and **pagination** (`1–80 / 200`, prev/next).
- **Row 2 (left):** primary action button **"New"**, plus an **Actions ⚙**
  menu (export, duplicate, delete, server actions). **(center/right):** the
  **search bar** with removable filter *facets*, a **Filters** dropdown, a
  **Group By** dropdown, and **Favorites** (saved searches).

We have: breadcrumb, `view` switcher tabs, primary "Nuovo" button, per-field
filter bar, pagination. **Missing:** the Actions menu, Group By, Favorites/saved
searches, faceted multi-filter search, and the search box.

## 3. Views (the data area)

### List view
- Column headers; rows; optional inline edit.
- Multi-select checkboxes → bulk actions.
- **Group by** folds rows into collapsible groups with per-group counts and
  **column aggregates** (sum/avg) in group headers and the footer.
- Optional column chooser.

We have: a static table with detail links. **Missing:** grouping/aggregates,
selection + bulk actions, inline edit, column chooser.

### Kanban view
- Columns = a group field (usually a stage/status). Cards show a few fields.
- Per-column header shows the count (and often an aggregate).
- **Quick create** at the top of a column; drag cards between columns; fold/hide
  columns; optional per-column progress bar.
- Card context menu (color, edit, delete).

We have: columns by enum stage, cards with fields, counts, detail links.
**Missing:** drag-and-drop, per-column quick create, card menu, folding, progress.

### Form view
Vertical anatomy, top → bottom:
- **Statusbar** (top strip): workflow **status buttons** (e.g. Confirm, Cancel)
  on the left; a clickable **stage pipeline** (the status flow) on the right.
- **Button box / smart buttons** (top-right of the sheet): boxes linking to
  related records with counts (e.g. "3 Meetings", "€ 12k Invoiced").
- **Sheet:** a large **title** field, then fields arranged in **two columns of
  groups** (label + widget). Relational fields render as pickers (Many2one =
  dropdown/autocomplete; One2many/Many2many = embedded editable table / tags).
- **Notebook**: tabbed sections below the groups (e.g. "Order Lines", "Other
  Info") — often holding related line tables.
- **Chatter** (right side on desktop, below on mobile): **Send message**, **Log
  note**, **Schedule activity**, **followers**, and the chronological message /
  activity history.

We have: a flat field list + enum/relation selects + a create button; drawer
detail = a `<dl>` of fields + workflow actions. **Missing:** statusbar with stage
pipeline, smart-button box, two-column grouped sheet, notebook tabs, chatter/
activity timeline, embedded one2many line editor.

### Other views (mention / later)
Calendar, Pivot, Graph, Activity, Gantt — each selectable from the view switcher.

## 4. Search / filter model
- Free-text search that proposes field-scoped facets.
- **Filters** (predefined + custom conditions) and **Group By** combine into the
  active facet set shown in the search bar.
- **Favorites**: save the current filters+groupby (and set a default).
- Optional **search panel** on the left (category tree / filter chips).

## 5. Priority gap list for our AXL portal (roadmap)
Ordered by impact for "feels like an ERP":
1. **Form sheet layout**: two-column grouped fields + a bold title (primitive:
   field groups / `group`), replacing the flat list.
2. **Statusbar** on record forms: clickable stage pipeline + status action
   buttons (primitive: `statusbar <enum-field>` + status actions).
3. **Notebook tabs** on forms for related line tables (primitive: `tab "Label"`).
4. **Chatter / activity** timeline on record detail (primitive: `chatter` / a
   messages+activities widget backed by an AXL store).
5. **Search + Group By + Favorites** in the control panel (primitives:
   `search`, `groupby`, saved-view metadata).
6. **List grouping + aggregates** and **multi-select bulk actions**.
7. **Kanban quick-create + drag/reorder** (drag needs a client hook; keep the
   state transition in AXL via the existing `avanza`-style actions).
8. **Systray**: global search + user menu; **app menu bar** per app.

Each item should ship as an open AXL UI primitive with: syntax → AST → Graph IR
(new `ui_*` node kind + packed opcode) → `axl-ui/1` manifest field → HTML
renderer + CSS, positive tests, and a negative diagnostic — same pattern already
used for `kanban`, `gallery`, `view`, and `select`.

## 6. What we already have (baseline)
- Modular platform: installable module registry, app launcher `/apps`, catalog
  `/apps/store`, install/uninstall actions.
- Views: `page` (list table), `kanban` board, `gallery` grid, `drawer`/`modal`
  detail, `kpi`/`chart` dashboards.
- Control panel: `view` switcher tabs + primary "Nuovo" button.
- Forms: enum selects, **relation dropdowns** (`select … from Flow`), validation
  re-render, `omit`/`title`/`submit_label`.
- Workflow: enum stage transitions via `match` + `action` (e.g. `avanza`).
