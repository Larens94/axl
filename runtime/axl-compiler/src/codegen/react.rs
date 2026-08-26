use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, compact_views: Option<&[crate::compact_ui::UiView]>, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("src/pages"))?;
    fs::create_dir_all(output.join("src/components"))?;
    
    generate_package_json(app, output)?;
    generate_vite_config(output)?;
    generate_tsconfig(output)?;
    generate_index_html(app, output)?;
    generate_main_tsx(output)?;
    generate_icon_registry(output)?;
    generate_app_shell(app, output)?;
    generate_data_table(output)?;
    generate_app_tsx(app, output)?;

    for entity in &app.entities {
        let table_view = compact_views.and_then(|views| views.iter().find(|view| {
            view.root.component_id == 64
                && compact_prop_string(&view.root, 1).as_deref() == Some(entity.table_name.as_str())
        }));
        generate_list_page(entity, table_view, output)?;
        generate_create_page(entity, output)?;
        generate_edit_page(entity, output)?;
        generate_show_page(entity, output)?;
    }
    
    if let Some(views) = compact_views {
        generate_compact_dashboard(app, views, output)?;
    } else {
        generate_dashboard(app, output)?;
    }
    generate_admin_pages(output)?;
    
    Ok(())
}

fn generate_package_json(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let name = app.name.to_lowercase().replace(" ", "-");
    let content = format!(
        r#"{{
  "name": "{name}",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "@refinedev/core": "^4.0.0",
    "@refinedev/mui": "^5.0.0",
    "@refinedev/react-router-v6": "^4.0.0",
    "@refinedev/simple-rest": "^5.0.0",
    "@refinedev/react-hook-form": "^4.0.0",
    "@mui/material": "^5.0.0",
    "@mui/icons-material": "^5.0.0",
    "@mui/x-data-grid": "^6.0.0",
    "@tanstack/react-table": "^8.21.3",
    "@emotion/react": "^11.0.0",
    "@emotion/styled": "^11.0.0",
    "lucide-react": "^0.468.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "react-router-dom": "^6.0.0",
    "react-hook-form": "^7.0.0",
    "@tanstack/react-query": "^5.0.0"
    ,"axios": "^1.7.0"
  }},
  "devDependencies": {{
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "@vitejs/plugin-react": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^6.4.3"
  }}
}}"#
    );
    
    fs::write(output.join("package.json"), content)?;
    Ok(())
}

fn generate_vite_config(output: &Path) -> Result<()> {
    let content = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        manualChunks: {
          framework: ['react', 'react-dom', 'react-router-dom', '@refinedev/core', '@refinedev/react-router-v6', '@refinedev/simple-rest', '@refinedev/mui', '@refinedev/react-hook-form', '@mui/material', '@mui/icons-material', '@mui/x-data-grid', '@tanstack/react-table', 'lucide-react', '@emotion/react', '@emotion/styled']
        }
      }
    }
  },
  server: {
    proxy: {
      '/api': 'http://localhost:3000'
    }
  }
})
"#;
    
    fs::write(output.join("vite.config.ts"), content)?;
    Ok(())
}

fn generate_tsconfig(output: &Path) -> Result<()> {
    let content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#;
    
    fs::write(output.join("tsconfig.json"), content)?;
    
    // Generate tsconfig.node.json
    let node_content = r#"{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
"#;
    fs::write(output.join("tsconfig.node.json"), node_content)?;
    
    Ok(())
}

fn generate_index_html(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let name = &app.name;
    let content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 64 64%22><rect width=%2264%22 height=%2264%22 rx=%2214%22 fill=%22%235b4bdb%22/><text x=%2232%22 y=%2243%22 text-anchor=%22middle%22 font-size=%2234%22 fill=%22white%22>A</text></svg>" />
    <title>{name}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#
    );
    
    fs::write(output.join("index.html"), content)?;
    Ok(())
}

fn generate_main_tsx(output: &Path) -> Result<()> {
    let content = r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#;
    
    fs::write(output.join("src/main.tsx"), content)?;
    Ok(())
}

fn generate_icon_registry(output: &Path) -> Result<()> {
    let content = r#"import {
  Activity, Bell, Building2, CalendarDays, ChartNoAxesCombined,
  CircleDollarSign, ClipboardCheck, Columns3, ContactRound, Eye, FileText,
  Gauge, Handshake, LayoutDashboard, LogOut, MoreHorizontal, NotebookText,
  Pencil, Plus, Search, Settings, SlidersHorizontal, Target, Trash2, UserRound, UsersRound, X,
  type LucideIcon,
} from "lucide-react";

export type AxlIconName =
  | "activity" | "bell" | "calendar" | "customer" | "dashboard"
  | "close" | "columns" | "create" | "deal" | "delete" | "edit"
  | "lead" | "logout" | "more" | "note" | "report" | "search"
  | "settings" | "task" | "user" | "view";

const icons: Record<AxlIconName, LucideIcon> = {
  activity: Activity,
  bell: Bell,
  calendar: CalendarDays,
  close: X,
  columns: Columns3,
  create: Plus,
  customer: UsersRound,
  dashboard: LayoutDashboard,
  deal: Handshake,
  delete: Trash2,
  edit: Pencil,
  lead: Target,
  logout: LogOut,
  more: MoreHorizontal,
  note: NotebookText,
  report: ChartNoAxesCombined,
  search: Search,
  settings: Settings,
  task: ClipboardCheck,
  user: UserRound,
  view: Eye,
};

const resourceIcons: Record<string, AxlIconName> = {
  dashboard: "dashboard",
  customers: "customer",
  contacts: "customer",
  leads: "lead",
  deals: "deal",
  opportunities: "deal",
  activities: "activity",
  tasks: "task",
  notes: "note",
  reports: "report",
  settings: "settings",
};

export const iconForResource = (resource: string): AxlIconName =>
  resourceIcons[resource] ?? "customer";

export const AxlIcon = ({ name, size = 20, label }: { name: AxlIconName; size?: number; label?: string }) => {
  const Icon = icons[name];
  return label
    ? <Icon size={size} role="img" aria-label={label} />
    : <Icon size={size} aria-hidden="true" />;
};

export const iconLibrary = {
  Building2, CircleDollarSign, ContactRound, FileText, Gauge, SlidersHorizontal,
};
"#;
    fs::write(output.join("src/components/iconRegistry.tsx"), content)?;
    Ok(())
}

