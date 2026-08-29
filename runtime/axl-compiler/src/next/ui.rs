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
                        "layout": if matches!(path.as_str(), "/" | "/login" | "/register" | "/password-dimenticata" | "/reimposta-password") {
                            "guest"
                        } else if path.starts_with("/admin") {
                            "admin"
                        } else {
                            "app"
                        },
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
                        "on": action.metadata.get("on"),
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
        "theme": "dashboard-apple",
        "uis": uis,
    })
}

pub fn render_page(graph: &GraphIr, path: &str, input: Value) -> Result<UiRenderResult, String> {
    let mut runtime = runtime::BuiltinRuntime::new().map_err(|error| error.0)?;
    render_page_with_runtime(graph, &mut runtime, path, input, &BTreeMap::new())
}

pub fn render_page_with_runtime(
    graph: &GraphIr,
    provider_runtime: &mut dyn runtime::ProviderRuntime,
    path: &str,
    input: Value,
    headers: &BTreeMap<String, String>,
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
    let input = bind_page_input(graph, page, path, input, headers)?;
    let data = runtime::evaluate_flow_with_runtime(graph, &flow, input, provider_runtime)
        .map_err(|error| error.0)?;
    let sidebar = render_sidebar(graph, path);
    let html = render_page_html(graph, page, &graph.app, path, &output_type, &data, &sidebar);
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
    let sidebar = render_sidebar(graph, path);
    let html = render_form_html(graph, &graph.app, path, &entity, &submit, &sidebar);
    Ok(UiFormRenderResult {
        path: path.into(),
        entity,
        flow,
        submit,
        html,
    })
}

fn nav_group(path: &str) -> &'static str {
    let normalized = normalize_path(path);
    if normalized == "/" || normalized == "/home" {
        "Home"
    } else if normalized.starts_with("/admin") {
        "Amministrazione"
    } else if normalized.starts_with("/login")
        || normalized.starts_with("/register")
        || normalized.contains("password")
        || normalized.contains("reimposta")
    {
        "Accesso"
    } else {
        "Vendite"
    }
}

fn nav_label(path: &str) -> String {
    match normalize_path(path).as_str() {
        "/" => "Dashboard".into(),
        "/home" => "La mia home".into(),
        "/login" => "Accedi".into(),
        "/register" => "Registrati".into(),
        "/password-dimenticata" => "Password dimenticata".into(),
        "/reimposta-password" => "Reimposta password".into(),
        "/clienti" => "Clienti".into(),
        "/clienti/new" => "Nuovo cliente".into(),
        "/clienti/demo" => "Clienti demo".into(),
        "/preventivi" => "Preventivi".into(),
        "/preventivi/new" => "Nuovo preventivo".into(),
        "/preventivi/new-listino" => "Preventivo da listino".into(),
        "/preventivi/demo" => "Preventivi demo".into(),
        "/ordini" => "Ordini".into(),
        "/ordini/demo" => "Ordini demo".into(),
        "/prodotti" => "Prodotti".into(),
        "/prodotti/new" => "Nuovo prodotto".into(),
        "/prodotti/demo" => "Prodotti demo".into(),
        "/listini" => "Listini".into(),
        "/listini/new" => "Nuovo listino".into(),
        "/listini/demo" => "Listini demo".into(),
        "/admin/utenti" => "Utenti".into(),
        "/admin/ruoli" => "Ruoli".into(),
        "/admin/ruoli/new" => "Nuovo ruolo".into(),
        other => other.trim_start_matches('/').into(),
    }
}

fn nav_group_order(group: &str) -> u8 {
    match group {
        "Home" => 0,
        "Accesso" => 1,
        "Vendite" => 2,
        "Amministrazione" => 3,
        _ => 4,
    }
}

