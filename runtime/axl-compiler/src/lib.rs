pub mod parser;
pub mod analyzer;
pub mod codegen;
pub mod compact_ui;

use std::path::Path;
use std::fs;

use anyhow::{Context, Result};

/// Compile one AXL application into all of its platform artifacts.
///
/// AXL is the source language; Rust, React/TypeScript and SQL are implementation
/// targets kept below this boundary.
pub fn compile_application(input: &Path, output: &Path) -> Result<()> {
    let app = parser::parse_file(input)
        .with_context(|| format!("cannot parse AXL application '{}'", input.display()))?;
    let analyzed = analyzer::analyze(app).context("AXL application analysis failed")?;
    let compact_views = load_compact_ui(input)?;

    codegen::rust::generate(&analyzed, &output.join("backend"))
        .context("Rust backend generation failed")?;
    codegen::react::generate(&analyzed, compact_views.as_deref(), &output.join("frontend"))
        .context("frontend generation failed")?;
    codegen::sql::generate(&analyzed, &output.join("backend/migrations"))
        .context("SQL migration generation failed")?;
    generate_workspace_scripts(output).context("workspace runner generation failed")?;

    Ok(())
}

pub fn load_compact_ui(input: &Path) -> Result<Option<Vec<compact_ui::UiView>>> {
    let path = input.with_file_name(format!(
        "{}.ui.axl",
        input.file_stem().and_then(|name| name.to_str()).unwrap_or("app")
    ));
    if path.exists() { Ok(Some(compact_ui::parse_file(&path)?)) } else { Ok(None) }
}