fn generate_app_shell(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let mut navigation = String::from("  { key: \"dashboard\", label: \"Dashboard\", path: \"/\" },\n");
    for entity in &app.entities {
        navigation.push_str(&format!(
            "  {{ key: \"{}\", label: \"{}\", path: \"/{}\" }},\n",
            escape_tsx(&entity.table_name), escape_tsx(&entity.name), escape_tsx(&entity.table_name)
        ));
    }
    navigation.push_str("  { key: \"reports\", label: \"Reports\", path: \"/reports\" },\n");
    navigation.push_str("  { key: \"settings\", label: \"Settings\", path: \"/settings\" },\n");
    let title = escape_tsx(&app.name);
    let content = format!(r##"import {{ ReactNode, useMemo, useState }} from "react";
import {{
  AppBar, Avatar, BottomNavigation, BottomNavigationAction, Box, Divider,
  Drawer, IconButton, InputAdornment, List, ListItemButton, ListItemIcon,
  ListItemText, Paper, Stack, TextField, Toolbar, Tooltip, Typography,
  useMediaQuery, useTheme,
}} from "@mui/material";
import {{ useLocation, useNavigate }} from "react-router-dom";
import {{ AxlIcon, iconForResource }} from "./iconRegistry";

type NavItem = {{ key: string; label: string; path: string }};

const navigation: NavItem[] = [
{navigation}];

const drawerWidth = 264;

export const AxlAppShell = ({{ children }}: {{ children: ReactNode }}) => {{
  const theme = useTheme();
  const mobile = useMediaQuery(theme.breakpoints.down("md"));
  const location = useLocation();
  const navigate = useNavigate();
  const [moreOpen, setMoreOpen] = useState(false);
  const [quickFind, setQuickFind] = useState("");
  const user = useMemo(() => {{
    try {{ return JSON.parse(localStorage.getItem("axl_user") || "null") as {{ name?: string; email?: string }} | null; }}
    catch {{ return null; }}
  }}, []);

  const current = [...navigation]
    .sort((a, b) => b.path.length - a.path.length)
    .find((item) => item.path === "/" ? location.pathname === "/" : location.pathname.startsWith(item.path));
  const preferredKeys = ["dashboard", "customers", "deals", "tasks"];
  const mobileNavigation = preferredKeys
    .map((key) => navigation.find((item) => item.key === key))
    .filter((item): item is NavItem => Boolean(item));
  const mobileValue = mobileNavigation.findIndex((item) => item.key === current?.key);

  const go = (path: string) => {{ setMoreOpen(false); navigate(path); }};
  const submitQuickFind = () => {{
    const query = quickFind.trim().toLowerCase();
    if (!query) return;
    const match = navigation.find((item) => item.label.toLowerCase().includes(query) || item.key.includes(query));
    if (match) {{ go(match.path); setQuickFind(""); }}
  }};
  const logout = () => {{
    localStorage.removeItem("axl_token");
    localStorage.removeItem("axl_user");
    navigate("/login");
  }};

  const navigationList = (
    <List sx={{{{ px: 1.5, py: 1 }}}} aria-label="CRM navigation">
      {{navigation.map((item) => {{
        const selected = current?.key === item.key;
        return <ListItemButton
          key={{item.key}}
          selected={{selected}}
          onClick={{() => go(item.path)}}
          sx={{{{ mb: .5, minHeight: 44, borderRadius: 2.5,
            "&.Mui-selected": {{ bgcolor: "primary.main", color: "primary.contrastText",
              "& .MuiListItemIcon-root": {{ color: "inherit" }},
              "&:hover": {{ bgcolor: "primary.dark" }},
            }},
          }}}}
        >
          <ListItemIcon sx={{{{ minWidth: 40, color: selected ? "inherit" : "text.secondary" }}}}>
            <AxlIcon name={{iconForResource(item.key)}} />
          </ListItemIcon>
          <ListItemText primary={{item.label}} primaryTypographyProps={{{{ fontWeight: selected ? 700 : 550 }}}} />
        </ListItemButton>;
      }})}}
    </List>
  );

  return <Box sx={{{{ minHeight: "100dvh", bgcolor: "background.default" }}}}>
    <AppBar
      position="fixed"
      color="inherit"
      elevation={{0}}
      sx={{{{ borderBottom: "1px solid", borderColor: "divider", zIndex: theme.zIndex.drawer + 1,
        width: {{ md: `calc(100% - ${{drawerWidth}}px)` }}, ml: {{ md: `${{drawerWidth}}px` }},
      }}}}
    >
      <Toolbar sx={{{{ minHeight: {{ xs: 64, md: 72 }}, gap: 2 }}}}>
        <Stack direction="row" alignItems="center" spacing={{1.25}} sx={{{{ display: {{ md: "none" }} }}}}>
          <Box sx={{{{ display: "grid", placeItems: "center", width: 34, height: 34, borderRadius: 2, bgcolor: "primary.main", color: "white", fontWeight: 900 }}}}>A</Box>
          <Typography fontWeight={{850}}>{title}</Typography>
        </Stack>
        <Box sx={{{{ flex: 1 }}}} />
        <TextField
          size="small"
          value={{quickFind}}
          onChange={{(event) => setQuickFind(event.target.value)}}
          onKeyDown={{(event) => {{ if (event.key === "Enter") submitQuickFind(); }}}}
          placeholder="Jump to a section…"
          inputProps={{{{ "aria-label": "Jump to a CRM section" }}}}
          InputProps={{{{ startAdornment: <InputAdornment position="start"><AxlIcon name="search" size={{18}} /></InputAdornment> }}}}
          sx={{{{ display: {{ xs: "none", sm: "block" }}, width: {{ sm: 260, lg: 360 }} }}}}
        />
        <Tooltip title="Notifications"><IconButton aria-label="Notifications"><AxlIcon name="bell" /></IconButton></Tooltip>
        <Tooltip title={{user?.name || user?.email || "Account"}}>
          <Avatar sx={{{{ width: 36, height: 36, bgcolor: "secondary.main", fontSize: 14, fontWeight: 800 }}}}>
            {{(user?.name || user?.email || "A").slice(0, 1).toUpperCase()}}
          </Avatar>
        </Tooltip>
      </Toolbar>
    </AppBar>

    <Drawer
      variant="permanent"
      sx={{{{ display: {{ xs: "none", md: "block" }}, width: drawerWidth, flexShrink: 0,
        "& .MuiDrawer-paper": {{ width: drawerWidth, boxSizing: "border-box", borderRightColor: "divider" }},
      }}}}
    >
      <Toolbar sx={{{{ minHeight: "72px !important", px: "20px !important" }}}}>
        <Stack direction="row" alignItems="center" spacing={{1.25}}>
          <Box sx={{{{ display: "grid", placeItems: "center", width: 36, height: 36, borderRadius: 2.5, bgcolor: "primary.main", color: "white", fontWeight: 900 }}}}>A</Box>
          <Box><Typography fontWeight={{900}} lineHeight={{1.1}}>{title}</Typography><Typography variant="caption" color="text.secondary">Agent-native CRM</Typography></Box>
        </Stack>
      </Toolbar>
      <Divider />
      <Box sx={{{{ flex: 1, overflowY: "auto" }}}}>{{navigationList}}</Box>
      <Divider />
      <List sx={{{{ p: 1.5 }}}}><ListItemButton onClick={{logout}} sx={{{{ minHeight: 44, borderRadius: 2.5 }}}}><ListItemIcon sx={{{{ minWidth: 40 }}}}><AxlIcon name="logout" /></ListItemIcon><ListItemText primary="Sign out" /></ListItemButton></List>
    </Drawer>

    <Box component="main" id="main-content" tabIndex={{-1}} sx={{{{
      ml: {{ md: `${{drawerWidth}}px` }}, pt: {{ xs: "80px", md: "92px" }},
      pb: {{ xs: "96px", md: 4 }}, px: {{ xs: 2, sm: 3, lg: 4 }}, minWidth: 0,
    }}}}>{{children}}</Box>

    {{mobile && <Paper elevation={{12}} sx={{{{ position: "fixed", zIndex: theme.zIndex.appBar, left: 0, right: 0, bottom: 0, pb: "env(safe-area-inset-bottom)", borderRadius: 0 }}}}>
      <BottomNavigation
        showLabels
        value={{mobileValue >= 0 ? mobileValue : mobileNavigation.length}}
        onChange={{(_, value) => {{
          if (value === mobileNavigation.length) setMoreOpen(true);
          else if (mobileNavigation[value]) go(mobileNavigation[value].path);
        }}}}
        aria-label="Primary mobile navigation"
      >
        {{mobileNavigation.map((item) => <BottomNavigationAction key={{item.key}} label={{item.label}} icon={{<AxlIcon name={{iconForResource(item.key)}} size={{21}} />}} />)}}
        <BottomNavigationAction label="More" icon={{<AxlIcon name="more" size={{21}} />}} />
      </BottomNavigation>
    </Paper>}}

    <Drawer anchor="bottom" open={{moreOpen}} onClose={{() => setMoreOpen(false)}} PaperProps={{{{ sx: {{ maxHeight: "78dvh", borderRadius: "20px 20px 0 0", pb: "env(safe-area-inset-bottom)" }} }}}}>
      <Box sx={{{{ width: 44, height: 4, bgcolor: "divider", borderRadius: 4, mx: "auto", mt: 1.5 }}}} />
      <Stack direction="row" alignItems="center" sx={{{{ px: 2, pt: 1 }}}}>
        <Typography variant="h6" fontWeight={{800}} sx={{{{ flex: 1 }}}}>All sections</Typography>
        <IconButton onClick={{() => setMoreOpen(false)}} aria-label="Close navigation"><AxlIcon name="close" /></IconButton>
      </Stack>
      {{navigationList}}
      <Divider />
      <List sx={{{{ px: 1.5, pb: 2 }}}}><ListItemButton onClick={{logout}} sx={{{{ borderRadius: 2.5 }}}}><ListItemIcon sx={{{{ minWidth: 40 }}}}><AxlIcon name="logout" /></ListItemIcon><ListItemText primary="Sign out" /></ListItemButton></List>
    </Drawer>
  </Box>;
}};
"##);
    fs::write(output.join("src/components/AxlAppShell.tsx"), content)?;
    Ok(())
}

fn generate_data_table(output: &Path) -> Result<()> {
    let content = r#"import { useEffect, useMemo, useState } from "react";
import { BaseRecord, useDelete, useList } from "@refinedev/core";
import {
  ColumnDef, RowSelectionState, SortingState, VisibilityState, flexRender,
  getCoreRowModel, useReactTable,
} from "@tanstack/react-table";
import {
  Alert, Box, Button, Card, CardContent, Checkbox, Chip, CircularProgress,
  Divider, IconButton, InputAdornment, Menu, MenuItem, Paper, Skeleton,
  Stack, Table, TableBody, TableCell, TableContainer, TableHead, TablePagination,
  TableRow, TableSortLabel, TextField, ToggleButton, ToggleButtonGroup,
  Tooltip, Typography,
} from "@mui/material";
import { useNavigate } from "react-router-dom";
import { AxlIcon } from "./iconRegistry";

export type AxlColumnKind = "text" | "number" | "money" | "date" | "status";

export type AxlTableColumn = {
  key: string;
  label: string;
  kind?: AxlColumnKind;
  priority?: 1 | 2 | 3;
  minWidth?: number;
};

type Props = {
  resource: string;
  entityLabel: string;
  createPath: string;
  columns: AxlTableColumn[];
  initialPageSize?: number;
  initialDensity?: "compact" | "comfortable";
  mobileMode?: "cards" | "table";
};

const useDebounced = (value: string, delay: number) => {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(value), delay);
    return () => window.clearTimeout(timer);
  }, [delay, value]);
  return debounced;
};