fn render_sidebar(graph: &GraphIr, current_path: &str) -> String {
    let current = normalize_path(current_path);
    let mut links = Vec::new();
    for node in &graph.nodes {
        if node.kind != "page" && node.kind != "form" {
            continue;
        }
        let Some(path) = node.metadata.get("path") else {
            continue;
        };
        if path.contains('{') || path.contains("/demo") {
            continue;
        }
        let normalized = normalize_path(path);
        let group = nav_group(path);
        let label = nav_label(path);
        let is_form = node.kind == "form";
        links.push((group, normalized, path.clone(), label, is_form));
    }
    links.sort_by(|left, right| {
        nav_group_order(left.0)
            .cmp(&nav_group_order(right.0))
            .then_with(|| left.2.cmp(&right.2))
    });
    if links.is_empty() {
        return String::new();
    }
    let mut sections = Vec::new();
    let mut current_group = "";
    let mut items = String::new();
    for (group, normalized, path, label, is_form) in links {
        if group != current_group {
            if !items.is_empty() {
                sections.push(format!(
                    r#"    <div class="nav-section">
      <span class="nav-label">{current_group}</span>
{items}    </div>"#
                ));
                items.clear();
            }
            current_group = group;
        }
        let active = if normalized == current { " active" } else { "" };
        let badge = if is_form {
            r#" <span class="badge">form</span>"#
        } else {
            ""
        };
        items.push_str(&format!(
            r#"      <a class="nav-link{active}" href="{path}">{label}{badge}</a>
"#
        ));
    }
    if !items.is_empty() {
        sections.push(format!(
            r#"    <div class="nav-section">
      <span class="nav-label">{current_group}</span>
{items}    </div>"#
        ));
    }
    format!(
        r#"  <aside class="sidebar">
    <div class="brand">
      <span class="brand-mark" aria-hidden="true"></span>
      <div class="brand-copy">
        <span class="brand-name">{app}</span>
        <span class="brand-tag">Portale</span>
      </div>
    </div>
    <nav class="sidebar-nav">
{sections}
    </nav>
    <form method="post" action="/auth/logout" class="logout-form">
      <button type="submit" class="btn btn-secondary btn-block">Esci</button>
    </form>
  </aside>"#,
        app = graph.app,
        sections = sections.join("\n")
    )
}

fn page_heading(path: &str) -> String {
    nav_label(path)
}

fn page_breadcrumb(path: &str) -> String {
    let group = nav_group(path);
    format!("{group} · {}", page_heading(path))
}

fn payload_object(data: &Value) -> Option<&serde_json::Map<String, Value>> {
    data.get("ok")
        .and_then(|value| value.as_object())
        .or_else(|| data.as_object())
}

fn render_home_dashboard(data: &Value) -> Option<String> {
    let map = payload_object(data)?;
    let titolo = map.get("titolo").and_then(|v| v.as_str())?;
    let messaggio = map.get("messaggio").and_then(|v| v.as_str()).unwrap_or("");
    let totale = map
        .get("totale_utenti")
        .map(display_value)
        .unwrap_or_else(|| "—".into());
    Some(format!(
        r#"  <section class="hero">
    <p class="eyebrow">Benvenuto</p>
    <h2 class="hero-title">{titolo}</h2>
    <p class="hero-subtitle">{messaggio}</p>
  </section>
  <section class="stat-grid">
    <article class="stat-card">
      <p class="stat-label">Utenti registrati</p>
      <p class="stat-value">{totale}</p>
      <p class="stat-hint">Totale nel sistema</p>
    </article>
  </section>
  <section class="quick-grid">
    <a class="quick-card" href="/clienti/demo"><span class="quick-label">Clienti</span><span class="quick-hint">Anagrafica e CRM</span></a>
    <a class="quick-card" href="/preventivi/demo"><span class="quick-label">Preventivi</span><span class="quick-hint">Offerte e workflow</span></a>
    <a class="quick-card" href="/ordini/demo"><span class="quick-label">Ordini</span><span class="quick-hint">Conferme e stati</span></a>
    <a class="quick-card" href="/login"><span class="quick-label">Accedi</span><span class="quick-hint">Sessione e permessi</span></a>
    <a class="quick-card" href="/admin/utenti"><span class="quick-label">Admin</span><span class="quick-hint">Utenti e ruoli</span></a>
  </section>"#
    ))
}

fn render_detail_card(fields: &[(String, String)]) -> String {
    let rows = fields
        .iter()
        .map(|(label, value)| format!("        <dt>{label}</dt>\n        <dd>{value}</dd>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"  <section class="card detail-card">
    <div class="card-header">
      <h2 class="card-title">Dettaglio</h2>
    </div>
    <dl class="detail-list">
{rows}
    </dl>
  </section>"#
    )
}

fn apply_ui_pagination_defaults(
    graph: &GraphIr,
    page: &super::ir::GraphNode,
    value: Value,
) -> Result<Value, String> {
    let Value::Object(map) = value else {
        return Ok(value);
    };
    let input_type = page
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "ui_page_has_no_input_type".to_string())?;
    let entity = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == input_type)
        .ok_or_else(|| format!("composite_input_is_not_entity:{input_type}"))?;
    let mut result = map.clone();
    for pagination in children(graph, &page.id, "ui_pagination") {
        let field = pagination.name.as_str();
        if result.contains_key(field) {
            continue;
        }
        let Some(default) = pagination.metadata.get("default") else {
            return Err(format!("missing_query_parameter:{field}"));
        };
        let field_type = children(graph, &entity.id, "field")
            .iter()
            .find(|candidate| candidate.name == field)
            .and_then(|candidate| candidate.type_name.as_deref())
            .unwrap_or("int");
        result.insert(
            field.into(),
            super::http::parse_bound_scalar(field_type, default)?,
        );
    }
    Ok(Value::Object(result))
}

