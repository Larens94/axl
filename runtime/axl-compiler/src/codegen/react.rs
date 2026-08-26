use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("src/pages"))?;
    fs::create_dir_all(output.join("src/components"))?;
    
    generate_package_json(app, output)?;
    generate_vite_config(output)?;
    generate_tsconfig(output)?;
    generate_index_html(app, output)?;
    generate_main_tsx(output)?;
    generate_app_tsx(app, output)?;
    
    for entity in &app.entities {
        generate_list_page(entity, output)?;
        generate_create_page(entity, output)?;
        generate_edit_page(entity, output)?;
        generate_show_page(entity, output)?;
    }
    
    generate_dashboard(app, output)?;
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
    "@emotion/react": "^11.0.0",
    "@emotion/styled": "^11.0.0",
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
          framework: ['react', 'react-dom', 'react-router-dom', '@refinedev/core', '@refinedev/react-router-v6', '@refinedev/simple-rest', '@refinedev/mui', '@refinedev/react-hook-form', '@mui/material', '@mui/icons-material', '@mui/x-data-grid', '@emotion/react', '@emotion/styled']
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

fn generate_app_tsx(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let mut imports = String::new();
    let mut routes = String::new();
    let mut resources = String::new();
    
    for entity in &app.entities {
        let entity_lower = entity.name.to_lowercase();
        let resource = &entity.table_name;
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
            "          {{ name: \"{}\", list: \"/{}\", create: \"/{}/create\", edit: \"/{}/edit/:id\", show: \"/{}/show/:id\", meta: {{ label: \"{}\" }} }},\n",
            resource, resource, resource, resource, resource, entity.name
        ));
    }
    
    let content = format!(
        r##"import {{ lazy, Suspense }} from "react";
import {{ Authenticated, AuthProvider, Refine }} from "@refinedev/core";
import {{ AuthPage, ThemedLayoutV2, ThemedTitleV2, useNotificationProvider }} from "@refinedev/mui";
import {{ Box, CircularProgress, CssBaseline, GlobalStyles, ThemeProvider, createTheme }} from "@mui/material";
import {{ BrowserRouter, Outlet, Route, Routes }} from "react-router-dom";
import routerProvider from "@refinedev/react-router-v6";
import dataProvider from "@refinedev/simple-rest";
import axios from "axios";

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
    const response = await apiClient.get(`/api/${{params.resource}}`, {{ params: {{ page: current, per_page: pageSize }} }});
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
  palette: {{ mode: "light", primary: {{ main: "#5b4bdb" }}, secondary: {{ main: "#00a896" }}, background: {{ default: "#f5f6fa" }} }},
  shape: {{ borderRadius: 12 }},
  typography: {{ fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif' }},
  components: {{
    MuiButtonBase: {{ defaultProps: {{ disableRipple: false }} }},
    MuiCard: {{ styleOverrides: {{ root: {{ border: "1px solid #e8e9f1", boxShadow: "0 8px 28px rgba(25, 28, 50, 0.06)" }} }} }},
  }},
}});

const App: React.FC = () => {{
  return (
    <ThemeProvider theme={{theme}}><BrowserRouter>
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
            <Route element={{<ThemedLayoutV2 Title={{() => <ThemedTitleV2 text="AXL CRM" collapsed={{false}} />}}><Box component="main" id="main-content" tabIndex={{-1}} sx={{{{ minWidth: 0 }}}}><Outlet /></Box></ThemedLayoutV2>}}>
              <Route index element={{<Dashboard />}} />
{routes}              <Route path="reports" element={{<Reports />}} />
              <Route path="settings" element={{<Settings />}} />
            </Route>
          </Route>
        </Routes></Suspense>
      </Refine>
    </BrowserRouter></ThemeProvider>
  );
}};

export default App;
"##
    );
    
    fs::write(output.join("src/App.tsx"), content)?;
    Ok(())
}