const statusColor = (value: unknown): "success" | "warning" | "error" | "info" | "default" => {
  const normalized = String(value ?? "").toLowerCase();
  if (["active", "hot", "closed", "done", "won", "high"].includes(normalized)) return "success";
  if (["warm", "pending", "proposal", "medium"].includes(normalized)) return "warning";
  if (["lost", "blocked", "overdue"].includes(normalized)) return "error";
  if (["open", "new", "discovery", "low"].includes(normalized)) return "info";
  return "default";
};

const renderValue = (value: unknown, kind: AxlColumnKind = "text") => {
  if (value === null || value === undefined || value === "") return <Typography component="span" color="text.disabled">—</Typography>;
  if (kind === "status") return <Chip size="small" label={String(value)} color={statusColor(value)} sx={{ fontWeight: 700, textTransform: "capitalize" }} />;
  if (kind === "money") return new Intl.NumberFormat(undefined, { style: "currency", currency: "USD", maximumFractionDigits: 0 }).format(Number(value));
  if (kind === "number") return new Intl.NumberFormat().format(Number(value));
  if (kind === "date") {
    const date = new Date(String(value));
    return Number.isNaN(date.getTime()) ? String(value) : new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(date);
  }
  return String(value);
};

export const AxlDataTable = ({
  resource, entityLabel, createPath, columns,
  initialPageSize = 25, initialDensity = "compact", mobileMode = "cards",
}: Props) => {
  const navigate = useNavigate();
  const collectionLabel = resource.slice(0, 1).toUpperCase() + resource.slice(1);
  const [page, setPage] = useState(0);
  const [pageSize, setPageSize] = useState(initialPageSize);
  const [search, setSearch] = useState("");
  const [filterValue, setFilterValue] = useState("all");
  const [sorting, setSorting] = useState<SortingState>([]);
  const [rowSelection, setRowSelection] = useState<RowSelectionState>({});
  const [columnMenu, setColumnMenu] = useState<HTMLElement | null>(null);
  const [visibility, setVisibility] = useState<VisibilityState>(() => {
    try { return JSON.parse(localStorage.getItem(`axl:columns:${resource}`) || "{}"); }
    catch { return {}; }
  });
  const [density, setDensity] = useState<"compact" | "comfortable">(() =>
    localStorage.getItem(`axl:density:${resource}`) === "comfortable" ? "comfortable" : initialDensity
  );
  const debouncedSearch = useDebounced(search, 300);
  const filterColumn = columns.find((column) => column.kind === "status");

  useEffect(() => { localStorage.setItem(`axl:columns:${resource}`, JSON.stringify(visibility)); }, [resource, visibility]);
  useEffect(() => { localStorage.setItem(`axl:density:${resource}`, density); }, [density, resource]);
  useEffect(() => { setPage(0); }, [debouncedSearch, filterValue, sorting]);

  const filters = [
    ...(debouncedSearch ? [{ field: "q", operator: "contains", value: debouncedSearch }] : []),
    ...(filterColumn && filterValue !== "all" ? [{ field: filterColumn.key, operator: "eq", value: filterValue }] : []),
  ];

  const query = useList<BaseRecord>({
    resource,
    pagination: { current: page + 1, pageSize },
    sorters: sorting.map((sort) => ({ field: sort.id, order: sort.desc ? "desc" : "asc" })),
    filters: filters as never,
  });
  const rows = query.data?.data ?? [];
  const total = query.data?.total ?? 0;
  const { mutate: deleteRecord } = useDelete();

  const definitions = useMemo<ColumnDef<BaseRecord>[]>(() => columns.map((column) => ({
    id: column.key,
    accessorKey: column.key,
    header: column.label,
    meta: column,
  })), [columns]);

  const table = useReactTable({
    data: rows,
    columns: definitions,
    state: { sorting, columnVisibility: visibility, rowSelection },
    onSortingChange: setSorting,
    onColumnVisibilityChange: setVisibility,
    onRowSelectionChange: setRowSelection,
    getCoreRowModel: getCoreRowModel(),
    getRowId: (row) => String(row.id),
    manualSorting: true,
    manualPagination: true,
    pageCount: Math.max(1, Math.ceil(total / pageSize)),
    enableRowSelection: true,
  });

  const visibleColumns = table.getVisibleLeafColumns();
  const mobileColumns = visibleColumns
    .filter((column) => ((column.columnDef.meta as AxlTableColumn | undefined)?.priority ?? 3) <= 2)
    .slice(0, 5);
  const filterOptions = useMemo(() => {
    if (!filterColumn) return [];
    return [...new Set(rows.map((row) => row[filterColumn.key]).filter(Boolean).map(String))].sort();
  }, [filterColumn, rows]);

  const remove = (id: BaseRecord["id"]) => {
    if (id === undefined || id === null) return;
    if (!window.confirm(`Delete this ${entityLabel.toLowerCase()}? This action cannot be undone.`)) return;
    deleteRecord({ resource, id }, { onSuccess: () => query.refetch() });
  };

  const actions = (row: BaseRecord) => <Stack direction="row" spacing={0.25} justifyContent="flex-end">
    <Tooltip title="View"><IconButton size="small" aria-label={`View ${entityLabel} ${row.id}`} onClick={() => navigate(`/${resource}/show/${row.id}`)}><AxlIcon name="view" size={17} /></IconButton></Tooltip>
    <Tooltip title="Edit"><IconButton size="small" aria-label={`Edit ${entityLabel} ${row.id}`} onClick={() => navigate(`/${resource}/edit/${row.id}`)}><AxlIcon name="edit" size={17} /></IconButton></Tooltip>
    <Tooltip title="Delete"><IconButton size="small" color="error" aria-label={`Delete ${entityLabel} ${row.id}`} onClick={() => remove(row.id)}><AxlIcon name="delete" size={17} /></IconButton></Tooltip>
  </Stack>;

  return <Stack spacing={2.5}>
    <Stack direction={{ xs: "column", sm: "row" }} alignItems={{ xs: "stretch", sm: "center" }} spacing={2}>
      <Box sx={{ flex: 1 }}>
        <Typography variant="h4" fontWeight={900}>{collectionLabel}</Typography>
        <Typography color="text.secondary">{total} records · synchronized with the CRM backend</Typography>
      </Box>
      <Button variant="contained" size="large" startIcon={<AxlIcon name="create" size={18} />} onClick={() => navigate(createPath)}>Create {entityLabel}</Button>
    </Stack>

    <Paper variant="outlined" sx={{ overflow: "hidden", borderRadius: 3 }}>
      <Stack direction={{ xs: "column", lg: "row" }} spacing={1.5} sx={{ p: 2 }} alignItems={{ lg: "center" }}>
        <TextField
          size="small" value={search} onChange={(event) => setSearch(event.target.value)}
          placeholder={`Search ${collectionLabel.toLowerCase()}`}
          inputProps={{ "aria-label": `Search ${collectionLabel.toLowerCase()}` }}
          InputProps={{ startAdornment: <InputAdornment position="start"><AxlIcon name="search" size={18} /></InputAdornment> }}
          sx={{ minWidth: { lg: 300 }, flex: { xs: 1, lg: 0 } }}
        />
        {filterColumn && <TextField select size="small" label={filterColumn.label} value={filterValue} onChange={(event) => setFilterValue(event.target.value)} sx={{ minWidth: 160 }}>
          <MenuItem value="all">All {filterColumn.label.toLowerCase()}</MenuItem>
          {filterOptions.map((option) => <MenuItem key={option} value={option}>{option}</MenuItem>)}
        </TextField>}
        <Box sx={{ flex: 1 }} />
        {Object.keys(rowSelection).length > 0 && <Chip label={`${Object.keys(rowSelection).length} selected`} onDelete={() => setRowSelection({})} />}
        <ToggleButtonGroup exclusive size="small" value={density} onChange={(_, value) => { if (value) setDensity(value); }} aria-label="Table density" sx={{ display: { xs: "none", sm: "flex" } }}>
          <ToggleButton value="compact">Compact</ToggleButton><ToggleButton value="comfortable">Comfort</ToggleButton>
        </ToggleButtonGroup>
        <Tooltip title="Choose columns"><IconButton aria-label="Choose visible columns" onClick={(event) => setColumnMenu(event.currentTarget)}><AxlIcon name="columns" /></IconButton></Tooltip>
        <Menu anchorEl={columnMenu} open={Boolean(columnMenu)} onClose={() => setColumnMenu(null)}>
          {table.getAllLeafColumns().map((column) => <MenuItem key={column.id} onClick={column.getToggleVisibilityHandler()}>
            <Checkbox size="small" checked={column.getIsVisible()} />{String(column.columnDef.header)}
          </MenuItem>)}
        </Menu>
      </Stack>
      <Divider />

      {query.isError && <Alert severity="error" sx={{ m: 2 }}>The records could not be loaded. Check the backend connection and retry.</Alert>}

      <TableContainer sx={{ display: { xs: mobileMode === "table" ? "block" : "none", md: "block" }, maxHeight: "calc(100dvh - 310px)", minHeight: 280 }}>
        <Table stickyHeader size={density === "compact" ? "small" : "medium"} aria-label={`${entityLabel} records`}>
          <TableHead><TableRow>
            <TableCell padding="checkbox"><Checkbox size="small" checked={table.getIsAllPageRowsSelected()} indeterminate={table.getIsSomePageRowsSelected()} onChange={table.getToggleAllPageRowsSelectedHandler()} inputProps={{ "aria-label": "Select all visible records" }} /></TableCell>
            {table.getHeaderGroups()[0]?.headers.map((header) => {
              const meta = header.column.columnDef.meta as AxlTableColumn | undefined;
              const direction = header.column.getIsSorted();
              return <TableCell key={header.id} sx={{ minWidth: meta?.minWidth ?? 140, fontWeight: 800 }} sortDirection={direction || false}>
                <TableSortLabel active={Boolean(direction)} direction={direction || "asc"} onClick={header.column.getToggleSortingHandler()}>
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </TableSortLabel>
              </TableCell>;
            })}
            <TableCell align="right" sx={{ width: 136, fontWeight: 800 }}>Actions</TableCell>
          </TableRow></TableHead>
          <TableBody>
            {query.isLoading && Array.from({ length: 6 }).map((_, index) => <TableRow key={index}><TableCell colSpan={visibleColumns.length + 2}><Skeleton height={density === "compact" ? 28 : 38} /></TableCell></TableRow>)}
            {!query.isLoading && table.getRowModel().rows.map((row) => <TableRow key={row.id} hover selected={row.getIsSelected()}>
              <TableCell padding="checkbox"><Checkbox size="small" checked={row.getIsSelected()} onChange={row.getToggleSelectedHandler()} inputProps={{ "aria-label": `Select ${entityLabel} ${row.original.id}` }} /></TableCell>
              {row.getVisibleCells().map((cell) => {
                const meta = cell.column.columnDef.meta as AxlTableColumn | undefined;
                return <TableCell key={cell.id}>{renderValue(cell.getValue(), meta?.kind)}</TableCell>;
              })}
              <TableCell align="right">{actions(row.original)}</TableCell>
            </TableRow>)}
            {!query.isLoading && rows.length === 0 && <TableRow><TableCell colSpan={visibleColumns.length + 2} align="center" sx={{ py: 8 }}><Typography fontWeight={800}>No records found</Typography><Typography color="text.secondary">Try changing the search or create the first record.</Typography></TableCell></TableRow>}
          </TableBody>
        </Table>
      </TableContainer>

      <Stack sx={{ display: { xs: mobileMode === "cards" ? "flex" : "none", md: "none" }, p: 1.5 }} spacing={1.25}>
        {query.isLoading && Array.from({ length: 4 }).map((_, index) => <Skeleton key={index} variant="rounded" height={150} />)}
        {!query.isLoading && table.getRowModel().rows.map((row) => {
          const primary = mobileColumns[0];
          return <Card component="article" variant="outlined" key={row.id} sx={{ borderRadius: 2.5 }}>
            <CardContent sx={{ "&:last-child": { pb: 2 } }}>
              <Stack direction="row" alignItems="flex-start" spacing={1}>
                <Checkbox size="small" checked={row.getIsSelected()} onChange={row.getToggleSelectedHandler()} inputProps={{ "aria-label": `Select ${entityLabel} ${row.original.id}` }} sx={{ ml: -1 }} />
                <Box sx={{ flex: 1, minWidth: 0 }}>
                  <Typography fontWeight={850} noWrap>{primary ? renderValue(row.original[primary.id], (primary.columnDef.meta as AxlTableColumn | undefined)?.kind) : `${entityLabel} ${row.original.id}`}</Typography>
                  <Typography variant="caption" color="text.secondary">Record #{String(row.original.id)}</Typography>
                </Box>
                {actions(row.original)}
              </Stack>
              <Divider sx={{ my: 1.5 }} />
              <Box sx={{ display: "grid", gridTemplateColumns: "minmax(90px, .7fr) minmax(0, 1.3fr)", gap: 1 }}>
                {mobileColumns.slice(1).map((column) => {
                  const meta = column.columnDef.meta as AxlTableColumn | undefined;
                  return <Box key={column.id} sx={{ display: "contents" }}><Typography variant="caption" color="text.secondary">{String(column.columnDef.header)}</Typography><Typography variant="body2" sx={{ minWidth: 0, overflowWrap: "anywhere" }}>{renderValue(row.original[column.id], meta?.kind)}</Typography></Box>;
                })}
              </Box>
            </CardContent>
          </Card>;
        })}
        {!query.isLoading && rows.length === 0 && <Box sx={{ textAlign: "center", py: 7 }}><Typography fontWeight={850}>No records found</Typography><Typography color="text.secondary">Try another search or create a new record.</Typography></Box>}
      </Stack>

      <Divider />
      <TablePagination
        component="div" count={total} page={page} rowsPerPage={pageSize}
        onPageChange={(_, nextPage) => setPage(nextPage)}
        onRowsPerPageChange={(event) => { setPageSize(Number(event.target.value)); setPage(0); }}
        rowsPerPageOptions={[...new Set([10, initialPageSize, 25, 50, 100])].sort((a, b) => a - b)}
        ActionsComponent={query.isFetching ? () => <Box sx={{ px: 2 }}><CircularProgress size={18} /></Box> : undefined}
      />
    </Paper>
  </Stack>;
};
"#;
    fs::write(output.join("src/components/AxlDataTable.tsx"), content)?;
    Ok(())
}