fn merge_query_param(query: &str, key: &str, value: &str) -> String {
    let mut pairs = BTreeMap::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        if let Some((name, current)) = pair.split_once('=') {
            pairs.insert(name.to_string(), current.to_string());
        }
    }
    pairs.insert(key.to_string(), value.to_string());
    pairs
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn render_page_pagination(page_path: &str, request_path: &str, data: &Value) -> Option<String> {
    let payload = data.get("ok").unwrap_or(data);
    let Value::Object(map) = payload else {
        return None;
    };
    let total = map.get("total")?.as_i64()?;
    let limit = map.get("limit")?.as_i64()?;
    let offset = map.get("offset")?.as_i64()?;
    if limit <= 0 || total <= limit {
        return None;
    }
    let base_path = page_path.split('?').next().unwrap_or(page_path);
    let query = request_path
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or("");
    let current_page = offset / limit + 1;
    let total_pages = (total + limit - 1) / limit;
    let prev_offset = (offset - limit).max(0);
    let next_offset = offset + limit;
    let prev_query = merge_query_param(query, "offset", &prev_offset.to_string());
    let next_query = merge_query_param(query, "offset", &next_offset.to_string());
    let mut controls = Vec::new();
    if offset > 0 {
        controls.push(format!(
            r#"<a class="page-link" href="{base_path}?{prev_query}">Precedente</a>"#
        ));
    }
    controls.push(format!(
        r#"<span class="page-status">Pagina {current_page} di {total_pages}</span>"#
    ));
    if next_offset < total {
        controls.push(format!(
            r#"<a class="page-link" href="{base_path}?{next_query}">Successiva</a>"#
        ));
    }
    Some(format!(
        r#"  <nav class="pagination-bar" aria-label="Paginazione">
    {}
  </nav>"#,
        controls.join("\n    ")
    ))
}

fn render_page_filters(graph: &GraphIr, page: &super::ir::GraphNode, path: &str) -> Option<String> {
    let filters = children(graph, &page.id, "ui_filter");
    if filters.is_empty() {
        return None;
    }
    let page_path = page.metadata.get("path")?.split('?').next()?;
    let query = path.split_once('?').map(|(_, query)| query).unwrap_or("");
    let fields = filters
        .iter()
        .map(|filter| {
            let name = filter.name.clone();
            let current = super::http::query_value(query, &name).unwrap_or_default();
            format!(
                r#"        <label class="filter-field">{name} <input name="{name}" value="{current}" /></label>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!(
        r#"  <section class="card filter-card">
    <form class="filter-form" method="get" action="{page_path}">
{fields}
      <button type="submit">Filtra</button>
      <a class="filter-reset" href="{page_path}">Reset</a>
    </form>
  </section>"#
    ))
}

fn render_page_html(
    graph: &GraphIr,
    page: &super::ir::GraphNode,
    app: &str,
    path: &str,
    output_type: &str,
    data: &Value,
    sidebar: &str,
) -> String {
    let title = format!("{app}{path}");
    let heading = page_heading(path);
    let body = if strip_result(output_type) == "HomePage" {
        render_home_dashboard(data)
            .unwrap_or_else(|| render_detail_card(&collect_fields(graph, output_type, data)))
    } else if let Some(table) = render_items_table(graph, path, output_type, data) {
        let filters = render_page_filters(graph, page, path).unwrap_or_default();
        let pagination = render_page_pagination(path, path, data).unwrap_or_default();
        format!("{filters}{table}{pagination}")
    } else {
        let fields = collect_fields(graph, output_type, data);
        render_detail_card(&fields)
    };
    let actions = render_page_actions(graph, path, data);
    let content = format!("{body}{actions}");
    if is_guest_path(path) {
        return wrap_html_guest(app, path, &title, &heading, &content);
    }
    let body_class = if is_admin_path(path) {
        " admin-layout"
    } else {
        ""
    };
    wrap_html(app, path, &title, &heading, sidebar, &content, body_class)
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
                .get("on")
                .or_else(|| action.metadata.get("redirect"))
                .is_some_and(|target| path_template_matches(target, &normalized))
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
    format!(
        "\n  <section class=\"card actions-card\">\n    <div class=\"card-header\"><h2 class=\"card-title\">Azioni</h2></div>\n    <div class=\"actions\">\n{forms}\n    </div>\n  </section>"
    )
}

fn render_action_form(
    graph: &GraphIr,
    action: &super::ir::GraphNode,
    page_path: &str,
    page_data: &Value,
) -> String {
    let submit_template = action.metadata.get("submit").cloned().unwrap_or_default();
    let redirect = action.metadata.get("redirect").cloned();
    let on = action.metadata.get("on").map(String::as_str);
    let path_params = action_page_path_parameters(page_path, on, redirect.as_deref());
    let submit = if submit_template.contains('{') {
        substitute_path_template(&submit_template, &path_params)
            .unwrap_or_else(|| submit_template.clone())
    } else {
        submit_template.clone()
    };
    let label = action_label(action);
    let hidden = render_action_hidden_inputs(
        &submit_template,
        redirect.as_deref(),
        &path_params,
        page_data,
    );
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
        r#"      <form method="post" action="{submit}" class="action-form">
{inputs}        <button type="submit" class="btn btn-secondary">{label}</button>
      </form>"#
    )
}

