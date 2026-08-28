use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::ir::GraphIr;
use super::runtime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRenderResult {
    pub path: String,
    pub flow: String,
    pub output_type: String,
    pub data: Value,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiFormRenderResult {
    pub path: String,
    pub entity: String,
    pub flow: String,
    pub submit: String,
    pub html: String,
}

pub fn ui_manifest(graph: &GraphIr) -> Value {
    let uis = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "ui")
        .map(|ui| {
            let mut pages = children(graph, &ui.id, "page");
            pages.sort_by_key(|page| {
                page.metadata
                    .get("order")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(usize::MAX)
            });
            let mut forms = children(graph, &ui.id, "form");
            forms.sort_by_key(|form| {
                form.metadata
                    .get("order")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(usize::MAX)
            });
            json!({
                "name": ui.name,
                "pages": pages.into_iter().map(|page| {
                    let (input, output) = page.type_name.as_deref()
                        .and_then(|value| value.split_once("->"))
                        .unwrap_or(("", ""));
                    json!({
                        "path": page.metadata.get("path"),
                        "input": input,
                        "output": output,
                        "flow": page.metadata.get("flow"),
                    })
                }).collect::<Vec<_>>(),
                "forms": forms.into_iter().map(|form| {
                    let (entity, output) = form.type_name.as_deref()
                        .and_then(|value| value.split_once("->"))
                        .unwrap_or(("", ""));
                    json!({
                        "path": form.metadata.get("path"),
                        "entity": entity,
                        "output": output,
                        "flow": form.metadata.get("flow"),
                        "submit": form.metadata.get("submit"),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "protocol": "axl-ui/1",
        "uis": uis,
    })
}

pub fn render_page(graph: &GraphIr, path: &str, input: Value) -> Result<UiRenderResult, String> {
    let normalized = normalize_path(path);
    let page = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "page")
        .find(|node| {
            node.metadata
                .get("path")
                .is_some_and(|value| normalize_path(value) == normalized)
        })
        .ok_or_else(|| format!("ui_page_not_found:{path}"))?;
    let flow = page
        .metadata
        .get("flow")
        .cloned()
        .ok_or_else(|| "ui_page_has_no_flow".to_string())?;
    let output_type = page
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->").map(|(_, output)| output.to_string()))
        .unwrap_or_else(|| "text".to_string());
    let data = runtime::evaluate_flow(graph, &flow, input).map_err(|error| error.0)?;
    let nav = render_nav(graph);
    let html = render_page_html(graph, &graph.app, path, &output_type, &data, &nav);
    Ok(UiRenderResult {
        path: path.into(),
        flow,
        output_type,
        data,
        html,
    })
}

pub fn render_form(graph: &GraphIr, path: &str) -> Result<UiFormRenderResult, String> {
    let normalized = normalize_path(path);
    let form = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "form")
        .find(|node| {
            node.metadata
                .get("path")
                .is_some_and(|value| normalize_path(value) == normalized)
        })
        .ok_or_else(|| format!("ui_form_not_found:{path}"))?;
    let entity = form
        .metadata
        .get("entity")
        .cloned()
        .or_else(|| {
            form.type_name
                .as_deref()
                .and_then(|value| value.split_once("->").map(|(entity, _)| entity.to_string()))
        })
        .ok_or_else(|| "ui_form_has_no_entity".to_string())?;
    let flow = form
        .metadata
        .get("flow")
        .cloned()
        .ok_or_else(|| "ui_form_has_no_flow".to_string())?;
    let submit = form
        .metadata
        .get("submit")
        .cloned()
        .ok_or_else(|| "ui_form_has_no_submit".to_string())?;
    let nav = render_nav(graph);
    let html = render_form_html(graph, &graph.app, path, &entity, &submit, &nav);
    Ok(UiFormRenderResult {
        path: path.into(),
        entity,
        flow,
        submit,
        html,
    })
}

fn render_nav(graph: &GraphIr) -> String {
    let mut links = Vec::new();
    for node in &graph.nodes {
        if node.kind != "page" && node.kind != "form" {
            continue;
        }
        let Some(path) = node.metadata.get("path") else {
            continue;
        };
        let label = if node.kind == "form" {
            format!("{path} (form)")
        } else {
            path.clone()
        };
        links.push((path.clone(), label));
    }
    links.sort_by(|left, right| left.0.cmp(&right.0));
    if links.is_empty() {
        return String::new();
    }
    let items = links
        .into_iter()
        .map(|(path, label)| format!(r#"    <a href="{path}">{label}</a>"#))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"  <nav class="nav">
    <strong>Navigation</strong>
{items}
  </nav>
"#
    )
}

fn render_page_html(
    graph: &GraphIr,
    app: &str,
    path: &str,
    output_type: &str,
    data: &Value,
    nav: &str,
) -> String {
    let title = format!("{app}{path}");
    if let Some(table) = render_items_table(graph, output_type, data) {
        return wrap_html(&title, nav, &table);
    }
    let fields = collect_fields(graph, output_type, data);
    let rows = fields
        .iter()
        .map(|(label, value)| format!("    <dt>{label}</dt>\n    <dd>{value}</dd>"))
        .collect::<Vec<_>>()
        .join("\n");
    wrap_html(
        &title,
        nav,
        &format!(
            r#"  <dl>
{rows}
  </dl>"#
        ),
    )
}

fn render_form_html(
    graph: &GraphIr,
    app: &str,
    path: &str,
    entity: &str,
    submit: &str,
    nav: &str,
) -> String {
    let title = format!("{app}{path}");
    let fields = entity_fields(graph, entity);
    let inputs = fields
        .iter()
        .map(|field| render_form_field(graph, field))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        r#"  <p class="meta">Submit entity JSON to <code>{submit}</code> via POST API.</p>
  <form method="post" action="{submit}">
{inputs}
    <button type="submit">Submit</button>
  </form>"#
    );
    wrap_html(&title, nav, &body)
}

fn render_form_field(graph: &GraphIr, field: &super::ir::GraphNode) -> String {
    let name = &field.name;
    let type_name = field.type_name.as_deref().unwrap_or("text");
    let optional = field_optional(field);
    let required = if optional { "" } else { " required" };
    let label = format!(r#"    <label for="{name}">{name}</label>"#);
    if let Some(variants) = enum_variants(graph, type_name) {
        let options = variants
            .iter()
            .map(|variant| format!(r#"      <option value="{variant}">{variant}</option>"#))
            .collect::<Vec<_>>()
            .join("\n");
        return format!(
            r#"  <div class="field">
{label}
    <select id="{name}" name="{name}"{required}>
{options}
    </select>
  </div>"#
        );
    }
    let input_type = match type_name {
        "int" | "float" | "money" => "number",
        "bool" => "checkbox",
        "email" => "email",
        _ => "text",
    };
    format!(
        r#"  <div class="field">
{label}
    <input id="{name}" name="{name}" type="{input_type}"{required}>
  </div>"#
    )
}

fn field_optional(field: &super::ir::GraphNode) -> bool {
    field
        .metadata
        .get("qualifiers")
        .is_some_and(|values| values.split(',').any(|value| value == "optional"))
        || field
            .type_name
            .as_deref()
            .is_some_and(|type_name| type_name.starts_with("Option<"))
}

fn enum_variants(graph: &GraphIr, type_name: &str) -> Option<Vec<String>> {
    let enum_node = graph
        .nodes
        .iter()
        .find(|node| node.kind == "enum" && node.name == type_name)?;
    let mut variants = children(graph, &enum_node.id, "variant")
        .into_iter()
        .map(|variant| variant.name.clone())
        .collect::<Vec<_>>();
    variants.sort();
    Some(variants)
}

fn entity_fields<'a>(graph: &'a GraphIr, entity_name: &str) -> Vec<&'a super::ir::GraphNode> {
    graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == entity_name)
        .map(|entity| {
            let mut fields = children(graph, &entity.id, "field");
            fields.sort_by(|left, right| left.name.cmp(&right.name));
            fields
        })
        .unwrap_or_default()
}

fn wrap_html(title: &str, nav: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    :root {{ color-scheme: light dark; font-family: ui-sans-serif, system-ui, sans-serif; }}
    body {{ margin: 2rem auto; max-width: 56rem; line-height: 1.5; }}
    h1 {{ font-size: 1.5rem; margin-bottom: 1rem; }}
    .nav {{ display: flex; flex-wrap: wrap; gap: 0.75rem; margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 1px solid #ccc; }}
    .nav a {{ color: inherit; }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{ border: 1px solid #ccc; padding: 0.4rem 0.6rem; text-align: left; }}
    th {{ background: #f4f4f4; }}
    dl {{ display: grid; grid-template-columns: minmax(8rem, 30%) 1fr; gap: 0.5rem 1rem; }}
    dt {{ font-weight: 600; color: #555; }}
    dd {{ margin: 0; }}
    .field {{ margin-bottom: 1rem; }}
    .field label {{ display: block; font-weight: 600; margin-bottom: 0.25rem; }}
    .field input, .field select {{ width: 100%; padding: 0.4rem 0.6rem; }}
    .meta {{ color: #666; font-size: 0.875rem; margin-bottom: 1.5rem; }}
    button {{ margin-top: 0.5rem; padding: 0.5rem 1rem; }}
  </style>
</head>
<body>
  <h1>{title}</h1>
  <p class="meta">Rendered from AXL UI IR (axl-ui/1)</p>
{nav}{body}
</body>
</html>
"#
    )
}

fn render_items_table(graph: &GraphIr, output_type: &str, data: &Value) -> Option<String> {
    let payload = if let Some(ok) = data.get("ok") {
        ok
    } else {
        data
    };
    let Value::Object(map) = payload else {
        return None;
    };
    let Value::Array(items) = map.get("items")? else {
        return None;
    };
    if items.is_empty() {
        return Some("  <p>No items.</p>".into());
    }
    let item_type = page_item_type(graph, output_type)?;
    let columns = entity_field_names(graph, &item_type);
    if columns.is_empty() {
        return None;
    }
    let header = columns
        .iter()
        .map(|column| format!("      <th>{column}</th>"))
        .collect::<Vec<_>>()
        .join("\n");
    let rows = items
        .iter()
        .filter_map(|item| {
            let Value::Object(row) = item else {
                return None;
            };
            let cells = columns
                .iter()
                .map(|column| {
                    let value = row.get(column).map(display_value).unwrap_or_default();
                    format!("        <td>{value}</td>")
                })
                .collect::<Vec<_>>()
                .join("\n");
            Some(format!("      <tr>\n{cells}\n      </tr>"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!(
        "  <table>\n    <thead>\n      <tr>\n{header}\n      </tr>\n    </thead>\n    <tbody>\n{rows}\n    </tbody>\n  </table>"
    ))
}

fn page_item_type(graph: &GraphIr, output_type: &str) -> Option<String> {
    let entity_name = strip_result(output_type);
    let entity = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == entity_name)?;
    let items_field = children(graph, &entity.id, "field")
        .into_iter()
        .find(|field| field.name == "items")?;
    items_field
        .type_name
        .as_deref()
        .and_then(|type_name| {
            type_name
                .strip_prefix("List<")
                .and_then(|value| value.strip_suffix('>'))
        })
        .map(str::to_string)
}

fn entity_field_names(graph: &GraphIr, entity_name: &str) -> Vec<String> {
    graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == entity_name)
        .map(|entity| {
            children(graph, &entity.id, "field")
                .into_iter()
                .map(|field| field.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn collect_fields(graph: &GraphIr, output_type: &str, data: &Value) -> Vec<(String, String)> {
    if let Some(error) = data.get("error") {
        return vec![("error".into(), display_value(error))];
    }
    if let Some(ok) = data.get("ok") {
        return collect_fields(graph, strip_result(output_type), ok);
    }
    if is_scalar_type(output_type) {
        return vec![(output_type.into(), display_value(data))];
    }
    if let Some(entity) = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == output_type)
    {
        let fields = children(graph, &entity.id, "field");
        if let Value::Object(map) = data {
            return fields
                .iter()
                .filter_map(|field| {
                    map.get(&field.name)
                        .map(|value| (field.name.clone(), display_value(value)))
                })
                .collect();
        }
    }
    if let Value::Object(map) = data {
        return map
            .iter()
            .map(|(key, value)| (key.clone(), display_value(value)))
            .collect();
    }
    vec![("value".into(), display_value(data))]
}

fn strip_result(type_name: &str) -> &str {
    type_name
        .strip_prefix("Result<")
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(type_name)
}

fn is_scalar_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "text"
            | "string"
            | "email"
            | "uuid"
            | "datetime"
            | "duration"
            | "int"
            | "float"
            | "money"
            | "bool"
            | "unit"
    )
}

fn display_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

fn normalize_path(path: &str) -> String {
    if path == "/" {
        return "/".into();
    }
    path.trim_end_matches('/').to_string()
}

fn children<'a>(graph: &'a GraphIr, owner: &str, kind: &str) -> Vec<&'a super::ir::GraphNode> {
    let by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == owner)
        .filter_map(|edge| by_id.get(edge.to.as_str()).copied())
        .filter(|node| node.kind == kind)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_source;

    const BALANCE_UI: &str = r#"axl 4
app BalanceUI

entity BalanceInput
  income: money required
  expense: money required

flow CalculateBalance BalanceInput -> money
  let balance = input.income - input.expense
  return balance

ui BalanceScreen
  page /balance BalanceInput -> money = CalculateBalance
"#;

    const MOVEMENT_UI: &str = r#"axl 4
app MovementUI

enum MovementKind
  income
  expense

entity Movement
  id: uuid required
  kind: MovementKind required
  amount: money required
  category: text required

entity MovementView
  direction: text required
  signed_amount: money required
  category: text required

flow BuildMovementView Movement -> MovementView
  let direction = if input.kind == MovementKind.income then "Entrata" else "Uscita"
  match signed_amount: money = input.kind
    income => input.amount
    expense => -input.amount
  make view: MovementView
    direction = direction
    signed_amount = signed_amount
    category = input.category
  return view

ui MovementScreen
  page /view Movement -> MovementView = BuildMovementView
"#;

    const FORM_UI: &str = r#"axl 4
app FormUI

enum ClienteStato
  attivo
  inattivo

entity Cliente
  nome: text required
  email: email required
  budget: money required
  priorita: int optional
  stato: ClienteStato required

flow CreaCliente Cliente -> Result<Cliente>
  require input.nome != "" else "nome_required"
  return input

api ClienteApi
  post /clienti Cliente -> Result<Cliente> = CreaCliente

ui ClienteScreen
  page /clienti unit -> text = EchoClienti
  form /clienti/new Cliente -> Result<Cliente> = CreaCliente submit /clienti

flow EchoClienti unit -> text
  return "clienti"
"#;

    #[test]
    fn ui_manifest_lists_declared_pages() {
        let graph = compile_source(BALANCE_UI).unwrap().graph;
        let manifest = ui_manifest(&graph);
        assert_eq!(manifest["protocol"], "axl-ui/1");
        assert_eq!(manifest["uis"][0]["pages"][0]["flow"], "CalculateBalance");
    }

    #[test]
    fn ui_manifest_lists_declared_forms() {
        let graph = compile_source(FORM_UI).unwrap().graph;
        let manifest = ui_manifest(&graph);
        assert_eq!(manifest["uis"][0]["forms"][0]["submit"], "/clienti");
        assert_eq!(manifest["uis"][0]["forms"][0]["entity"], "Cliente");
    }

    #[test]
    fn render_page_evaluates_flow_and_emits_html() {
        let graph = compile_source(BALANCE_UI).unwrap().graph;
        let rendered = render_page(
            &graph,
            "/balance",
            json!({"income": 125000, "expense": 45000}),
        )
        .unwrap();
        assert_eq!(rendered.data, json!(80000));
        assert!(rendered.html.contains("<!DOCTYPE html>"));
        assert!(rendered.html.contains("80000"));
        assert!(rendered.html.contains("<dt>money</dt>"));
    }

    #[test]
    fn render_page_displays_entity_fields() {
        let graph = compile_source(MOVEMENT_UI).unwrap().graph;
        let rendered = render_page(
            &graph,
            "/view",
            json!({
                "id": "m1",
                "kind": "income",
                "amount": 125000,
                "category": "consulting"
            }),
        )
        .unwrap();
        assert_eq!(rendered.data["direction"], "Entrata");
        assert!(rendered.html.contains("<dt>direction</dt>"));
        assert!(rendered.html.contains("Entrata"));
        assert!(rendered.html.contains("<dt>signed_amount</dt>"));
    }

    #[test]
    fn render_form_emits_inputs_and_nav() {
        let graph = compile_source(FORM_UI).unwrap().graph;
        let rendered = render_form(&graph, "/clienti/new").unwrap();
        assert_eq!(rendered.submit, "/clienti");
        assert!(
            rendered
                .html
                .contains(r#"<form method="post" action="/clienti">"#)
        );
        assert!(rendered.html.contains(r#"name="nome""#));
        assert!(
            rendered
                .html
                .contains(r#"<option value="attivo">attivo</option>"#)
        );
        assert!(rendered.html.contains("/clienti/new (form)"));
        assert!(rendered.html.contains(r#"<a href="/clienti">/clienti</a>"#));
    }
}