fn generate_app_tsx(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let mut imports = String::new();
    let mut routes = String::new();
    let mut resources = String::new();
    
    for entity in &app.entities {
        let entity_lower = entity.name.to_lowercase();
        let resource = &entity.table_name;
        let icon = match resource.as_str() {
            "customers" | "contacts" => "customer",
            "leads" => "lead",
            "deals" | "opportunities" => "deal",
            "activities" => "activity",
            "tasks" => "task",
            "notes" => "note",
            _ => "customer",
        };
        for (suffix, file) in [("List", "list"), ("Create", "create"), ("Edit", "edit"), ("Show", "show")] {
            imports.push_str(&format!(
                "const {}{} = lazy(() => import('./pages/{}/{}').then(module => ({{ default: module.{}{} }})))\n",
                entity.name, suffix, entity_lower, file, entity.name, suffix
            ));
        }
        
        routes.push_str(&format!(
            "            <Route path=\"{}\" element={{<{}List />}} />\n",
            resource, entity.name
        ));
        routes.push_str(&format!(
            "            <Route path=\"{}/create\" element={{<{}Create />}} />\n",
            resource, entity.name
        ));
        routes.push_str(&format!(
            "            <Route path=\"{}/edit/:id\" element={{<{}Edit />}} />\n",
            resource, entity.name
        ));
        routes.push_str(&format!(
            "            <Route path=\"{}/show/:id\" element={{<{}Show />}} />\n",
            resource, entity.name
        ));
        
        resources.push_str(&format!(
            "          {{ name: \"{}\", list: \"/{}\", create: \"/{}/create\", edit: \"/{}/edit/:id\", show: \"/{}/show/:id\", meta: {{ label: \"{}\", icon: \"{}\" }} }},\n",
            resource, resource, resource, resource, resource, entity.name, icon
        ));
    }
    
    let content = format!(
        r##"import {{ lazy, Suspense }} from "react";
import {{ Authenticated, AuthProvider, Refine }} from "@refinedev/core";
import {{ AuthPage, RefineSnackbarProvider, useNotificationProvider }} from "@refinedev/mui";
import {{ Box, CircularProgress, CssBaseline, GlobalStyles, ThemeProvider, createTheme }} from "@mui/material";
import {{ BrowserRouter, Outlet, Route, Routes }} from "react-router-dom";
import routerProvider from "@refinedev/react-router-v6";
import dataProvider from "@refinedev/simple-rest";
import axios from "axios";
import {{ AxlAppShell }} from "./components/AxlAppShell";

{imports}const Dashboard = lazy(() => import("./pages/dashboard").then(module => ({{ default: module.Dashboard }})));
const Reports = lazy(() => import("./pages/reports").then(module => ({{ default: module.Reports }})));
const Settings = lazy(() => import("./pages/settings").then(module => ({{ default: module.Settings }})));

const authProvider: AuthProvider = {{
  login: async ({{ email, password }}) => {{
    const response = await fetch("/api/auth/login", {{ method: "POST", headers: {{ "Content-Type": "application/json" }}, body: JSON.stringify({{ email, password }}) }});
    if (!response.ok) return {{ success: false, error: {{ name: "Login failed", message: "Invalid email or password" }} }};
    const data = await response.json();
    localStorage.setItem("axl_token", data.token);
    localStorage.setItem("axl_user", JSON.stringify(data.user));
    return {{ success: true, redirectTo: "/" }};
  }},
  register: async ({{ email, password, name }}) => {{
    const response = await fetch("/api/auth/register", {{ method: "POST", headers: {{ "Content-Type": "application/json" }}, body: JSON.stringify({{ email, password, name: name || email }}) }});
    if (!response.ok) return {{ success: false, error: {{ name: "Registration failed", message: "Check the form or use another email" }} }};
    const data = await response.json();
    localStorage.setItem("axl_token", data.token);
    localStorage.setItem("axl_user", JSON.stringify(data.user));
    return {{ success: true, redirectTo: "/" }};
  }},
  logout: async () => {{ localStorage.removeItem("axl_token"); localStorage.removeItem("axl_user"); return {{ success: true, redirectTo: "/login" }}; }},
  check: async () => localStorage.getItem("axl_token") ? {{ authenticated: true }} : {{ authenticated: false, redirectTo: "/login" }},
  getIdentity: async () => JSON.parse(localStorage.getItem("axl_user") || "null"),
  onError: async (error) => (error?.statusCode === 401 ? {{ logout: true, redirectTo: "/login", error }} : {{ error }}),
}};

const apiClient = axios.create();
apiClient.interceptors.request.use((config) => {{
  const token = localStorage.getItem("axl_token");
  if (token) config.headers.Authorization = `Bearer ${{token}}`;
  return config;
}});

const restDataProvider = dataProvider("/api", apiClient);
const axlDataProvider = {{
  ...restDataProvider,
  getList: async (params: Parameters<typeof restDataProvider.getList>[0]) => {{
    const current = params.pagination?.current ?? 1;
    const pageSize = params.pagination?.pageSize ?? 25;
    const fieldFilters = (params.filters ?? []).filter((filter) => "field" in filter);
    const search = fieldFilters.find((filter) => filter.field === "q");
    const exact = fieldFilters.find((filter) => filter.field !== "q");
    const sorter = params.sorters?.[0];
    const response = await apiClient.get(`/api/${{params.resource}}`, {{ params: {{
      page: current,
      per_page: pageSize,
      q: search && "value" in search ? search.value : undefined,
      filter_field: exact && "field" in exact ? exact.field : undefined,
      filter_value: exact && "value" in exact ? exact.value : undefined,
      sort: sorter?.field,
      order: sorter?.order,
    }} }});
    return {{ data: response.data.data, total: Number(response.data.total ?? 0) }};
  }},
  getOne: async (params: Parameters<typeof restDataProvider.getOne>[0]) => {{
    const response = await apiClient.get(`/api/${{params.resource}}/${{params.id}}`);
    return {{ data: response.data.data }};
  }},
  create: async (params: Parameters<typeof restDataProvider.create>[0]) => {{
    const response = await apiClient.post(`/api/${{params.resource}}`, params.variables);
    return {{ data: response.data.data }};
  }},
  update: async (params: Parameters<typeof restDataProvider.update>[0]) => {{
    const response = await apiClient.put(`/api/${{params.resource}}/${{params.id}}`, params.variables);
    return {{ data: response.data.data }};
  }},
  deleteOne: async (params: Parameters<typeof restDataProvider.deleteOne>[0]) => {{
    await apiClient.delete(`/api/${{params.resource}}/${{params.id}}`);
    return {{ data: {{ id: params.id }} }};
  }},
}} as typeof restDataProvider;

const theme = createTheme({{
  palette: {{
    mode: "light",
    primary: {{ main: "#5b4bdb", dark: "#4435bb", light: "#ebe9ff" }},
    secondary: {{ main: "#008f80" }},
    background: {{ default: "#f6f7fb", paper: "#ffffff" }},
    divider: "#e6e8f0",
  }},
  shape: {{ borderRadius: 14 }},
  typography: {{
    fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    h4: {{ fontSize: "clamp(1.7rem, 3vw, 2.25rem)", letterSpacing: "-0.035em" }},
    button: {{ fontWeight: 750, textTransform: "none" }},
  }},
  components: {{
    MuiButtonBase: {{ defaultProps: {{ disableRipple: false }} }},
    MuiButton: {{ styleOverrides: {{ root: {{ borderRadius: 10, boxShadow: "none" }} }} }},
    MuiCard: {{ styleOverrides: {{ root: {{ borderColor: "#e6e8f0", boxShadow: "0 8px 28px rgba(25, 28, 50, 0.055)" }} }} }},
    MuiPaper: {{ styleOverrides: {{ root: {{ backgroundImage: "none" }} }} }},
    MuiTableCell: {{ styleOverrides: {{ head: {{ backgroundColor: "#fafafe" }} }} }},
    MuiChip: {{ styleOverrides: {{ root: {{ borderRadius: 8 }} }} }},
  }},
}});

const App: React.FC = () => {{
  return (
    <ThemeProvider theme={{theme}}><RefineSnackbarProvider><BrowserRouter>
      <Refine
        dataProvider={{axlDataProvider}}
        routerProvider={{routerProvider}}
        notificationProvider={{useNotificationProvider}}
        authProvider={{authProvider}}
        resources={{[
{resources}          {{ name: "reports", list: "/reports", meta: {{ label: "Reports" }} }},
          {{ name: "settings", list: "/settings", meta: {{ label: "Settings" }} }},
        ]}}
      >
        <CssBaseline />
        <GlobalStyles styles={{{{
          html: {{ WebkitFontSmoothing: "antialiased" }},
          body: {{ minWidth: 320 }},
          ".axl-skip-link": {{ position: "fixed", top: 8, left: 8, zIndex: 2000, padding: "8px 12px", background: "#fff", borderRadius: 8, transform: "translateY(-150%)" }},
          ".axl-skip-link:focus": {{ transform: "translateY(0)" }},
          "*:focus-visible": {{ outline: "3px solid #00a896", outlineOffset: 2 }},
          "@media (prefers-reduced-motion: reduce)": {{ "*, *::before, *::after": {{ scrollBehavior: "auto !important", transitionDuration: "0.01ms !important", animationDuration: "0.01ms !important" }} }},
        }}}} />
        <Box component="a" className="axl-skip-link" href="#main-content">Skip to main content</Box>
        <Suspense fallback={{<Box display="grid" minHeight="60vh" sx={{{{ placeItems: "center" }}}}><CircularProgress /></Box>}}><Routes>
          <Route path="/login" element={{<AuthPage type="login" />}} />
          <Route path="/register" element={{<AuthPage type="register" />}} />
          <Route element={{<Authenticated key="crm-private" fallback={{<AuthPage type="login" />}}><Outlet /></Authenticated>}}>
            <Route element={{<AxlAppShell><Outlet /></AxlAppShell>}}>
              <Route index element={{<Dashboard />}} />
{routes}              <Route path="reports" element={{<Reports />}} />
              <Route path="settings" element={{<Settings />}} />
            </Route>
          </Route>
        </Routes></Suspense>
      </Refine>
    </BrowserRouter></RefineSnackbarProvider></ThemeProvider>
  );
}};

export default App;
"##
    );
    
    fs::write(output.join("src/App.tsx"), content)?;
    Ok(())
}