fn generate_workspace_scripts(output: &Path) -> Result<()> {
    let dev = r#"#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export JWT_SECRET=${JWT_SECRET:-axl-local-demo-secret-change-in-production}
export DATABASE_URL=${DATABASE_URL:-sqlite://$ROOT/backend/app.db?mode=rwc}
cleanup() { kill "${BACKEND_PID:-}" "${FRONTEND_PID:-}" 2>/dev/null || true; }
trap cleanup EXIT INT TERM
(cd "$ROOT/backend" && "${CARGO:-cargo}" run) & BACKEND_PID=$!
(cd "$ROOT/frontend" && { test -d node_modules || npm install; } && npm run dev) & FRONTEND_PID=$!
echo "AXL CRM: http://localhost:5173/register"
wait
"#;
    let smoke = r#"#!/bin/sh
set -eu
API=${API_URL:-http://localhost:3000/api}
UNAUTH=$(curl -s -o /dev/null -w '%{http_code}' "$API/customers")
test "$UNAUTH" = "401"
EMAIL="axl-smoke-$(date +%s)@example.test"
AUTH=$(curl -fsS -X POST "$API/auth/register" -H 'Content-Type: application/json' -d "{\"email\":\"$EMAIL\",\"password\":\"demo-password\",\"name\":\"AXL Smoke Agent\"}")
TOKEN=$(printf '%s' "$AUTH" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
test -n "$TOKEN"
CUSTOMERS=$(curl -fsS "$API/customers?page=1&per_page=1" -H "Authorization: Bearer $TOKEN")
printf '%s' "$CUSTOMERS" | grep -q '"data"'
printf '%s' "$CUSTOMERS" | grep -q '"total"'
SEARCH=$(curl -fsS "$API/customers?page=1&per_page=25&q=Northstar" -H "Authorization: Bearer $TOKEN")
printf '%s' "$SEARCH" | grep -q 'Northstar Labs'
printf '%s' "$SEARCH" | grep -q '"total":1'
FILTER=$(curl -fsS "$API/deals?page=1&per_page=25&filter_field=stage&filter_value=proposal" -H "Authorization: Bearer $TOKEN")
printf '%s' "$FILTER" | grep -q 'Northstar AI rollout'
printf '%s' "$FILTER" | grep -q '"total":1'
SORT=$(curl -fsS "$API/customers?page=1&per_page=1&sort=name&order=asc" -H "Authorization: Bearer $TOKEN")
printf '%s' "$SORT" | grep -q 'Luca Ferri'
echo "AXL smoke test passed: auth, JWT, pagination, search, filters and sorting"
"#;
    fs::write(output.join("dev.sh"), dev)?;
    fs::write(output.join("smoke-test.sh"), smoke)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output.join("dev.sh"), fs::Permissions::from_mode(0o755))?;
        fs::set_permissions(output.join("smoke-test.sh"), fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_stack_generation_contains_admin_and_auth_surfaces() {
        let root = std::env::temp_dir().join(format!("axl_full_stack_{}", std::process::id()));
        let input = root.join("crm.axl");
        let output = root.join("build");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&input, r#"
entity Customer {
  field name: String
  field email: String?
  field status: String = "active"
}
seed Customer {
  name: "Demo Customer"
}
api Customer {
  query page 25 max 100 sort created_at desc
  GET /api/customers -> list
  POST /api/customers -> create
}
auth {
  POST /api/auth/login -> login
  POST /api/auth/register -> register
}
ui Dashboard {
  components: [app-shell, data-table, stat-card]
}
"#).unwrap();

        compile_application(&input, &output).unwrap();
        let backend = std::fs::read_to_string(output.join("backend/src/auth.rs")).unwrap();
        let frontend = std::fs::read_to_string(output.join("frontend/src/App.tsx")).unwrap();
        let dashboard = std::fs::read_to_string(output.join("frontend/src/pages/dashboard.tsx")).unwrap();
        let generated_main = std::fs::read_to_string(output.join("backend/src/main.rs")).unwrap();
        let customer_handler = std::fs::read_to_string(output.join("backend/src/handlers/customer.rs")).unwrap();
        let customer_create = std::fs::read_to_string(output.join("frontend/src/pages/customer/create.tsx")).unwrap();
        assert!(backend.contains("Argon2::default().verify_password"));
        assert!(backend.contains("pub async fn require_auth"));
        assert!(!backend.contains("NOT_IMPLEMENTED"));
        assert!(generated_main.contains("Demo Customer"));
        assert!(generated_main.contains("route_layer"));
        assert!(customer_handler.contains("unwrap_or(25).clamp(1, 100)"));
        assert!(customer_handler.contains("order_by_desc(Column::CreatedAt)"));
        assert!(customer_handler.contains("\"total\": total"));
        assert!(frontend.contains("<AxlAppShell>"));
        assert!(frontend.contains("<RefineSnackbarProvider>"));
        assert!(frontend.contains("AuthPage type=\"register\""));
        assert!(frontend.contains("Authorization = `Bearer"));
        assert!(frontend.contains("dataProvider={axlDataProvider}"));
        assert!(frontend.contains("per_page: pageSize"));
        assert!(frontend.contains("filter_field"));
        assert!(customer_handler.contains("Condition::any()"));
        assert!(customer_handler.contains("requested_sort"));
        let table = std::fs::read_to_string(output.join("frontend/src/components/AxlDataTable.tsx")).unwrap();
        let shell = std::fs::read_to_string(output.join("frontend/src/components/AxlAppShell.tsx")).unwrap();
        assert!(table.contains("useReactTable"));
        assert!(table.contains("mobileMode === \"cards\""));
        assert!(shell.contains("BottomNavigation"));
        assert!(shell.contains("safe-area-inset-bottom"));
        assert!(customer_create.contains("register(\"email\")"));
        assert!(!customer_create.contains("register(\"email\", { required"));
        assert!(output.join("dev.sh").exists());
        assert!(output.join("smoke-test.sh").exists());
        assert!(dashboard.contains("LinearProgress"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn compact_ui_sidecar_drives_react_layout() {
        let root = std::env::temp_dir().join(format!("axl_compact_ui_{}", std::process::id()));
        let input = root.join("demo.axl");
        let output = root.join("build");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&input, "entity Customer {\n  field name: String\n}\napi Customer {\n  GET /api/customers -> list\n}\n").unwrap();
        std::fs::write(root.join("demo.ui.axl"), "3;60|1;61|1|1;62|1|\"Demo\";61|2|72;62|1|\"Accounts from compact UI\";61|3|33;62|1|$customers.total;62|2|\"h3\";99;99;99;60|2;61|10|64;62|1|\"customers\";62|2|\"Customer\";62|3|#25;62|4|\"compact\";62|5|\"cards\";61|11|65;62|1|\"name\";62|2|\"Account name\";62|3|\"text\";62|4|#1;62|5|#240;99;99").unwrap();
        compile_application(&input, &output).unwrap();
        let dashboard = std::fs::read_to_string(output.join("frontend/src/pages/dashboard.tsx")).unwrap();
        assert!(dashboard.contains("Accounts from compact UI"));
        assert!(dashboard.contains("bindings[\"customers.total\"]"));
        assert!(dashboard.contains("data-axl-node=\"2\""));
        let customer_list = std::fs::read_to_string(output.join("frontend/src/pages/customer/list.tsx")).unwrap();
        assert!(customer_list.contains("label: \"Account name\""));
        assert!(customer_list.contains("minWidth: 240"));
        assert!(customer_list.contains("mobileMode=\"cards\""));
        std::fs::remove_dir_all(root).unwrap();
    }
}