fn action_page_path_parameters(
    page_path: &str,
    on: Option<&str>,
    redirect: Option<&str>,
) -> BTreeMap<String, String> {
    on.or(redirect)
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
            let value = path_params.get(&name).cloned().or_else(|| {
                ok.and_then(|object| object.get(&name))
                    .and_then(|value| value.as_str().map(String::from))
            });
            value.map(|value| {
                format!(r#"      <input type="hidden" name="{name}" value="{value}">"#)
            })
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
    sidebar: &str,
) -> String {
    let title = format!("{app}{path}");
    let heading = page_heading(path);
    let fields = entity_fields(graph, entity);
    let inputs = fields
        .iter()
        .map(|field| render_form_field(graph, field))
        .collect::<Vec<_>>()
        .join("\n");
    let body = format!(
        r#"  <section class="card form-card">
    <div class="card-header">
      <h2 class="card-title">Nuovo record</h2>
      <p class="card-subtitle">Invia i dati a <code>{submit}</code></p>
    </div>
    <form method="post" action="{submit}" class="stack-form">
{inputs}
      <div class="form-actions">
        <button type="submit" class="btn btn-primary">Salva</button>
      </div>
    </form>
  </section>"#
    );
    if is_guest_path(path) {
        return wrap_html_guest(app, path, &title, &heading, &body);
    }
    let body_class = if is_admin_path(path) {
        " admin-layout"
    } else {
        ""
    };
    wrap_html(app, path, &title, &heading, sidebar, &body, body_class)
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
    <select id="{name}" name="{name}" class="control"{required}>
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
    <input id="{name}" name="{name}" type="{input_type}" class="control"{required}>
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

fn dashboard_styles() -> &'static str {
    r#"
    :root {
      color-scheme: light dark;
      --font: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Segoe UI", sans-serif;
      --bg: #f5f5f7;
      --surface: rgba(255, 255, 255, 0.78);
      --surface-solid: #ffffff;
      --border: rgba(0, 0, 0, 0.08);
      --text: #1d1d1f;
      --muted: #6e6e73;
      --accent: #0071e3;
      --accent-hover: #0077ed;
      --shadow: 0 18px 50px rgba(0, 0, 0, 0.08);
      --radius: 18px;
      --sidebar-width: 17rem;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --bg: #000000;
        --surface: rgba(28, 28, 30, 0.82);
        --surface-solid: #1c1c1e;
        --border: rgba(255, 255, 255, 0.1);
        --text: #f5f5f7;
        --muted: #98989d;
        --shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
      }
    }
    * { box-sizing: border-box; }
    html, body { height: 100%; }
    body {
      margin: 0;
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
      -webkit-font-smoothing: antialiased;
    }
    a { color: var(--accent); text-decoration: none; }
    a:hover { text-decoration: underline; }
    .app-shell {
      min-height: 100vh;
      display: grid;
      grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
    }
    body.guest-layout {
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      padding: 1.5rem;
    }
    .guest-shell {
      width: min(100%, 28rem);
      background: var(--surface-solid);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow);
      padding: 1.5rem;
    }
    .guest-header { margin-bottom: 1.25rem; }
    .guest-brand { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); margin: 0 0 0.5rem; }
    .guest-title { font-size: 1.5rem; margin: 0; font-weight: 600; }
    .guest-path { font-size: 0.85rem; color: var(--muted); margin: 0.35rem 0 0; }
    body.admin-layout .topbar-title::after {
      content: " · Admin";
      color: var(--muted);
      font-weight: 500;
      font-size: 0.85em;
    }
    .sidebar {
      position: sticky;
      top: 0;
      height: 100vh;
      padding: 1.25rem 1rem;
      border-right: 1px solid var(--border);
      background: var(--surface);
      backdrop-filter: blur(24px) saturate(180%);
      -webkit-backdrop-filter: blur(24px) saturate(180%);
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.5rem 0.75rem 1.25rem;
    }
    .brand-mark {
      width: 2rem;
      height: 2rem;
      border-radius: 0.65rem;
      background: linear-gradient(145deg, #0071e3, #64d2ff);
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.35);
    }
    .brand-copy { display: flex; flex-direction: column; gap: 0.1rem; }
    .brand-name { font-weight: 700; letter-spacing: -0.02em; }
    .brand-tag { font-size: 0.75rem; color: var(--muted); }
    .sidebar-nav { display: flex; flex-direction: column; gap: 1.25rem; }
    .nav-section { display: flex; flex-direction: column; gap: 0.25rem; }
    .nav-label {
      padding: 0 0.75rem;
      font-size: 0.6875rem;
      font-weight: 600;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
    }
    .nav-link {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
      padding: 0.55rem 0.75rem;
      border-radius: 0.75rem;
      color: var(--text);
      text-decoration: none;
      font-size: 0.9375rem;
    }
    .nav-link:hover { background: rgba(0, 113, 227, 0.08); text-decoration: none; }
    .nav-link.active {
      background: rgba(0, 113, 227, 0.12);
      color: var(--accent);
      font-weight: 600;
    }
    .badge {
      font-size: 0.625rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      padding: 0.15rem 0.4rem;
      border-radius: 999px;
      background: rgba(0, 113, 227, 0.12);
      color: var(--accent);
    }
    .main {
      min-width: 0;
      display: flex;
      flex-direction: column;
    }
    .topbar {
      position: sticky;
      top: 0;
      z-index: 1;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
      padding: 1rem 2rem;
      border-bottom: 1px solid var(--border);
      background: rgba(245, 245, 247, 0.72);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
    }
    @media (prefers-color-scheme: dark) {
      .topbar { background: rgba(0, 0, 0, 0.72); }
    }
    .topbar-title {
      margin: 0;
      font-size: 1.75rem;
      font-weight: 700;
      letter-spacing: -0.03em;
    }
    .topbar-meta {
      margin: 0;
      font-size: 0.8125rem;
      color: var(--muted);
    }
    .content {
      padding: 1.5rem 2rem 2.5rem;
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
    }
    .hero {
      padding: 1.75rem 0 0.5rem;
    }
    .eyebrow {
      margin: 0 0 0.35rem;
      font-size: 0.8125rem;
      font-weight: 600;
      letter-spacing: 0.06em;
      text-transform: uppercase;
      color: var(--accent);
    }
    .hero-title {
      margin: 0;
      font-size: clamp(2rem, 4vw, 2.75rem);
      line-height: 1.05;
      letter-spacing: -0.04em;
    }
    .hero-subtitle {
      margin: 0.75rem 0 0;
      max-width: 42rem;
      font-size: 1.0625rem;
      color: var(--muted);
    }
    .stat-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
      gap: 1rem;
    }
    .stat-card, .card {
      background: var(--surface-solid);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow);
    }
    .stat-card { padding: 1.25rem 1.35rem; }
    .stat-label {
      margin: 0;
      font-size: 0.8125rem;
      color: var(--muted);
    }
    .stat-value {
      margin: 0.35rem 0 0;
      font-size: 2rem;
      font-weight: 700;
      letter-spacing: -0.03em;
    }
    .stat-hint {
      margin: 0.35rem 0 0;
      font-size: 0.8125rem;
      color: var(--muted);
    }
    .quick-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
      gap: 0.85rem;
    }
    .quick-card {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      padding: 1rem 1.1rem;
      border-radius: var(--radius);
      border: 1px solid var(--border);
      background: var(--surface-solid);
      box-shadow: var(--shadow);
      color: inherit;
      text-decoration: none;
      transition: transform 0.15s ease, box-shadow 0.15s ease;
    }
    .quick-card:hover {
      transform: translateY(-2px);
      text-decoration: none;
      box-shadow: 0 22px 55px rgba(0, 113, 227, 0.12);
    }
    .quick-label {
      font-weight: 650;
      letter-spacing: -0.02em;
    }
    .quick-hint {
      font-size: 0.8125rem;
      color: var(--muted);
    }
    .card-header {
      padding: 1.25rem 1.35rem 0;
    }
    .card-title {
      margin: 0;
      font-size: 1.125rem;
      font-weight: 650;
      letter-spacing: -0.02em;
    }
    .card-subtitle {
      margin: 0.35rem 0 0;
      font-size: 0.875rem;
      color: var(--muted);
    }
    .detail-card .detail-list,
    .table-card .table-wrap { padding: 1rem 1.35rem 1.35rem; }
    .detail-list {
      display: grid;
      grid-template-columns: minmax(9rem, 28%) 1fr;
      gap: 0.75rem 1.25rem;
      margin: 0;
    }
    .detail-list dt {
      margin: 0;
      font-size: 0.8125rem;
      font-weight: 600;
      color: var(--muted);
    }
    .detail-list dd { margin: 0; word-break: break-word; }
    .table-wrap { overflow-x: auto; }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.9375rem;
    }
    th, td {
      padding: 0.75rem 0.85rem;
      text-align: left;
      border-bottom: 1px solid var(--border);
    }
    th {
      font-size: 0.75rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--muted);
    }
    tbody tr:hover { background: rgba(0, 113, 227, 0.04); }
    .nested-table th, .nested-table td { padding: 0.5rem 0.65rem; }
    .empty-state {
      padding: 2.5rem 1.35rem;
      text-align: center;
      color: var(--muted);
    }
    .filter-card .filter-form, .pagination-bar {
      display: flex;
      flex-wrap: wrap;
      gap: 0.75rem;
      align-items: center;
      padding: 1rem 1.35rem 1.35rem;
    }
    .filter-field input {
      margin-left: 0.35rem;
      padding: 0.45rem 0.65rem;
      border: 1px solid var(--border);
      border-radius: 0.65rem;
      background: var(--surface-solid);
      color: var(--text);
      font: inherit;
    }
    .pagination-bar {
      justify-content: flex-end;
      border-top: 1px solid var(--border);
    }
    .page-link, .filter-reset {
      color: var(--accent);
      text-decoration: none;
      font-weight: 600;
    }
    .page-status { color: var(--muted); font-size: 0.875rem; }
    .stack-form { padding: 1rem 1.35rem 1.35rem; }
    .field { margin-bottom: 1rem; }
    .field label {
      display: block;
      margin-bottom: 0.35rem;
      font-size: 0.8125rem;
      font-weight: 600;
      color: var(--muted);
    }
    .control {
      width: 100%;
      padding: 0.65rem 0.85rem;
      border: 1px solid var(--border);
      border-radius: 0.75rem;
      background: var(--surface-solid);
      color: var(--text);
      font: inherit;
    }
    .control:focus {
      outline: none;
      border-color: rgba(0, 113, 227, 0.55);
      box-shadow: 0 0 0 4px rgba(0, 113, 227, 0.15);
    }
    .form-actions { margin-top: 0.5rem; }
    .actions { display: flex; flex-wrap: wrap; gap: 0.75rem; padding: 0 1.35rem 1.35rem; }
    .action-form { display: flex; flex-wrap: wrap; align-items: end; gap: 0.75rem; }
    .btn {
      appearance: none;
      border: none;
      border-radius: 999px;
      padding: 0.65rem 1.15rem;
      font: inherit;
      font-weight: 600;
      cursor: pointer;
    }
    .btn-primary {
      background: var(--accent);
      color: #fff;
    }
    .btn-primary:hover { background: var(--accent-hover); }
    .btn-secondary {
      background: rgba(0, 113, 227, 0.1);
      color: var(--accent);
    }
    .btn-secondary:hover { background: rgba(0, 113, 227, 0.16); }
    .btn-block { width: 100%; margin-top: 0.75rem; }
    .logout-form { padding: 0 0.75rem 1rem; }
    .footer-note {
      padding: 0 2rem 1.5rem;
      font-size: 0.75rem;
      color: var(--muted);
    }
    @media (max-width: 900px) {
      .app-shell { grid-template-columns: 1fr; }
      .sidebar {
        position: relative;
        height: auto;
        border-right: none;
        border-bottom: 1px solid var(--border);
      }
      .topbar, .content { padding-left: 1rem; padding-right: 1rem; }
    }