fn generate_list_page(
    entity: &crate::analyzer::AnalyzedEntity,
    table_view: Option<&crate::compact_ui::UiView>,
    output: &Path,
) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let entity_name = &entity.name;
    let resource = &entity.table_name;

    let mut columns = String::new();
    let mut page_size = 25i64;
    let mut density = "compact".to_string();
    let mut mobile_mode = "cards".to_string();
    if let Some(view) = table_view {
        page_size = compact_prop_integer(&view.root, 3).unwrap_or(25);
        density = compact_prop_string(&view.root, 4).unwrap_or_else(|| "compact".into());
        mobile_mode = compact_prop_string(&view.root, 5).unwrap_or_else(|| "cards".into());
        if !(1..=100).contains(&page_size) {
            anyhow::bail!("compact UI table '{}' page size must be between 1 and 100", resource);
        }
        if !matches!(density.as_str(), "compact" | "comfortable") {
            anyhow::bail!("compact UI table '{}' density must be compact or comfortable", resource);
        }
        if !matches!(mobile_mode.as_str(), "cards" | "table") {
            anyhow::bail!("compact UI table '{}' mobile mode must be cards or table", resource);
        }
        let mut declared = std::collections::HashSet::new();
        for node in &view.root.children {
            if node.component_id != 65 {
                anyhow::bail!("compact UI table '{}' only accepts column component 65", resource);
            }
            let field_name = compact_prop_string(node, 1)
                .ok_or_else(|| anyhow::anyhow!("compact UI table '{}' has a column without a field", resource))?;
            if !declared.insert(field_name.clone()) {
                anyhow::bail!("compact UI table '{}' repeats column '{}'", resource, field_name);
            }
            if !entity.fields.iter().any(|field| field.name == field_name && !field.is_primary_key) {
                anyhow::bail!("compact UI table '{}' references unknown column '{}'", resource, field_name);
            }
            let label = compact_prop_string(node, 2).unwrap_or_else(|| field_name.replace('_', " "));
            let kind = compact_prop_string(node, 3).unwrap_or_else(|| "text".into());
            let priority = compact_prop_integer(node, 4).unwrap_or(3);
            let width = compact_prop_integer(node, 5).unwrap_or(140);
            if !matches!(kind.as_str(), "text" | "number" | "money" | "date" | "status") {
                anyhow::bail!("compact UI column '{}.{}' has invalid kind '{}'", resource, field_name, kind);
            }
            if !(1..=3).contains(&priority) || !(80..=600).contains(&width) {
                anyhow::bail!("compact UI column '{}.{}' has invalid priority or width", resource, field_name);
            }
            columns.push_str(&format!(
                "  {{ key: \"{}\", label: \"{}\", kind: \"{}\", priority: {}, minWidth: {} }},\n",
                escape_tsx(&field_name), escape_tsx(&label), kind, priority, width
            ));
        }
        if declared.is_empty() {
            anyhow::bail!("compact UI table '{}' requires at least one column", resource);
        }
    } else {
        let mut visible_index = 0usize;
        for field in &entity.fields {
            if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
                visible_index += 1;
                let header = field.name.replace("_", " ")
                    .chars()
                    .enumerate()
                    .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() })
                    .collect::<String>();
                let kind = if matches!(field.name.as_str(), "status" | "stage" | "priority" | "type" | "lifecycle") {
                    "status"
                } else if matches!(field.name.as_str(), "value" | "amount" | "revenue" | "price") {
                    "money"
                } else if matches!(field.field_type.as_str(), "Integer" | "Int" | "Float" | "Double") {
                    "number"
                } else if field.name.contains("date") || field.name.ends_with("_at") {
                    "date"
                } else {
                    "text"
                };
                let priority = if visible_index == 1 || matches!(field.name.as_str(), "name" | "title" | "company" | "contact" | "status" | "stage") {
                    1
                } else if matches!(field.name.as_str(), "email" | "phone" | "owner" | "priority" | "value" | "due_date" | "next_action") {
                    2
                } else {
                    3
                };
                let min_width = if matches!(field.name.as_str(), "name" | "title" | "company" | "email" | "description" | "next_action") { 190 } else { 140 };
                columns.push_str(&format!(
                    "  {{ key: \"{}\", label: \"{}\", kind: \"{}\", priority: {}, minWidth: {} }},\n",
                    field.name, header, kind, priority, min_width
                ));
            }
        }
    }

    let content = format!(
        r#"import {{ AxlDataTable, AxlTableColumn }} from "../../components/AxlDataTable";

const columns: AxlTableColumn[] = [
{columns}];

export const {entity_name}List: React.FC = () => <AxlDataTable
  resource="{resource}"
  entityLabel="{entity_name}"
  createPath="/{resource}/create"
  initialPageSize={{{page_size}}}
  initialDensity="{density}"
  mobileMode="{mobile_mode}"
  columns={{columns}}
/>;
"#
    );
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/list.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_create_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let entity_name = &entity.name;
    let resource = &entity.table_name;
    
    let fields = generate_form_fields(entity);
    
    let content = format!(
        r#"import {{ useForm }} from "@refinedev/react-hook-form";
import {{ Create }} from "@refinedev/mui";
import {{ TextField, Box }} from "@mui/material";

export const {entity_name}Create: React.FC = () => {{
  const {{
    saveButtonProps,
    refineCore: {{ formLoading }},
    register,
    formState: {{ errors }},
  }} = useForm({{
    refineCoreProps: {{ resource: "{resource}" }},
  }});

  return (
    <Create isLoading={{formLoading}} saveButtonProps={{saveButtonProps}}>
      <Box component="form" sx={{{{ display: "flex", flexDirection: "column" }}}} autoComplete="off">
{fields}      </Box>
    </Create>
  );
}};
"#
    );
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/create.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_edit_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let entity_name = &entity.name;
    let resource = &entity.table_name;
    
    let fields = generate_form_fields(entity);
    
    let content = format!(
        r#"import {{ useForm }} from "@refinedev/react-hook-form";
import {{ Edit }} from "@refinedev/mui";
import {{ TextField, Box }} from "@mui/material";

export const {entity_name}Edit: React.FC = () => {{
  const {{
    saveButtonProps,
    refineCore: {{ formLoading }},
    register,
    formState: {{ errors }},
  }} = useForm({{
    refineCoreProps: {{ resource: "{resource}" }},
  }});

  return (
    <Edit isLoading={{formLoading}} saveButtonProps={{saveButtonProps}}>
      <Box component="form" sx={{{{ display: "flex", flexDirection: "column" }}}} autoComplete="off">
{fields}      </Box>
    </Edit>
  );
}};
"#
    );
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/edit.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_form_fields(entity: &crate::analyzer::AnalyzedEntity) -> String {
    let mut fields = String::new();
    for field in &entity.fields {
        if field.is_primary_key || matches!(field.name.as_str(), "created_at" | "updated_at") {
            continue;
        }
        let label = field.name.replace('_', " ")
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() })
            .collect::<String>();
        let validation = if field.optional {
            format!("{{...register(\"{}\")}}", field.name)
        } else {
            format!("{{...register(\"{}\", {{ required: \"{} is required\" }})}}", field.name, label)
        };
        let required = if field.optional { "" } else { "\n            required" };
        let input_type = match field.field_type.as_str() {
            "Integer" | "Int" | "Float" | "Double" => "number",
            "DateTime" => "datetime-local",
            _ if field.name == "email" || field.name.ends_with("_email") => "email",
            _ => "text",
        };
        let multiline = if matches!(field.name.as_str(), "description" | "notes" | "content") {
            "\n            multiline\n            minRows={3}"
        } else {
            ""
        };
        fields.push_str(&format!(
            "          <TextField\n            label=\"{}\"\n            type=\"{}\"\n            {}{}{}\n            error={{!!errors.{} }}\n            helperText={{errors.{}?.message as string | undefined}}\n            fullWidth\n            margin=\"normal\"\n          />\n",
            label, input_type, validation, required, multiline, field.name, field.name
        ));
    }
    fields
}