fn generate_list_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let entity_name = &entity.name;
    let resource = &entity.table_name;
    
    let mut columns = String::new();
    for field in &entity.fields {
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            let header = field.name.replace("_", " ")
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() })
                .collect::<String>();
            let presentation = if matches!(field.name.as_str(), "status" | "stage" | "priority" | "type") {
                ", renderCell: (params) => <Chip size=\"small\" label={params.value ?? \"—\"} color={params.value === \"active\" || params.value === \"hot\" || params.value === \"closed\" ? \"success\" : \"default\"} />"
            } else {
                ""
            };
            columns.push_str(&format!(
                "          {{ field: \"{}\", headerName: \"{}\", flex: 1, minWidth: 150{} }},\n",
                field.name, header, presentation
            ));
        }
    }
    
    let content = format!(
        r#"import {{ useDataGrid }} from "@refinedev/mui";
import {{ DataGrid, GridColDef }} from "@mui/x-data-grid";
import {{ List, EditButton, ShowButton, DeleteButton }} from "@refinedev/mui";
import {{ Box, Button, Chip, InputAdornment, MenuItem, Stack, TextField }} from "@mui/material";
import SearchIcon from "@mui/icons-material/Search";
import {{ useNavigate }} from "react-router-dom";
import {{ useState }} from "react";

export const {entity_name}List: React.FC = () => {{
  const {{ dataGridProps }} = useDataGrid({{
    resource: "{resource}",
  }});
  
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("all");

  const columns: GridColDef[] = [
{columns}    {{
      field: "actions",
      headerName: "Actions",
      width: 144,
      sortable: false,
      filterable: false,
      renderCell: (params) => (
        <Stack direction="row" spacing={{1}}>
          <EditButton hideText aria-label={{`Edit {entity_name} ${{params.row.id}}`}} recordItemId={{params.row.id}} />
          <ShowButton hideText aria-label={{`View {entity_name} ${{params.row.id}}`}} recordItemId={{params.row.id}} />
          <DeleteButton hideText aria-label={{`Delete {entity_name} ${{params.row.id}}`}} recordItemId={{params.row.id}} />
        </Stack>
      ),
    }},
  ];

  return (
    <List
      headerButtons={{(
        <Button
          variant="contained"
          onClick={{() => navigate("/{resource}/create")}}
        >
          Create {entity_name}
        </Button>
      )}}
    >
      <Stack direction={{{{ xs: "column", md: "row" }}}} spacing={{2}} sx={{{{ mb: 2 }}}}>
        <TextField
          size="small"
          value={{search}}
          onChange={{(event) => setSearch(event.target.value)}}
          placeholder="Search {resource}"
          inputProps={{{{ "aria-label": "Search {resource}" }}}}
          InputProps={{{{ startAdornment: <InputAdornment position="start"><SearchIcon /></InputAdornment> }}}}
        />
        <TextField select size="small" label="Status" value={{status}} onChange={{(event) => setStatus(event.target.value)}} sx={{{{ minWidth: 160 }}}}>
          <MenuItem value="all">All statuses</MenuItem>
          <MenuItem value="active">Active</MenuItem>
          <MenuItem value="open">Open</MenuItem>
          <MenuItem value="closed">Closed</MenuItem>
        </TextField>
        <Chip label={{`${{(dataGridProps.rows ?? []).length}} records`}} variant="outlined" />
      </Stack>
      <Box sx={{{{ overflowX: "auto" }}}}>
        <DataGrid
          {{...dataGridProps}}
          rows={{(dataGridProps.rows ?? []).filter((row) => JSON.stringify(row).toLowerCase().includes(search.toLowerCase()) && (status === "all" || row.status === status))}}
          columns={{columns}}
          autoHeight
          disableRowSelectionOnClick
          pageSizeOptions={{[10, 25, 50]}}
          sx={{{{ minWidth: 720, "& .MuiDataGrid-cell:focus-within": {{ outlineOffset: -3 }} }}}}
        />
      </Box>
    </List>
  );
}};
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