"#
}

fn is_guest_path(path: &str) -> bool {
    matches!(
        normalize_path(path).as_str(),
        "/" | "/login" | "/register" | "/password-dimenticata" | "/reimposta-password"
    )
}

fn is_admin_path(path: &str) -> bool {
    normalize_path(path).starts_with("/admin")
}

fn wrap_html_guest(
    app: &str,
    page_path: &str,
    document_title: &str,
    heading: &str,
    body: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="it">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{document_title}</title>
  <style>{styles}</style>
</head>
<body class="guest-layout">
  <main class="guest-shell">
    <header class="guest-header">
      <p class="guest-brand">{app}</p>
      <h1 class="guest-title">{heading}</h1>
      <p class="guest-path">{page_path}</p>
    </header>
    <div class="guest-body">
{body}
    </div>
    <p class="footer-note">AXL UI · axl-ui/1 · guest layout</p>
  </main>
</body>
</html>
"#,
        styles = dashboard_styles(),
    )
}

fn wrap_html(
    app: &str,
    page_path: &str,
    document_title: &str,
    heading: &str,
    sidebar: &str,
    body: &str,
    body_class: &str,
) -> String {
    let breadcrumb = page_breadcrumb(page_path);
    format!(
        r#"<!DOCTYPE html>
<html lang="it">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{document_title}</title>
  <style>{styles}</style>
</head>
<body class="{body_class}">
  <div class="app-shell">
{sidebar}
    <div class="main">
      <header class="topbar">
        <div>
          <p class="topbar-meta">{breadcrumb}</p>
          <h1 class="topbar-title">{heading}</h1>
        </div>
        <p class="topbar-meta">{app}</p>
      </header>
      <main class="content">
{body}
      </main>
      <p class="footer-note">AXL UI · axl-ui/1 · dashboard kit</p>
    </div>
  </div>
</body>
</html>
"#,
        styles = dashboard_styles(),
        body_class = body_class.trim(),
    )
}