fn generate_show_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let entity_name = &entity.name;
    let resource = &entity.table_name;
    
    let mut fields = String::new();
    for field in &entity.fields {
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            let label = field.name.replace("_", " ")
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() })
                .collect::<String>();
            fields.push_str(&format!(
                "          <Typography variant=\"subtitle1\">{}: {{record?.{}}}</Typography>\n",
                label, field.name
            ));
        }
    }
    
    let content = format!(
        r#"import {{ useShow }} from "@refinedev/core";
import {{ Show }} from "@refinedev/mui";
import {{ Typography, Stack }} from "@mui/material";

export const {entity_name}Show: React.FC = () => {{
  const {{ queryResult }} = useShow({{
    resource: "{resource}",
  }});
  
  const {{ data, isLoading }} = queryResult;
  const record = data?.data;

  return (
    <Show isLoading={{isLoading}}>
      <Stack spacing={{2}}>
{fields}      </Stack>
    </Show>
  );
}};
"#
    );
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/show.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_dashboard(_app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = r#"import { Alert, Avatar, Box, Card, CardContent, Chip, Divider, Grid, LinearProgress, List, ListItem, ListItemAvatar, ListItemText, Skeleton, Stack, Tab, Tabs, Typography } from "@mui/material";
import { useList } from "@refinedev/core";
import { useState } from "react";

export const Dashboard: React.FC = () => {
  const { data: customers } = useList({ resource: "customers" });
  const { data: leads } = useList({ resource: "leads" });
  const { data: deals } = useList({ resource: "deals" });
  const { data: activities } = useList({ resource: "activities" });
  const { data: tasks } = useList({ resource: "tasks" });
  const [tab, setTab] = useState(0);
  const revenue = deals?.data?.reduce((sum: number, deal: any) => sum + (deal.value || 0), 0) || 0;
  const stats = [
    ["Customers", customers?.total ?? customers?.data?.length ?? 0, "primary"],
    ["Qualified leads", leads?.total ?? leads?.data?.length ?? 0, "warning"],
    ["Open deals", deals?.total ?? deals?.data?.length ?? 0, "success"],
    ["Pipeline value", `$${revenue.toLocaleString()}`, "info"],
  ] as const;

  return (
    <Stack spacing={3}>
      <Box>
        <Typography variant="h4" fontWeight={800}>Good morning</Typography>
        <Typography color="text.secondary">Here is what is happening across your CRM today.</Typography>
      </Box>
      <Alert severity="info">Your sales pipeline is ready. Review high-score leads and overdue tasks.</Alert>
      <Grid container spacing={3}>
        {stats.map(([label, value, color]) => (
          <Grid item xs={12} sm={6} lg={3} key={label}>
            <Card><CardContent><Stack direction="row" justifyContent="space-between"><Box><Typography color="text.secondary">{label}</Typography><Typography variant="h4" fontWeight={800}>{value}</Typography></Box><Avatar sx={{ bgcolor: `${color}.main` }}>{label[0]}</Avatar></Stack></CardContent></Card>
          </Grid>
        ))}
        <Grid item xs={12} lg={8}>
          <Card><CardContent>
            <Stack direction="row" justifyContent="space-between"><Typography variant="h6">Sales pipeline</Typography><Chip label={`${deals?.data?.length ?? 0} deals`} /></Stack>
            <Divider sx={{ my: 2 }} />
            {(deals?.data ?? []).slice(0, 5).map((deal: any) => <Box key={deal.id} sx={{ mb: 2 }}><Stack direction="row" justifyContent="space-between"><Typography>{deal.name}</Typography><Typography fontWeight={700}>${Number(deal.value || 0).toLocaleString()}</Typography></Stack><LinearProgress variant="determinate" value={deal.probability || 0} sx={{ mt: 1, height: 8, borderRadius: 4 }} /></Box>)}
            {!deals && <Skeleton variant="rounded" height={180} />}
          </CardContent></Card>
        </Grid>
        <Grid item xs={12} lg={4}>
          <Card><CardContent>
            <Tabs value={tab} onChange={(_, value) => setTab(value)}><Tab label="Activities" /><Tab label="Tasks" /></Tabs>
            <List>{(tab === 0 ? activities?.data : tasks?.data)?.slice(0, 5).map((item: any) => <ListItem key={item.id}><ListItemAvatar><Avatar>{(item.type || item.title || "A")[0]}</Avatar></ListItemAvatar><ListItemText primary={item.description || item.title} secondary={item.date || item.due_date || item.status} /></ListItem>)}</List>
            {((tab === 0 ? activities?.data : tasks?.data)?.length ?? 0) === 0 && <Typography color="text.secondary" sx={{ py: 4, textAlign: "center" }}>Nothing scheduled yet</Typography>}
          </CardContent></Card>
        </Grid>
      </Grid>
    </Stack>
  );
};
"#;
    
    fs::write(output.join("src/pages/dashboard.tsx"), content)?;
    Ok(())
}

