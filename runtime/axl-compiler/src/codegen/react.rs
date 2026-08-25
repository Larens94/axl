use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("src/pages"))?;
    fs::create_dir_all(output.join("src/components"))?;
    
    // Generate package.json
    generate_package_json(app, output)?;
    
    // Generate vite.config.ts
    generate_vite_config(output)?;
    
    // Generate tsconfig.json
    generate_tsconfig(output)?;
    
    // Generate index.html
    generate_index_html(app, output)?;
    
    // Generate src/main.tsx
    generate_main_tsx(output)?;
    
    // Generate src/App.tsx
    generate_app_tsx(app, output)?;
    
    // Generate pages
    for entity in &app.entities {
        generate_list_page(entity, output)?;
        generate_create_page(entity, output)?;
    }
    
    // Generate dashboard
    generate_dashboard(app, output)?;
    
    Ok(())
}

fn generate_package_json(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = format!(r#"{{
  "name": "{}",
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
    "@mui/material": "^5.0.0",
    "@mui/icons-material": "^5.0.0",
    "@mui/x-data-grid": "^6.0.0",
    "@emotion/react": "^11.0.0",
    "@emotion/styled": "^11.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "react-router-dom": "^6.0.0",
    "@tanstack/react-query": "^5.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "@vitejs/plugin-react": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0"
  }}
}}"#, app.name.to_lowercase().replace(" ", "-"));
    
    fs::write(output.join("package.json"), content)?;
    Ok(())
}

fn generate_vite_config(output: &Path) -> Result<()> {
    let content = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
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
    Ok(())
}

fn generate_index_html(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = format!(r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#, app.name);
    
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
        imports.push_str(&format!(
            "import {{ {}List }} from './pages/{}/list'\n",
            entity.name, entity_lower
        ));
        imports.push_str(&format!(
            "import {{ {}Create }} from './pages/{}/create'\n",
            entity.name, entity_lower
        ));
        
        routes.push_str(&format!(
            "          <Route path=\"{}s\" element={{<{}List />}} />\n",
            entity_lower, entity.name
        ));
        routes.push_str(&format!(
            "          <Route path=\"{}s/create\" element={{<{}Create />}} />\n",
            entity_lower, entity.name
        ));
        
        resources.push_str(&format!(
            "          {{ name: \"{}s\", list: \"/{}s\", create: \"/{}s/create\" }},\n",
            entity_lower, entity_lower, entity_lower
        ));
    }
    
    let content = format!(r#"import {{ Refine }} from "@refinedev/core";
import {{ RefineThemes, useNotificationProvider }} from "@refinedev/mui";
import {{ CssBaseline, GlobalStyles }} from "@mui/material";
import {{ BrowserRouter, Outlet, Route, Routes }} from "react-router-dom";
import routerProvider from "@refinedev/react-router-v6";
import dataProvider from "@refinedev/simple-rest";

{imports}

const App: React.FC = () => {{
  return (
    <BrowserRouter>
      <Refine
        dataProvider={{dataProvider("http://localhost:3000/api")}}
        routerProvider={{routerProvider}}
        notificationProvider={{useNotificationProvider}}
        resources={{[
{resources}        ]}}
      >
        <CssBaseline />
        <GlobalStyles styles={{{{ html: {{ WebkitFontSmoothing: "auto" }} }} }} />
        <Routes>
          <Route path="/" element={{<Outlet />}}>
{routes}          </Route>
        </Routes>
      </Refine>
    </BrowserRouter>
  );
}};

export default App;
"#);
    
    fs::write(output.join("src/App.tsx"), content)?;
    Ok(())
}

fn generate_list_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    
    let mut columns = String::new();
    for field in &entity.fields {
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            columns.push_str(&format!(
                "          {{ field: \"{}\", headerName: \"{}\", flex: 1 }},\n",
                field.name,
                field.name.replace("_", " ").chars().enumerate().map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() }).collect::<String>()
            ));
        }
    }
    
    let content = format!(r#"import {{ useDataGrid }} from "@refinedev/mui";
import {{ DataGrid, GridColDef }} from "@mui/x-data-grid";
import {{ List }} from "@refinedev/mui";
import {{ Button }} from "@mui/material";
import {{ useNavigate }} from "react-router-dom";

export const {}List: React.FC = () => {{
  const {{ dataGridProps }} = useDataGrid({{
    resource: "{}s",
  }});
  
  const navigate = useNavigate();

  const columns: GridColDef[] = [
{columns}  ];

  return (
    <List
      headerButtons={{(
        <Button
          variant="contained"
          onClick={{() => navigate("/{}s/create")}}
        >
          Create {}
        </Button>
      )}}
    >
      <DataGrid {{...dataGridProps}} columns={{columns}} autoHeight />
    </List>
  );
}};
"#, entity.name, entity_lower, entity_lower, entity.name);
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/list.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_create_page(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    
    let mut fields = String::new();
    for field in &entity.fields {
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            fields.push_str(&format!(
                "          <TextField\n            label=\"{}\"\n            {{...register(\"{}\", {{ required: true }})}}\n            fullWidth\n            margin=\"normal\"\n          />\n",
                field.name.replace("_", " ").chars().enumerate().map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_string() }).collect::<String>(),
                field.name
            ));
        }
    }
    
    let content = format!(r#"import {{ useForm }} from "@refinedev/react-hook-form";
import {{ Create }} from "@refinedev/mui";
import {{ TextField, Box }} from "@mui/material";

export const {}Create: React.FC = () => {{
  const {{
    saveButtonProps,
    refineCore: {{ formLoading }},
    register,
    formState: {{ errors }},
  }} = useForm({{
    resource: "{}s",
  }});

  return (
    <Create isLoading={{formLoading}} saveButtonProps={{saveButtonProps}}>
      <Box component="form" sx={{ display: "flex", flexDirection: "column" }} autoComplete="off">
{fields}      </Box>
    </Create>
  );
}};
"#, entity.name, entity_lower);
    
    fs::create_dir_all(output.join(format!("src/pages/{}", entity_lower)))?;
    fs::write(output.join(format!("src/pages/{}/create.tsx", entity_lower)), content)?;
    Ok(())
}

fn generate_dashboard(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = r#"import { Card, CardContent, Typography, Grid } from "@mui/material";

export const Dashboard: React.FC = () => {
  return (
    <Grid container spacing={3}>
      <Grid item xs={12} md={3}>
        <Card>
          <CardContent>
            <Typography variant="h6">Customers</Typography>
            <Typography variant="h4">-</Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid item xs={12} md={3}>
        <Card>
          <CardContent>
            <Typography variant="h6">Leads</Typography>
            <Typography variant="h4">-</Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid item xs={12} md={3}>
        <Card>
          <CardContent>
            <Typography variant="h6">Deals</Typography>
            <Typography variant="h4">-</Typography>
          </CardContent>
        </Card>
      </Grid>
      <Grid item xs={12} md={3}>
        <Card>
          <CardContent>
            <Typography variant="h6">Revenue</Typography>
            <Typography variant="h4">-</Typography>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
};
"#;
    
    fs::write(output.join("src/pages/dashboard.tsx"), content)?;
    Ok(())
}