fn render_entity_array_table(graph: &GraphIr, item_type: &str, items: &[Value]) -> Option<String> {
    let columns = entity_field_names(graph, item_type);
    if columns.is_empty() {
        return None;
    }
    Some(render_object_array_table(&columns, items))
}

fn render_object_array_table(columns: &[String], items: &[Value]) -> String {
    let header = columns
        .iter()
        .map(|column| format!("          <th>{column}</th>"))
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
                    format!("          <td>{value}</td>")
                })
                .collect::<Vec<_>>()
                .join("\n");
            Some(format!("        <tr>\n{cells}\n        </tr>"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"      <table class="nested-table">
        <thead>
          <tr>
{header}
          </tr>
        </thead>
        <tbody>
{rows}
        </tbody>
      </table>"#
    )
}

fn strip_list_type(type_name: &str) -> Option<&str> {
    type_name
        .strip_prefix("List<")
        .and_then(|value| value.strip_suffix('>'))
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
        return Some(
            r#"  <section class="card table-card">
    <div class="card-header"><h2 class="card-title">Elenco</h2></div>
    <p class="empty-state">Nessun elemento da mostrare.</p>
  </section>"#
                .into(),
        );
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
        r#"  <section class="card table-card">
    <div class="card-header"><h2 class="card-title">Elenco</h2></div>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
{header}
          </tr>
        </thead>
        <tbody>
{rows}
        </tbody>
      </table>
    </div>
  </section>"#
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
        .and_then(strip_list_type)
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
    let path_only = path.split('?').next().unwrap_or(path);
    let normalized = normalize_path(path_only);
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
        if let Some(parameters) = match_http_path(pattern, path_only) {
            return Some((page, parameters));
        }
    }
    None
}