fn generate_compact_dashboard(
    app: &AnalyzedApp,
    views: &[crate::compact_ui::UiView],
    output: &Path,
) -> Result<()> {
    let view = views.first().ok_or_else(|| anyhow::anyhow!("compact UI has no dashboard view"))?;
    if view.root.component_id != 1 {
        anyhow::bail!("compact UI dashboard root must be component 1 (app)");
    }
    validate_compact_bindings(&view.root, app)?;
    let mut hooks = String::new();
    let mut bindings = String::new();
    for entity in &app.entities {
        let variable = entity.table_name.replace('-', "_");
        hooks.push_str(&format!(
            "  const {variable}Query = useList({{ resource: \"{}\", pagination: {{ current: 1, pageSize: 5 }} }});\n",
            entity.table_name
        ));
        bindings.push_str(&format!(
            "    \"{}.total\": {variable}Query.data?.total ?? {variable}Query.data?.data?.length ?? 0,\n",
            entity.table_name
        ));
    }
    let jsx = compact_node_jsx(&view.root, 2)?;
    let content = format!(
        r#"import {{ useList }} from "@refinedev/core";
import {{ Alert, Box, Button, Card, CardContent, Chip, Grid, LinearProgress, Stack, Typography }} from "@mui/material";
import {{ AxlIcon }} from "../components/iconRegistry";

export const Dashboard: React.FC = () => {{
{hooks}  const bindings: Record<string, unknown> = {{
{bindings}  }};

  return (
{jsx}  );
}};
"#
    );
    fs::write(output.join("src/pages/dashboard.tsx"), content)?;
    Ok(())
}

fn validate_compact_bindings(node: &crate::compact_ui::UiNode, app: &AnalyzedApp) -> Result<()> {
    for property in &node.properties {
        if let crate::compact_ui::UiValue::Binding(name) = &property.value {
            let Some((resource, metric)) = name.split_once('.') else {
                anyhow::bail!("compact UI binding '${name}' must use resource.metric");
            };
            if metric != "total" || !app.entities.iter().any(|entity| entity.table_name == resource) {
                anyhow::bail!("compact UI binding '${name}' is not available");
            }
        }
    }
    for child in &node.children { validate_compact_bindings(child, app)?; }
    Ok(())
}

