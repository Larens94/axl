use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::http::{
    match_http_path, parse_bound_scalar, path_template_matches, substitute_path_template,
};
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

pub fn matches_exact_ui_path(graph: &GraphIr, path: &str) -> bool {
    let normalized = normalize_path(path);
    graph.nodes.iter().any(|node| {
        matches!(node.kind.as_str(), "page" | "form")
            && node
                .metadata
                .get("path")
                .is_some_and(|value| !value.contains('{') && normalize_path(value) == normalized)
    })
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
            let mut actions = children(graph, &ui.id, "ui_action");
            actions.sort_by_key(|action| {
                action
                    .metadata
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
                    let path = page.metadata.get("path").cloned().unwrap_or_default();
                    let template = path.contains('{').then(|| path.clone());
                    json!({
                        "path": path,
                        "template": template,
                        "input": input,
                        "output": output,
                        "flow": page.metadata.get("flow"),
                        "input_source": page.metadata.get("input_source"),
                        "input_name": page.metadata.get("input_name"),
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
                        "redirect": form.metadata.get("redirect"),
                    })
                }).collect::<Vec<_>>(),
                "actions": actions.into_iter().map(|action| {
                    json!({
                        "path": action.metadata.get("path"),
                        "method": action.metadata.get("method"),
                        "submit": action.metadata.get("submit"),
                        "redirect": action.metadata.get("redirect"),
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
    let mut runtime = runtime::BuiltinRuntime::new().map_err(|error| error.0)?;
    render_page_with_runtime(graph, &mut runtime, path, input)
}

pub fn render_page_with_runtime(
    graph: &GraphIr,
    provider_runtime: &mut dyn runtime::ProviderRuntime,
    path: &str,
    input: Value,
) -> Result<UiRenderResult, String> {
    let (page, _) = find_page(graph, path).ok_or_else(|| format!("ui_page_not_found:{path}"))?;
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
    let input = bind_page_input(page, path, input)?;
    let data = runtime::evaluate_flow_with_runtime(graph, &flow, input, provider_runtime)
        .map_err(|error| error.0)?;
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
    let body = if let Some(table) = render_items_table(graph, path, output_type, data) {
        table
    } else {
        let fields = collect_fields(graph, output_type, data);
        let rows = fields
            .iter()
            .map(|(label, value)| format!("    <dt>{label}</dt>\n    <dd>{value}</dd>"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            r#"  <dl>
{rows}
  </dl>"#
        )
    };
    let actions = render_page_actions(graph, path, data);
    wrap_html(&title, nav, &format!("{body}{actions}"))
}

fn render_page_actions(graph: &GraphIr, page_path: &str, page_data: &Value) -> String {
    let normalized = normalize_path(page_path);
    let mut actions = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "ui_action")
        .filter(|action| {
            action
                .metadata
                .get("redirect")
                .is_some_and(|redirect| path_template_matches(redirect, &normalized))
        })
        .collect::<Vec<_>>();
    actions.sort_by_key(|action| {
        action
            .metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    if actions.is_empty() {
        return String::new();
    }
    let forms = actions
        .iter()
        .map(|action| render_action_form(graph, action, page_path, page_data))
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n  <section class=\"actions\">\n{forms}\n  </section>")
}

fn render_action_form(
    graph: &GraphIr,
    action: &super::ir::GraphNode,
    page_path: &str,
    page_data: &Value,
) -> String {
    let submit_template = action.metadata.get("submit").cloned().unwrap_or_default();
    let redirect = action.metadata.get("redirect").cloned();
    let path_params = action_page_path_parameters(page_path, redirect.as_deref());
    let submit = if submit_template.contains('{') {
        substitute_path_template(&submit_template, &path_params).unwrap_or(submit_template)
    } else {
        submit_template.clone()
    };
    let label = action_label(action);
    let hidden = render_action_hidden_inputs(&submit_template, redirect.as_deref(), &path_params, page_data);
    let entity_inputs = route_input_entity(graph, "post", &submit_template)
        .as_deref()
        .filter(|type_name| !is_scalar_type(type_name))
        .map(|entity| {
            entity_fields(graph, entity)
                .iter()
                .map(|field| render_form_field(graph, field))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let inputs = if hidden.is_empty() {
        entity_inputs
    } else if entity_inputs.is_empty() {
        hidden
    } else {
        format!("{hidden}{entity_inputs}")
    };
    format!(
        r#"    <form method="post" action="{submit}" class="action-form">
{inputs}      <button type="submit">{label}</button>
    </form>"#
    )
}

fn action_page_path_parameters(
    page_path: &str,
    redirect: Option<&str>,
) -> BTreeMap<String, String> {
    redirect
        .and_then(|template| match_http_path(template, page_path))
        .unwrap_or_default()
}

fn template_param_names(template: &str) -> Vec<String> {
    template
        .split('/')
        .filter_map(|segment| {
            segment
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
                .map(String::from)
        })
        .collect()
}

fn render_action_hidden_inputs(
    submit_template: &str,
    redirect: Option<&str>,
    path_params: &BTreeMap<String, String>,
    page_data: &Value,
) -> String {
    let ok = page_data.get("ok");
    let mut names = template_param_names(submit_template);
    if let Some(redirect) = redirect {
        for name in template_param_names(redirect) {
            if !names.iter().any(|existing| existing == &name) {
                names.push(name);
            }
        }
    }
    names
        .into_iter()
        .filter_map(|name| {
            let value = path_params
                .get(&name)
                .cloned()
                .or_else(|| {
                    ok.and_then(|object| object.get(&name))
                        .and_then(|value| value.as_str().map(String::from))
                });
            value.map(|value| format!(
                r#"      <input type="hidden" name="{name}" value="{value}">"#
            ))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn action_label(action: &super::ir::GraphNode) -> String {
    action
        .metadata
        .get("path")
        .and_then(|path| path.rsplit('/').next())
        .filter(|segment| !segment.is_empty())
        .unwrap_or("submit")
        .into()
}

fn normalized_route_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                "{}"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn route_input_entity(graph: &GraphIr, method: &str, submit_path: &str) -> Option<String> {
    find_route_by_template(graph, method, submit_path)
        .and_then(|route| route.type_name.as_deref())
        .and_then(|signature| signature.split_once("->"))
        .map(|(input, _)| input.trim().to_string())
}

fn find_route_by_template<'a>(
    graph: &'a GraphIr,
    method: &str,
    template: &str,
) -> Option<&'a super::ir::GraphNode> {
    let normalized = normalized_route_path(template);
    graph.nodes.iter().find(|node| {
        node.kind == "route"
            && node
                .metadata
                .get("method")
                .is_some_and(|value| value.eq_ignore_ascii_case(method))
            && node
                .metadata
                .get("path")
                .is_some_and(|value| normalized_route_path(value) == normalized)
    })
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

fn render_items_table(
    graph: &GraphIr,
    page_path: &str,
    output_type: &str,
    data: &Value,
) -> Option<String> {
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
    let detail_template = detail_path_template_for_list(graph, page_path, &item_type);
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
                    let cell = if column == "id"
                        && detail_template.is_some()
                        && field_type(graph, &item_type, column).as_deref() == Some("uuid")
                    {
                        let template = detail_template.as_deref().unwrap_or("");
                        let href = substitute_path_template(
                            template,
                            &BTreeMap::from([("id".into(), value.clone())]),
                        )
                        .unwrap_or_else(|| template.replace("{id}", &value));
                        format!(r#"<a href="{href}">{value}</a>"#)
                    } else {
                        value
                    };
                    format!("        <td>{cell}</td>")
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

fn field_type(graph: &GraphIr, entity_name: &str, field_name: &str) -> Option<String> {
    graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == entity_name)
        .and_then(|entity| {
            children(graph, &entity.id, "field")
                .into_iter()
                .find(|field| field.name == field_name)
        })
        .and_then(|field| field.type_name.clone())
}

fn find_page<'a>(
    graph: &'a GraphIr,
    path: &str,
) -> Option<(&'a super::ir::GraphNode, BTreeMap<String, String>)> {
    let normalized = normalize_path(path);
    let pages = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "page")
        .collect::<Vec<_>>();
    for page in &pages {
        let pattern = page.metadata.get("path")?;
        if !pattern.contains('{') && normalize_path(pattern) == normalized {
            return Some((page, BTreeMap::new()));
        }
    }
    for page in &pages {
        let pattern = page.metadata.get("path")?;
        if let Some(parameters) = match_http_path(pattern, path) {
            return Some((page, parameters));
        }
    }
    None
}

fn bind_page_input(
    page: &super::ir::GraphNode,
    request_path: &str,
    explicit_input: Value,
) -> Result<Value, String> {
    let source = page
        .metadata
        .get("input_source")
        .map(String::as_str)
        .unwrap_or("body");
    if source == "body" {
        return Ok(explicit_input);
    }
    let name = page
        .metadata
        .get("input_name")
        .ok_or_else(|| "ui_page_binding_has_no_name".to_string())?;
    let pattern = page
        .metadata
        .get("path")
        .ok_or_else(|| "ui_page_has_no_path".to_string())?;
    let parameters = match_http_path(pattern, request_path)
        .ok_or_else(|| format!("ui_page_path_mismatch:{request_path}"))?;
    let raw = parameters
        .get(name)
        .ok_or_else(|| format!("missing_path_parameter:{name}"))?;
    let input_type = page
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "ui_page_has_no_input_type".to_string())?;
    parse_bound_scalar(input_type, raw)
}

fn detail_path_template_for_list(
    graph: &GraphIr,
    list_path: &str,
    item_type: &str,
) -> Option<String> {
    let legacy = format!("{}/detail", normalize_path(list_path));
    if graph.nodes.iter().any(|node| {
        node.kind == "page"
            && node
                .metadata
                .get("path")
                .is_some_and(|path| normalize_path(path) == legacy)
    }) {
        return Some(legacy);
    }
    graph
        .nodes
        .iter()
        .filter(|node| node.kind == "page")
        .find_map(|page| {
            let path = page.metadata.get("path")?;
            if !path.contains("{id}") {
                return None;
            }
            let output = page
                .type_name
                .as_deref()
                .and_then(|value| value.split_once("->").map(|(_, output)| output))?;
            if strip_result(output) != item_type {
                return None;
            }
            Some(path.clone())
        })
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

    const ACTION_UI: &str = r#"axl 4
app ActionUI

entity WorkflowConfirm
  nota: text optional

entity Preventivo
  id: uuid key
  stato: text required

flow DettaglioPreventivo unit -> Result<Preventivo>
  make p: Preventivo
    id = "preventivo-001"
    stato = "bozza"
  return p

flow CercaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "bozza"
  return p

flow InviaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "inviato"
  return p

api PreventivoApi
  post /preventivi/demo/{id}/invia uuid -> Result<Preventivo> = InviaPreventivo from path.id

ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
  action /preventivi/demo/invia POST /preventivi/demo/{id}/invia redirect /preventivi/{id}
"#;

    #[test]
    fn ui_manifest_lists_declared_actions() {
        let graph = compile_source(ACTION_UI).unwrap().graph;
        let manifest = ui_manifest(&graph);
        assert_eq!(
            manifest["uis"][0]["actions"][0]["submit"],
            "/preventivi/demo/{id}/invia"
        );
        assert_eq!(
            manifest["uis"][0]["actions"][0]["redirect"],
            "/preventivi/{id}"
        );
        assert_eq!(
            manifest["uis"][0]["pages"][0]["template"],
            "/preventivi/{id}"
        );
    }

    #[test]
    fn render_page_embeds_actions_on_matching_detail_page() {
        let graph = compile_source(ACTION_UI).unwrap().graph;
        let rendered = render_page(&graph, "/preventivi/preventivo-001", json!(null)).unwrap();
        assert!(rendered.html.contains("class=\"actions\""));
        assert!(rendered.html.contains(r#"action="/preventivi/demo/preventivo-001/invia""#));
        assert!(rendered.html.contains(r#"name="id" value="preventivo-001""#));
        assert!(rendered.html.contains(">invia</button>"));
    }

    const TEMPLATED_LIST_UI: &str = r#"axl 4
app TemplatedListUI

entity Preventivo
  id: uuid key
  stato: text required

entity PreventivoPage
  items: List<Preventivo> required
  total: int required

flow PaginaPreventivi unit -> Result<PreventivoPage>
  make p: Preventivo
    id = "preventivo-001"
    stato = "bozza"
  make page: PreventivoPage
    items = [p]
    total = 1
  return page

flow CercaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "bozza"
  return p

ui PreventivoScreen
  page /preventivi unit -> Result<PreventivoPage> = PaginaPreventivi
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
"#;

    #[test]
    fn render_list_links_templated_detail_paths() {
        let graph = compile_source(TEMPLATED_LIST_UI).unwrap().graph;
        let rendered = render_page(&graph, "/preventivi", json!(null)).unwrap();
        assert!(
            rendered
                .html
                .contains(r#"href="/preventivi/preventivo-001""#)
        );
    }

    #[test]
    fn render_templated_page_binds_path_input() {
        let graph = compile_source(TEMPLATED_LIST_UI).unwrap().graph;
        let rendered = render_page(&graph, "/preventivi/preventivo-001", json!(null)).unwrap();
        assert_eq!(rendered.data["ok"]["id"], "preventivo-001");
    }
}