fn bind_page_input(
    graph: &GraphIr,
    page: &super::ir::GraphNode,
    request_path: &str,
    explicit_input: Value,
    headers: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let source = page
        .metadata
        .get("input_source")
        .map(String::as_str)
        .unwrap_or("body");
    if source == "composite" {
        let (path_only, query) = request_path.split_once('?').unwrap_or((request_path, ""));
        let path_params = page
            .metadata
            .get("path")
            .and_then(|pattern| super::http::match_http_path(pattern, path_only))
            .unwrap_or_default();
        let bound = super::http::bind_composite_input(
            graph,
            page,
            explicit_input,
            &path_params,
            query,
            headers,
        )?;
        return apply_ui_pagination_defaults(graph, page, bound);
    }
    if source == "body" {
        return Ok(explicit_input);
    }
    let name = page
        .metadata
        .get("input_name")
        .ok_or_else(|| "ui_page_binding_has_no_name".to_string())?;
    let input_type = page
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "ui_page_has_no_input_type".to_string())?;
    let raw = match source {
        "path" => {
            let pattern = page
                .metadata
                .get("path")
                .ok_or_else(|| "ui_page_has_no_path".to_string())?;
            let parameters = match_http_path(pattern, request_path)
                .ok_or_else(|| format!("ui_page_path_mismatch:{request_path}"))?;
            parameters
                .get(name)
                .ok_or_else(|| format!("missing_path_parameter:{name}"))?
                .clone()
        }
        "query" => {
            let query = request_path
                .split_once('?')
                .map(|(_, query)| query)
                .unwrap_or("");
            super::http::query_value(query, name)
                .ok_or_else(|| format!("missing_query_parameter:{name}"))?
        }
        "header" => headers
            .get(name)
            .cloned()
            .ok_or_else(|| format!("missing_header:{name}"))?,
        "cookie" => super::http::cookie_value(headers, name)
            .ok_or_else(|| format!("missing_cookie:{name}"))?,
        other => return Err(format!("unsupported_ui_page_source:{other}")),
    };
    parse_bound_scalar(input_type, &raw)
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
                    let value = map.get(&field.name)?;
                    if let Value::Array(items) = value
                        && let Some(item_type) =
                            field.type_name.as_deref().and_then(strip_list_type)
                        && let Some(table) = render_entity_array_table(graph, item_type, items)
                    {
                        return Some((field.name.clone(), table));
                    }
                    Some((field.name.clone(), display_value(value)))
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
        assert!(rendered.html.contains("class=\"app-shell\""));
        assert!(rendered.html.contains("class=\"sidebar\""));
        assert!(rendered.html.contains("80000"));
        assert!(rendered.html.contains("<dt>money</dt>"));
    }

    #[test]
    fn render_page_emits_dashboard_shell() {
        let graph = compile_source(BALANCE_UI).unwrap().graph;
        let rendered = render_page(
            &graph,
            "/balance",
            json!({"income": 125000, "expense": 45000}),
        )
        .unwrap();
        assert!(rendered.html.contains("class=\"topbar\""));
        assert!(rendered.html.contains("dashboard kit"));
        assert!(rendered.html.contains("-apple-system"));
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
                .contains(r#"<form method="post" action="/clienti" class="stack-form">"#)
        );
        assert!(rendered.html.contains(r#"name="nome""#));
        assert!(
            rendered
                .html
                .contains(r#"<option value="attivo">attivo</option>"#)
        );
        assert!(
            rendered
                .html
                .contains(r#"class="btn btn-primary">Salva</button>"#)
        );
        assert!(rendered.html.contains("Clienti"));
        assert!(rendered.html.contains(r#"href="/clienti""#));
        assert!(rendered.html.contains("form-card"));
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
        assert!(
            rendered
                .html
                .contains(r#"action="/preventivi/demo/preventivo-001/invia""#)
        );
        assert!(
            rendered
                .html
                .contains(r#"name="id" value="preventivo-001""#)
        );
        assert!(rendered.html.contains(">invia</button>"));
    }

    const ON_ACTION_UI: &str = r#"axl 4
app OnActionUI

entity Preventivo
  id: uuid key
  stato: text required

flow CercaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "confermato"
  return p

flow CreaOrdine uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "confermato"
  return p

api OrdineApi
  post /ordini/da-preventivo/{id} uuid -> Result<Preventivo> = CreaOrdine from path.id

ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
  page /ordini/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
  action /preventivi/ordine POST /ordini/da-preventivo/{id} on /preventivi/{id} redirect /ordini/{id}
"#;

    #[test]
    fn render_page_embeds_actions_on_explicit_on_path() {
        let graph = compile_source(ON_ACTION_UI).unwrap().graph;
        let rendered = render_page(&graph, "/preventivi/preventivo-001", json!(null)).unwrap();
        assert!(rendered.html.contains("class=\"actions\""));
        assert!(
            rendered
                .html
                .contains(r#"action="/ordini/da-preventivo/preventivo-001""#)
        );
        let ordine_detail = render_page(&graph, "/ordini/preventivo-001", json!(null)).unwrap();
        assert!(!ordine_detail.html.contains("ordini/da-preventivo"));
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

    const NESTED_DETAIL_UI: &str = r#"axl 4
app NestedDetailUI

entity RigaPreventivo
  prodotto_id: text required
  quantita: int required
  prezzo_unitario: money required
  importo: money required

entity Preventivo
  id: uuid key
  stato: text required
  totale: money required
  righe: List<RigaPreventivo> required

flow DettaglioPreventivo unit -> Result<Preventivo>
  make r1: RigaPreventivo
    prodotto_id = "prodotto-001"
    quantita = 2
    prezzo_unitario = 129900
    importo = 259800
  make p: Preventivo
    id = "preventivo-001"
    stato = "bozza"
    totale = 259800
    righe = [r1]
  return p

ui PreventivoScreen
  page /preventivi unit -> Result<Preventivo> = DettaglioPreventivo
"#;

    #[test]
    fn render_detail_page_emits_nested_entity_list_table() {
        let graph = compile_source(NESTED_DETAIL_UI).unwrap().graph;
        let rendered = render_page(&graph, "/preventivi", json!(null)).unwrap();
        assert!(rendered.html.contains("<dt>righe</dt>"));
        assert!(rendered.html.contains("class=\"nested-table\""));
        assert!(rendered.html.contains("<th>prodotto_id</th>"));
        assert!(rendered.html.contains("<th>quantita</th>"));
        assert!(rendered.html.contains("prodotto-001"));
        assert!(!rendered.html.contains(r#"<dd>[{"#));
    }
}