fn compact_node_jsx(node: &crate::compact_ui::UiNode, depth: usize) -> Result<String> {
    let indent = "  ".repeat(depth);
    let child_indent = "  ".repeat(depth + 1);
    let children = node.children.iter().map(|child| compact_node_jsx(child, depth + 1)).collect::<Result<Vec<_>>>()?.join("");
    let text = |id| compact_prop_jsx(node, id, "");
    let string = |id, fallback: &str| compact_prop_string(node, id).unwrap_or_else(|| fallback.to_string());
    let integer = |id, fallback: i64| compact_prop_integer(node, id).unwrap_or(fallback);
    let jsx = match node.component_id {
        1 => format!("{indent}<Stack spacing={{3}} data-axl-node=\"{}\">\n{children}{indent}</Stack>\n", node.id),
        10 => format!("{indent}<Box data-axl-node=\"{}\">\n{children}{indent}</Box>\n", node.id),
        11 => {
            let columns = integer(1, 4).clamp(1, 12);
            let gap = string(2, "16px");
            let wrapped = node.children.iter().map(|child| {
                let rendered = compact_node_jsx(child, depth + 2)?;
                Ok(format!("{child_indent}<Grid item xs={{12}} md={{{}}}>\n{rendered}{child_indent}</Grid>\n", 12 / columns.max(1)))
            }).collect::<Result<Vec<String>>>()?.join("");
            format!("{indent}<Grid container spacing={{2}} sx={{{{ gap: \"{}\" }}}} data-axl-node=\"{}\">\n{wrapped}{indent}</Grid>\n", escape_tsx(&gap), node.id)
        }
        12 => format!("{indent}<Stack direction={{{{ xs: \"column\", md: \"{}\" }}}} spacing={{2}} data-axl-node=\"{}\">\n{children}{indent}</Stack>\n", escape_tsx(&string(1, "row")), node.id),
        33 => format!("{indent}<Typography variant=\"{}\" data-axl-node=\"{}\">{}</Typography>\n", escape_tsx(&string(2, "body1")), node.id, text(1)),
        45 => format!("{indent}<Button variant=\"{}\" data-axl-action=\"{}\">{}</Button>\n", escape_tsx(&string(2, "contained")), node.events.first().map(|event| event.action_id).unwrap_or(0), text(1)),
        50 => format!("{indent}<Alert severity=\"{}\" data-axl-node=\"{}\"><strong>{}</strong> {}</Alert>\n", escape_tsx(&string(3, "info")), node.id, text(1), text(2)),
        54 => format!("{indent}<LinearProgress variant=\"determinate\" value={{{}}} data-axl-node=\"{}\" />\n", integer(1, 0).clamp(0, 100), node.id),
        63 => format!("{indent}<Chip label={{{}}} color=\"{}\" data-axl-node=\"{}\" />\n", compact_prop_attribute(node, 1, ""), escape_tsx(&string(2, "default")), node.id),
        71 => {
            let name = string(1, "dashboard");
            if !matches!(name.as_str(), "activity" | "bell" | "calendar" | "close" | "columns" | "create" | "customer" | "dashboard" | "deal" | "delete" | "edit" | "lead" | "logout" | "more" | "note" | "report" | "search" | "settings" | "task" | "user" | "view") {
                anyhow::bail!("compact UI icon node {} uses unknown semantic icon '{name}'", node.id);
            }
            format!("{indent}<Box sx={{{{ display: \"grid\", placeItems: \"center\", width: 40, height: 40, borderRadius: 2.5, bgcolor: \"primary.light\", color: \"primary.main\" }}}} data-axl-node=\"{}\"><AxlIcon name=\"{}\" /></Box>\n", node.id, escape_tsx(&name))
        }
        72 => format!("{indent}<Card data-axl-node=\"{}\"><CardContent><Typography variant=\"overline\">{}</Typography>\n{children}{indent}</CardContent></Card>\n", node.id, text(1)),
        other => anyhow::bail!("compact UI node {} uses unsupported React component id {other}", node.id),
    };
    Ok(jsx)
}

fn compact_prop_attribute(node: &crate::compact_ui::UiNode, id: i32, fallback: &str) -> String {
    match node.properties.iter().find(|property| property.id == id).map(|property| &property.value) {
        Some(crate::compact_ui::UiValue::String(value)) => format!("\"{}\"", escape_tsx(value)),
        Some(crate::compact_ui::UiValue::Integer(value)) => value.to_string(),
        Some(crate::compact_ui::UiValue::Boolean(value)) => value.to_string(),
        Some(crate::compact_ui::UiValue::Binding(name)) => format!("String(bindings[\"{}\"] ?? \"—\")", escape_tsx(name)),
        None => format!("\"{}\"", escape_tsx(fallback)),
    }
}

fn compact_prop_jsx(node: &crate::compact_ui::UiNode, id: i32, fallback: &str) -> String {
    match node.properties.iter().find(|property| property.id == id).map(|property| &property.value) {
        Some(crate::compact_ui::UiValue::String(value)) => escape_tsx(value),
        Some(crate::compact_ui::UiValue::Integer(value)) => value.to_string(),
        Some(crate::compact_ui::UiValue::Boolean(value)) => value.to_string(),
        Some(crate::compact_ui::UiValue::Binding(name)) => format!("{{String(bindings[\"{}\"] ?? \"—\")}}", escape_tsx(name)),
        None => escape_tsx(fallback),
    }
}

fn compact_prop_string(node: &crate::compact_ui::UiNode, id: i32) -> Option<String> {
    node.properties.iter().find(|property| property.id == id).and_then(|property| match &property.value {
        crate::compact_ui::UiValue::String(value) => Some(value.clone()),
        _ => None,
    })
}

fn compact_prop_integer(node: &crate::compact_ui::UiNode, id: i32) -> Option<i64> {
    node.properties.iter().find(|property| property.id == id).and_then(|property| match property.value {
        crate::compact_ui::UiValue::Integer(value) => Some(value),
        _ => None,
    })
}

fn escape_tsx(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn generate_admin_pages(output: &Path) -> Result<()> {
    let reports = r#"import { Alert, Box, Card, CardContent, Grid, LinearProgress, Stack, Tab, Tabs, Typography } from "@mui/material";
import { useList } from "@refinedev/core";
import { useState } from "react";

export const Reports: React.FC = () => {
  const [tab, setTab] = useState(0);
  const { data: leads } = useList({ resource: "leads" });
  const { data: deals } = useList({ resource: "deals" });
  const pipeline = deals?.data?.reduce((sum: number, deal: any) => sum + Number(deal.value || 0), 0) || 0;
  return <Stack spacing={3}><Box><Typography variant="h4" fontWeight={800}>Reports</Typography><Typography color="text.secondary">Pipeline health and conversion overview</Typography></Box><Tabs value={tab} onChange={(_, value) => setTab(value)}><Tab label="Sales" /><Tab label="Acquisition" /></Tabs><Alert severity="success">{leads?.data?.length || 0} leads are being tracked across the workspace.</Alert><Grid container spacing={3}>{[25, 48, 72, 86].map((value, index) => <Grid item xs={12} md={6} key={value}><Card><CardContent><Typography variant="h6">{["Lead conversion", "Proposal velocity", "Win probability", "Pipeline coverage"][index]}</Typography><Typography variant="h4">{index === 3 ? `$${pipeline.toLocaleString()}` : `${value}%`}</Typography><LinearProgress value={value} variant="determinate" sx={{ mt: 2, height: 8, borderRadius: 4 }} /></CardContent></Card></Grid>)}</Grid></Stack>;
};
"#;
    let settings = r#"import { Alert, Button, Card, CardContent, Checkbox, FormControlLabel, MenuItem, Stack, Tab, Tabs, TextField, Typography } from "@mui/material";
import { useState } from "react";

export const Settings: React.FC = () => {
  const [tab, setTab] = useState(0);
  const [saved, setSaved] = useState(false);
  return <Stack spacing={3}><Typography variant="h4" fontWeight={800}>Settings</Typography><Tabs value={tab} onChange={(_, value) => setTab(value)}><Tab label="Profile" /><Tab label="Workspace" /><Tab label="Notifications" /></Tabs>{saved && <Alert severity="success">Settings saved</Alert>}<Card><CardContent><Stack spacing={2} component="form" onSubmit={(event) => { event.preventDefault(); setSaved(true); }}><TextField label="Display name" defaultValue="AXL Admin" /><TextField label="Workspace" defaultValue="AXL CRM" /><TextField select label="Locale" defaultValue="en"><MenuItem value="en">English</MenuItem><MenuItem value="it">Italiano</MenuItem></TextField><TextField label="Notes" multiline minRows={3} /><FormControlLabel control={<Checkbox defaultChecked />} label="Email notifications" /><Button type="submit" variant="contained">Save changes</Button></Stack></CardContent></Card></Stack>;
};
"#;
    fs::write(output.join("src/pages/reports.tsx"), reports)?;
    fs::write(output.join("src/pages/settings.tsx"), settings)?;
    Ok(())
}
