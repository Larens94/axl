use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde_json::{Value, json};

use super::ir::GraphIr;
use super::runtime;
use super::runtime::{BuiltinRuntime, ProviderCall, ProviderRuntime};
use super::ui;

#[derive(Debug, Clone, PartialEq)]
pub struct HttpResult {
    pub status: u16,
    pub body: Value,
    pub headers: BTreeMap<String, String>,
}

impl HttpResult {
    fn new(status: u16, body: Value) -> Self {
        Self {
            status,
            body,
            headers: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
struct HttpState {
    graph: Arc<GraphIr>,
    runtime: Arc<Mutex<BuiltinRuntime>>,
}

pub fn dispatch(graph: &GraphIr, method: &str, path: &str, input: Value) -> HttpResult {
    let Ok(mut runtime) = BuiltinRuntime::new() else {
        return HttpResult::new(
            500,
            json!({ "error": "provider_runtime_initialization_failed" }),
        );
    };
    dispatch_with_runtime(graph, &mut runtime, method, path, input)
}

pub fn dispatch_with_runtime(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    method: &str,
    path: &str,
    input: Value,
) -> HttpResult {
    dispatch_with_headers(graph, runtime, method, path, input, &BTreeMap::new())
}

pub fn dispatch_with_authorization(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    method: &str,
    path: &str,
    input: Value,
    authorization: Option<&str>,
) -> HttpResult {
    let mut headers = BTreeMap::new();
    if let Some(authorization) = authorization {
        headers.insert("authorization".into(), authorization.into());
    }
    dispatch_with_headers(graph, runtime, method, path, input, &headers)
}

pub fn dispatch_with_headers(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    method: &str,
    path: &str,
    input: Value,
    headers: &BTreeMap<String, String>,
) -> HttpResult {
    let method = method.to_ascii_lowercase();
    let (request_path, query) = path.split_once('?').unwrap_or((path, ""));
    let api_match = if method == "options" {
        None
    } else {
        match_http_route(graph, &method, request_path)
    };
    let exact_ui = ui::matches_exact_ui_path(graph, request_path);
    if method == "get"
        && (exact_ui || accepts_html(headers))
        && let Some(result) = dispatch_ui_get(graph, runtime, request_path, headers)
    {
        return result;
    }
    if method == "options" {
        return dispatch_cors_preflight(graph, runtime, request_path, headers)
            .unwrap_or_else(|| HttpResult::new(404, json!({ "error": "route_not_found" })));
    }
    let Some((route, mut path_parameters)) = api_match else {
        return HttpResult::new(404, json!({ "error": "route_not_found" }));
    };
    if input.is_object()
        && let Some(route_path) = route.metadata.get("path")
    {
        prefer_form_path_parameters(&mut path_parameters, route_path, &input);
    }
    if let Some(result) =
        apply_request_middleware(graph, runtime, route, &method, request_path, headers)
    {
        return result;
    }
    if let Some(result) = apply_route_guards(graph, runtime, route, request_path, query, headers) {
        return result;
    }
    if let Some(result) = authorize_request(
        graph,
        runtime,
        route,
        headers.get("authorization").map(String::as_str),
    ) {
        return result;
    }
    let input = match bind_request_input(graph, route, input, &path_parameters, query, headers) {
        Ok(input) => input,
        Err(message) => {
            return HttpResult::new(400, json!({ "error": message }));
        }
    };
    let Some(flow) = route.metadata.get("flow") else {
        return HttpResult::new(500, json!({ "error": "route_has_no_flow" }));
    };
    let mut result = match runtime::evaluate_flow_with_runtime(graph, flow, input, runtime) {
        Ok(body) => HttpResult::new(
            if body.get("error").is_some() {
                422
            } else {
                200
            },
            body,
        ),
        Err(error) => HttpResult::new(400, json!({ "error": error.to_string() })),
    };
    apply_response_middleware(graph, runtime, route, &mut result);
    if method == "post" {
        apply_form_post_redirect(graph, request_path, headers, &mut result);
    }
    result
}

fn match_http_route<'a>(
    graph: &'a GraphIr,
    method: &str,
    request_path: &str,
) -> Option<(&'a super::ir::GraphNode, BTreeMap<String, String>)> {
    let mut candidates = graph.nodes.iter().filter(|node| {
        node.kind == "route"
            && node
                .metadata
                .get("method")
                .is_some_and(|value| value == method)
    });
    candidates
        .clone()
        .find(|node| {
            node.metadata
                .get("path")
                .is_some_and(|value| value == request_path)
        })
        .map(|route| (route, BTreeMap::new()))
        .or_else(|| {
            candidates.find_map(|route| {
                match_http_path(route.metadata.get("path")?, request_path)
                    .map(|parameters| (route, parameters))
            })
        })
}

fn dispatch_ui_get(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    request_path: &str,
    headers: &BTreeMap<String, String>,
) -> Option<HttpResult> {
    if let Ok(rendered) = ui::render_form(graph, request_path) {
        let mut result = HttpResult::new(200, Value::String(rendered.html));
        result
            .headers
            .insert("content-type".into(), "text/html; charset=utf-8".into());
        return Some(result);
    }
    if let Ok(rendered) =
        ui::render_page_with_runtime(graph, runtime, request_path, Value::Null, headers)
    {
        let mut result = HttpResult::new(200, Value::String(rendered.html));
        result
            .headers
            .insert("content-type".into(), "text/html; charset=utf-8".into());
        return Some(result);
    }
    None
}

pub(crate) fn match_http_path(pattern: &str, request: &str) -> Option<BTreeMap<String, String>> {
    let pattern = pattern.split('/').collect::<Vec<_>>();
    let request = request.split('/').collect::<Vec<_>>();
    if pattern.len() != request.len() {
        return None;
    }
    let mut parameters = BTreeMap::new();
    for (pattern, value) in pattern.into_iter().zip(request) {
        if let Some(name) = pattern
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        {
            if value.is_empty() {
                return None;
            }
            parameters.insert(name.into(), percent_decode(value)?);
        } else if pattern != value {
            return None;
        }
    }
    Some(parameters)
}

fn dispatch_cors_preflight(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    request_path: &str,
    headers: &BTreeMap<String, String>,
) -> Option<HttpResult> {
    let route = find_route_any_method(graph, request_path)?;
    if !route_has_cors_middleware(graph, route) {
        return None;
    }
    if let Some(result) =
        apply_request_middleware(graph, runtime, route, "options", request_path, headers)
    {
        return Some(result);
    }
    let mut result = HttpResult::new(204, Value::Null);
    apply_response_middleware(graph, runtime, route, &mut result);
    Some(result)
}

fn find_route_any_method<'a>(
    graph: &'a GraphIr,
    request_path: &str,
) -> Option<&'a super::ir::GraphNode> {
    let mut candidates = graph.nodes.iter().filter(|node| node.kind == "route");
    candidates
        .clone()
        .find(|node| {
            node.metadata
                .get("path")
                .is_some_and(|value| value == request_path)
        })
        .or_else(|| {
            candidates.find(|node| {
                node.metadata
                    .get("path")
                    .and_then(|pattern| match_http_path(pattern, request_path))
                    .is_some()
            })
        })
}

fn route_has_cors_middleware(graph: &GraphIr, route: &super::ir::GraphNode) -> bool {
    let Some(api) = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "owns" && edge.to == route.id)
        .map(|edge| edge.from.as_str())
    else {
        return false;
    };
    for phase in ["request", "response"] {
        for middleware in api_middlewares(graph, api, phase) {
            let provider_id = graph
                .edges
                .iter()
                .find(|edge| edge.kind == "bind" && edge.from == middleware.id)
                .map(|edge| edge.to.as_str());
            let implementation = provider_id
                .and_then(|id| graph.nodes.iter().find(|node| node.id == id))
                .and_then(|node| node.implementation.as_deref());
            if implementation == Some("rust::axl::middleware::cors") {
                return true;
            }
        }
    }
    false
}

fn field_is_optional(field: &super::ir::GraphNode) -> bool {
    let field_type = field.type_name.as_deref().unwrap_or("unit");
    field.metadata.get("qualifiers").is_some_and(|values| {
        let qualifiers = values.split(',').collect::<Vec<_>>();
        qualifiers.contains(&"optional")
            || (qualifiers.contains(&"key") && !qualifiers.contains(&"required"))
    }) || field_type.starts_with("Option<")
}

fn bind_request_input(
    graph: &GraphIr,
    route: &super::ir::GraphNode,
    body: Value,
    path: &BTreeMap<String, String>,
    query: &str,
    headers: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let source = route
        .metadata
        .get("input_source")
        .map(String::as_str)
        .unwrap_or("body");
    if source == "composite" {
        return bind_composite_input(graph, route, body, path, query, headers);
    }
    if source == "body" {
        return coerce_body_input(graph, route, body);
    }
    let name = route
        .metadata
        .get("input_name")
        .ok_or_else(|| "request_binding_has_no_name".to_string())?;
    let raw = match source {
        "path" => path
            .get(name)
            .cloned()
            .ok_or_else(|| format!("missing_path_parameter:{name}"))?,
        "query" => {
            query_value(query, name).ok_or_else(|| format!("missing_query_parameter:{name}"))?
        }
        "header" => header_value(headers, name).ok_or_else(|| format!("missing_header:{name}"))?,
        "cookie" => cookie_value(headers, name).ok_or_else(|| format!("missing_cookie:{name}"))?,
        _ => return Err(format!("unsupported_request_source:{source}")),
    };
    let input_type = route
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "route_has_no_input_type".to_string())?;
    parse_bound_scalar(input_type, &raw)
}

pub(crate) fn bind_composite_input(
    graph: &GraphIr,
    route: &super::ir::GraphNode,
    body: Value,
    path: &BTreeMap<String, String>,
    query: &str,
    headers: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let input_type = route
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "route_has_no_input_type".to_string())?;
    let entity = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == input_type)
        .ok_or_else(|| format!("composite_input_is_not_entity:{input_type}"))?;
    let mut bindings = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == route.id)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == "request_binding")
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| {
        binding
            .metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    let fields = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == entity.id)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == "field")
        .collect::<Vec<_>>();
    let mut result = serde_json::Map::new();
    for binding in bindings {
        let target = binding.name.as_str();
        let field = fields
            .iter()
            .find(|field| field.name == target)
            .ok_or_else(|| format!("unknown_composite_field:{target}"))?;
        let field_type = field.type_name.as_deref().unwrap_or("unit");
        let optional = field_is_optional(field);
        let source = binding
            .metadata
            .get("source")
            .map(String::as_str)
            .unwrap_or("body");
        let name = binding.metadata.get("name").map(String::as_str);
        let value = match source {
            "body" if name.is_none() => Some(body.clone()),
            "body" => body
                .as_object()
                .and_then(|object| object.get(name.unwrap_or_default()))
                .cloned(),
            "path" => name
                .and_then(|name| path.get(name))
                .map(|value| parse_bound_scalar(field_type, value))
                .transpose()?,
            "query" => name
                .and_then(|name| query_value(query, name))
                .map(|value| parse_bound_scalar(field_type, &value))
                .transpose()?,
            "header" => name
                .and_then(|name| header_value(headers, name))
                .map(|value| parse_bound_scalar(field_type, &value))
                .transpose()?,
            "cookie" => name
                .and_then(|name| cookie_value(headers, name))
                .map(|value| parse_bound_scalar(field_type, &value))
                .transpose()?,
            _ => return Err(format!("unsupported_request_source:{source}")),
        };
        match value {
            Some(value) => {
                result.insert(target.into(), value);
            }
            None if optional => {}
            None => return Err(format!("missing_{source}_value:{target}")),
        }
    }
    Ok(Value::Object(result))
}

pub(crate) fn query_value(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(key).as_deref() == Some(name)).then(|| percent_decode(value))?
    })
}

fn header_value(headers: &BTreeMap<String, String>, name: &str) -> Option<String> {
    headers.get(&name.to_ascii_lowercase()).cloned()
}

pub(crate) fn cookie_value(headers: &BTreeMap<String, String>, name: &str) -> Option<String> {
    let cookie = headers.get("cookie")?;
    cookie.split(';').find_map(|part| {
        let part = part.trim();
        let (key, value) = part.split_once('=')?;
        (key.trim() == name).then(|| value.trim().to_string())
    })
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
                output.push(u8::from_str_radix(hex, 16).ok()?);
                index += 3;
            }
            b'%' => return None,
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            value => {
                output.push(value);
                index += 1;
            }
        }
    }
    String::from_utf8(output).ok()
}

fn coerce_body_input(
    graph: &GraphIr,
    route: &super::ir::GraphNode,
    body: Value,
) -> Result<Value, String> {
    let input_type = route
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .map(|value| value.0)
        .ok_or_else(|| "route_has_no_input_type".to_string())?;
    let Some(entity) = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == input_type)
    else {
        return Ok(body);
    };
    let Some(body_object) = body.as_object() else {
        return Ok(body);
    };
    let fields = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == entity.id)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == "field")
        .collect::<Vec<_>>();
    let mut result = serde_json::Map::new();
    for field in fields {
        let target = field.name.as_str();
        let field_type = field.type_name.as_deref().unwrap_or("unit");
        let optional = field_is_optional(field);
        match body_object.get(target) {
            Some(Value::String(raw)) => {
                result.insert(target.into(), parse_bound_scalar(field_type, raw)?);
            }
            Some(value) => {
                result.insert(target.into(), value.clone());
            }
            None if field_type == "bool" && !optional => {
                result.insert(target.into(), Value::Bool(false));
            }
            None if optional => {}
            None => return Err(format!("missing_body_field:{target}")),
        }
    }
    Ok(Value::Object(result))
}

fn is_form_urlencoded(content_type: Option<&str>) -> bool {
    content_type
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            value
                .trim()
                .eq_ignore_ascii_case("application/x-www-form-urlencoded")
        })
}

fn accepts_html(headers: &BTreeMap<String, String>) -> bool {
    headers.get("accept").is_some_and(|value| {
        value.split(',').any(|part| {
            part.split(';')
                .next()
                .is_some_and(|media| media.trim().eq_ignore_ascii_case("text/html"))
        })
    })
}

fn form_parent_path(form_path: &str) -> Option<String> {
    let trimmed = form_path.trim_end_matches('/');
    let (parent, _) = trimmed.rsplit_once('/')?;
    if parent.is_empty() {
        Some("/".into())
    } else {
        Some(parent.into())
    }
}

pub(crate) fn substitute_path_template(
    template: &str,
    parameters: &BTreeMap<String, String>,
) -> Option<String> {
    let mut resolved = template.to_string();
    for segment in template.split('/') {
        let Some(name) = segment
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        else {
            continue;
        };
        let value = parameters.get(name)?;
        resolved = resolved.replace(&format!("{{{name}}}"), value);
    }
    if resolved.contains('{') {
        return None;
    }
    Some(resolved)
}

pub(crate) fn substitute_path_from_value(template: &str, body: &Value) -> Option<String> {
    let ok = body.get("ok")?;
    let mut parameters = BTreeMap::new();
    for segment in template.split('/') {
        let Some(name) = segment
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        else {
            continue;
        };
        let value = ok.get(name)?.as_str()?.to_string();
        parameters.insert(name.into(), value);
    }
    substitute_path_template(template, &parameters)
}

pub(crate) fn path_template_matches(template: &str, request: &str) -> bool {
    if !template.contains('{') {
        return normalize_http_path(template) == normalize_http_path(request);
    }
    match_http_path(template, request).is_some()
}

fn normalize_http_path(path: &str) -> String {
    if path == "/" {
        return "/".into();
    }
    path.trim_end_matches('/').to_string()
}

fn submit_path_matches(template: &str, request_path: &str) -> bool {
    if template.contains('{') {
        path_template_matches(template, request_path)
    } else {
        normalize_http_path(template) == normalize_http_path(request_path)
    }
}

fn prefer_form_path_parameters(
    path_parameters: &mut BTreeMap<String, String>,
    route_path_template: &str,
    body: &Value,
) {
    if !body.is_object() {
        return;
    }
    for segment in route_path_template.split('/') {
        let Some(name) = segment
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        else {
            continue;
        };
        if let Some(Value::String(value)) = body.get(name) {
            path_parameters.insert(name.into(), value.clone());
        }
    }
}

fn form_redirect_location(graph: &GraphIr, submit_path: &str) -> Option<String> {
    if let Some(form) = graph.nodes.iter().find(|node| {
        node.kind == "form"
            && node
                .metadata
                .get("submit")
                .is_some_and(|value| submit_path_matches(value, submit_path))
    }) {
        if let Some(redirect) = form.metadata.get("redirect") {
            return Some(redirect.clone());
        }
        let form_path = form.metadata.get("path")?;
        return form_parent_path(form_path);
    }
    let action = graph.nodes.iter().find(|node| {
        node.kind == "ui_action"
            && node
                .metadata
                .get("submit")
                .is_some_and(|value| submit_path_matches(value, submit_path))
    })?;
    if let Some(redirect) = action.metadata.get("redirect") {
        return Some(redirect.clone());
    }
    action
        .metadata
        .get("path")
        .and_then(|path| form_parent_path(path))
}

fn form_clear_cookie(graph: &GraphIr, submit_path: &str) -> Option<String> {
    graph.nodes.iter().find_map(|node| {
        if node.kind != "ui_action" {
            return None;
        }
        let matches = node
            .metadata
            .get("submit")
            .is_some_and(|value| submit_path_matches(value, submit_path));
        if !matches {
            return None;
        }
        node.metadata.get("clear_cookie").cloned()
    })
}

fn apply_form_post_redirect(
    graph: &GraphIr,
    submit_path: &str,
    headers: &BTreeMap<String, String>,
    result: &mut HttpResult,
) {
    if result.status != 200 || result.body.get("error").is_some() {
        return;
    }
    let wants_redirect = is_form_urlencoded(headers.get("content-type").map(String::as_str))
        || accepts_html(headers);
    if !wants_redirect {
        return;
    }
    let Some(location) = form_redirect_location(graph, submit_path) else {
        return;
    };
    let location = substitute_path_from_value(&location, &result.body).unwrap_or(location);
    result.status = 303;
    result.headers.insert("location".into(), location);
    if let Some(session_id) = result
        .body
        .get("ok")
        .and_then(Value::as_object)
        .and_then(|object| object.get("session_id"))
        .and_then(Value::as_str)
    {
        result.headers.insert(
            "set-cookie".into(),
            format!("sid={session_id}; Path=/; HttpOnly; SameSite=Lax"),
        );
    }
    if let Some(cookie_name) = form_clear_cookie(graph, submit_path) {
        result.headers.insert(
            "set-cookie".into(),
            format!("{cookie_name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
        );
    }
}

fn parse_form_urlencoded(body: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(body)
        .map_err(|_| "invalid_form_encoding:body_is_not_utf8".to_string())?;
    let mut object = serde_json::Map::new();
    for pair in text.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = percent_decode(key).ok_or_else(|| format!("invalid_form_key:{key}"))?;
        let value = percent_decode(value).ok_or_else(|| format!("invalid_form_value:{value}"))?;
        object.insert(key, Value::String(value));
    }
    Ok(Value::Object(object))
}

pub(crate) fn parse_bound_scalar(type_name: &str, value: &str) -> Result<Value, String> {
    match type_name {
        "bool" => match value {
            "on" | "true" | "1" => Ok(Value::Bool(true)),
            "off" | "false" | "0" => Ok(Value::Bool(false)),
            other => other
                .parse::<bool>()
                .map(Value::Bool)
                .map_err(|_| format!("invalid_bool:{other}")),
        },
        "int" => value
            .parse::<i64>()
            .map(|value| json!(value))
            .map_err(|_| format!("invalid_int:{value}")),
        "float" | "money" => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| format!("invalid_number:{value}")),
        _ => Ok(Value::String(value.into())),
    }
}

fn apply_request_middleware(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    route: &super::ir::GraphNode,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
) -> Option<HttpResult> {
    let api = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "owns" && edge.to == route.id)
        .map(|edge| edge.from.as_str())?;
    let middlewares = api_middlewares(graph, api, "request");
    if middlewares.is_empty() {
        return None;
    }
    let mut envelope = json!({
        "method": method,
        "path": path,
        "headers": headers,
    });
    for middleware in middlewares {
        let provider_id = graph
            .edges
            .iter()
            .find(|edge| edge.kind == "bind" && edge.from == middleware.id)
            .map(|edge| edge.to.as_str());
        let provider = provider_id.and_then(|id| graph.nodes.iter().find(|node| node.id == id));
        let Some(provider) = provider else {
            return Some(HttpResult::new(
                500,
                json!({ "error": "middleware_provider_missing" }),
            ));
        };
        let Some(implementation) = provider.implementation.as_deref() else {
            return Some(HttpResult::new(
                500,
                json!({ "error": "middleware_provider_has_no_binding" }),
            ));
        };
        let config = match runtime::provider_config(graph, &provider.id) {
            Ok(config) => config,
            Err(error) => {
                return Some(HttpResult::new(500, json!({ "error": error.to_string() })));
            }
        };
        let capacity = middleware.type_name.as_deref().unwrap_or("");
        if capacity_has_allow(graph, capacity)
            && !capacity_has_operation(graph, capacity, "process")
        {
            let key = format!("{method} {path}");
            match runtime.invoke(ProviderCall {
                provider: &provider.name,
                capacity,
                implementation,
                operation: "allow",
                config,
                input: Value::String(key),
            }) {
                Ok(Value::Bool(true)) => {}
                Ok(Value::Bool(false)) => {
                    return Some(HttpResult::new(
                        429,
                        json!({ "error": "rate_limit_exceeded" }),
                    ));
                }
                Ok(Value::Object(object)) if object.get("error").is_some() => {
                    return Some(HttpResult::new(
                        middleware_reject_status(
                            object.get("error").and_then(Value::as_str).unwrap_or(""),
                        ),
                        Value::Object(object),
                    ));
                }
                Ok(_) => {
                    return Some(HttpResult::new(
                        429,
                        json!({ "error": "rate_limit_exceeded" }),
                    ));
                }
                Err(error) => {
                    return Some(HttpResult::new(
                        middleware_reject_status(&error),
                        json!({ "error": error }),
                    ));
                }
            }
            continue;
        }
        match runtime.invoke(ProviderCall {
            provider: &provider.name,
            capacity,
            implementation,
            operation: "process",
            config,
            input: envelope.clone(),
        }) {
            Ok(Value::Object(object)) if object.contains_key("method") => {
                envelope = Value::Object(object);
            }
            Ok(Value::Object(object)) if object.get("error").is_some() => {
                let status = middleware_reject_status(
                    object.get("error").and_then(Value::as_str).unwrap_or(""),
                );
                return Some(HttpResult::new(status, Value::Object(object)));
            }
            Ok(_) => {
                return Some(HttpResult::new(
                    403,
                    json!({ "error": "middleware_rejected" }),
                ));
            }
            Err(error) => {
                return Some(HttpResult::new(
                    middleware_reject_status(&error),
                    json!({ "error": error }),
                ));
            }
        }
    }
    None
}

fn capacity_has_allow(graph: &GraphIr, capacity: &str) -> bool {
    capacity_has_operation(graph, capacity, "allow")
}

fn capacity_has_operation(graph: &GraphIr, capacity: &str, operation: &str) -> bool {
    let capacity_id = format!("capacity.{capacity}");
    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == capacity_id)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .any(|node| node.kind == "operation" && node.name == operation)
}

fn middleware_reject_status(error: &str) -> u16 {
    if error == "rate_limit_exceeded" {
        429
    } else {
        403
    }
}

fn apply_response_middleware(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    route: &super::ir::GraphNode,
    result: &mut HttpResult,
) {
    let Some(api) = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "owns" && edge.to == route.id)
        .map(|edge| edge.from.as_str())
    else {
        return;
    };
    let middlewares = api_middlewares(graph, api, "response");
    if middlewares.is_empty() {
        return;
    }
    let body_text = match serde_json::to_string(&result.body) {
        Ok(text) => text,
        Err(error) => {
            *result = HttpResult::new(500, json!({ "error": error.to_string() }));
            return;
        }
    };
    let mut envelope = json!({
        "status": result.status,
        "headers": result.headers,
        "body": body_text,
    });
    for middleware in middlewares {
        let provider_id = graph
            .edges
            .iter()
            .find(|edge| edge.kind == "bind" && edge.from == middleware.id)
            .map(|edge| edge.to.as_str());
        let provider = provider_id.and_then(|id| graph.nodes.iter().find(|node| node.id == id));
        let Some(provider) = provider else {
            *result = HttpResult::new(500, json!({ "error": "middleware_provider_missing" }));
            return;
        };
        let Some(implementation) = provider.implementation.as_deref() else {
            *result = HttpResult::new(
                500,
                json!({ "error": "middleware_provider_has_no_binding" }),
            );
            return;
        };
        let config = match runtime::provider_config(graph, &provider.id) {
            Ok(config) => config,
            Err(error) => {
                *result = HttpResult::new(500, json!({ "error": error.to_string() }));
                return;
            }
        };
        match runtime.invoke(ProviderCall {
            provider: &provider.name,
            capacity: middleware.type_name.as_deref().unwrap_or(""),
            implementation,
            operation: "process",
            config,
            input: envelope.clone(),
        }) {
            Ok(Value::Object(object)) if object.contains_key("status") => {
                envelope = Value::Object(object);
            }
            Ok(Value::Object(object)) if object.get("error").is_some() => {
                *result = HttpResult::new(403, Value::Object(object));
                return;
            }
            Ok(_) => {
                *result = HttpResult::new(403, json!({ "error": "middleware_rejected" }));
                return;
            }
            Err(error) => {
                *result = HttpResult::new(403, json!({ "error": error }));
                return;
            }
        }
    }
    let status = envelope
        .get("status")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(result.status);
    let headers = envelope
        .get("headers")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value
                        .as_str()
                        .map(|text| (key.to_ascii_lowercase(), text.to_string()))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let body = match envelope.get("body").and_then(Value::as_str) {
        Some(text) => serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.into())),
        None => result.body.clone(),
    };
    *result = HttpResult {
        status,
        body,
        headers,
    };
}

fn api_middlewares<'a>(
    graph: &'a GraphIr,
    api: &str,
    phase: &str,
) -> Vec<&'a super::ir::GraphNode> {
    let mut middlewares = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == api)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| {
            node.kind == "middleware"
                && node.metadata.get("phase").map(String::as_str) == Some(phase)
        })
        .collect::<Vec<_>>();
    middlewares.sort_by_key(|middleware| {
        middleware
            .metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    middlewares
}

fn route_guards<'a>(
    graph: &'a GraphIr,
    route: &super::ir::GraphNode,
) -> Vec<&'a super::ir::GraphNode> {
    let mut guards = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == route.id)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == "route_guard")
        .collect::<Vec<_>>();
    guards.sort_by_key(|guard| {
        guard
            .metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    guards
}

fn apply_route_guards(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    route: &super::ir::GraphNode,
    request_path: &str,
    query: &str,
    headers: &BTreeMap<String, String>,
) -> Option<HttpResult> {
    let guards = route_guards(graph, route);
    if guards.is_empty() {
        return None;
    }
    let path_params = route
        .metadata
        .get("path")
        .and_then(|pattern| match_http_path(pattern, request_path))
        .unwrap_or_default();
    for guard in guards {
        let kind = guard.metadata.get("kind").map(String::as_str).unwrap_or("");
        let flow = guard.metadata.get("flow")?;
        let source = guard
            .metadata
            .get("source")
            .map(String::as_str)
            .unwrap_or("cookie");
        let name = guard.metadata.get("name").map(String::as_str);
        let raw = match source {
            "cookie" => name.and_then(|name| cookie_value(headers, name)),
            "header" => name.and_then(|name| header_value(headers, name)),
            "query" => name.and_then(|name| query_value(query, name)),
            "path" => name.and_then(|name| path_params.get(name).cloned()),
            _ => None,
        };
        match kind {
            "guest" => {
                let Some(raw) = raw else {
                    continue;
                };
                match runtime::evaluate_flow_with_runtime(graph, flow, Value::String(raw), runtime)
                {
                    Ok(body) if body.get("error").is_none() => {
                        return Some(HttpResult::new(
                            403,
                            json!({ "error": "already_authenticated" }),
                        ));
                    }
                    _ => continue,
                }
            }
            "session" => {
                let Some(raw) = raw else {
                    return Some(HttpResult::new(401, json!({ "error": "session_required" })));
                };
                match runtime::evaluate_flow_with_runtime(graph, flow, Value::String(raw), runtime)
                {
                    Ok(body) if body.get("error").is_none() => continue,
                    Ok(body) => {
                        return Some(HttpResult::new(
                            401,
                            json!({
                                "error": body.get("error").cloned().unwrap_or_else(|| json!("session_invalid"))
                            }),
                        ));
                    }
                    Err(error) => {
                        return Some(HttpResult::new(401, json!({ "error": error.to_string() })));
                    }
                }
            }
            "can" => {
                let Some(raw) = raw else {
                    return Some(HttpResult::new(401, json!({ "error": "session_required" })));
                };
                let permesso = guard.metadata.get("param").cloned().unwrap_or_default();
                let input = json!({
                    "session_id": raw,
                    "permesso": permesso,
                });
                match runtime::evaluate_flow_with_runtime(graph, flow, input, runtime) {
                    Ok(body) if body.get("error").is_none() => continue,
                    Ok(body) => {
                        return Some(HttpResult::new(
                            403,
                            json!({
                                "error": body.get("error").cloned().unwrap_or_else(|| json!("permesso_negato"))
                            }),
                        ));
                    }
                    Err(error) => {
                        return Some(HttpResult::new(403, json!({ "error": error.to_string() })));
                    }
                }
            }
            other => {
                return Some(HttpResult::new(
                    500,
                    json!({ "error": format!("unsupported_route_guard:{other}") }),
                ));
            }
        }
    }
    None
}

fn authorize_request(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    route: &super::ir::GraphNode,
    authorization: Option<&str>,
) -> Option<HttpResult> {
    let api = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "owns" && edge.to == route.id)
        .map(|edge| edge.from.as_str())?;
    let auth = graph.edges.iter().find_map(|edge| {
        (edge.kind == "owns" && edge.from == api)
            .then(|| graph.nodes.iter().find(|node| node.id == edge.to))
            .flatten()
            .filter(|node| node.kind == "auth")
    })?;
    let unauthorized = || HttpResult::new(401, json!({ "error": "authorization_required" }));
    let Some(header) = authorization else {
        return Some(unauthorized());
    };
    let Some((scheme, token)) = header.split_once(' ') else {
        return Some(unauthorized());
    };
    if !scheme.eq_ignore_ascii_case(&auth.name) || token.is_empty() {
        return Some(unauthorized());
    }
    let provider_id = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "bind" && edge.from == auth.id)
        .map(|edge| edge.to.as_str());
    let provider = provider_id.and_then(|id| graph.nodes.iter().find(|node| node.id == id));
    let Some(provider) = provider else {
        return Some(HttpResult::new(
            500,
            json!({ "error": "auth_provider_missing" }),
        ));
    };
    let Some(implementation) = provider.implementation.as_deref() else {
        return Some(HttpResult::new(
            500,
            json!({ "error": "auth_provider_has_no_binding" }),
        ));
    };
    let config = match runtime::provider_config(graph, &provider.id) {
        Ok(config) => config,
        Err(error) => {
            return Some(HttpResult::new(500, json!({ "error": error.to_string() })));
        }
    };
    match runtime.invoke(ProviderCall {
        provider: &provider.name,
        capacity: auth.type_name.as_deref().unwrap_or(""),
        implementation,
        operation: "authorize",
        config,
        input: Value::String(token.into()),
    }) {
        Ok(Value::Bool(true)) => None,
        Ok(Value::Bool(false)) => Some(HttpResult::new(
            403,
            json!({ "error": "authorization_denied" }),
        )),
        Ok(_) | Err(_) => Some(HttpResult::new(
            403,
            json!({ "error": "authorization_failed" }),
        )),
    }
}

pub async fn serve(graph: GraphIr, address: &str) -> anyhow::Result<()> {
    let address: SocketAddr = address.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let local = listener.local_addr()?;
    println!("AXL HTTP listening on http://{local}");
    let app = Router::new().fallback(handle).with_state(HttpState {
        graph: Arc::new(graph),
        runtime: Arc::new(Mutex::new(BuiltinRuntime::new()?)),
    });
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle(
    State(state): State<HttpState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let headers = headers
        .iter()
        .filter_map(|(name, value)| {
            Some((
                name.as_str().to_ascii_lowercase(),
                value.to_str().ok()?.to_string(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let input = if body.is_empty() {
        Value::Null
    } else {
        let content_type = headers.get("content-type").map(String::as_str);
        if is_form_urlencoded(content_type) {
            match parse_form_urlencoded(&body) {
                Ok(value) => value,
                Err(message) => {
                    return (StatusCode::BAD_REQUEST, Json(json!({ "error": message })))
                        .into_response();
                }
            }
        } else {
            match serde_json::from_slice(&body) {
                Ok(value) => value,
                Err(error) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": format!("invalid_json: {error}") })),
                    )
                        .into_response();
                }
            }
        }
    };
    let result = match state.runtime.lock() {
        Ok(mut runtime) => dispatch_with_headers(
            &state.graph,
            &mut *runtime,
            method.as_str(),
            uri.path_and_query()
                .map(|value| value.as_str())
                .unwrap_or_else(|| uri.path()),
            input,
            &headers,
        ),
        Err(_) => HttpResult::new(500, json!({ "error": "provider_runtime_unavailable" })),
    };
    let status = StatusCode::from_u16(result.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = if result.headers.get("content-type").map(String::as_str)
        == Some("text/html; charset=utf-8")
    {
        let html = result.body.as_str().unwrap_or("");
        (
            status,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            html.to_string(),
        )
            .into_response()
    } else {
        (status, Json(result.body)).into_response()
    };
    for (name, value) in result.headers {
        let Ok(name) = axum::http::HeaderName::try_from(name) else {
            continue;
        };
        let Ok(value) = axum::http::HeaderValue::try_from(value) else {
            continue;
        };
        response.headers_mut().insert(name, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::compile_source;

    const SOURCE: &str = r#"axl 4
app HttpDemo
entity Input
  amount: money required
entity CompositeInput
  id: text required
  term: text required
  amount: money required
  verbose: bool optional
  user: text optional
  sid: text optional
entity HttpRequest
  method: text required
  path: text required
  headers: Map<text,text> required
entity HttpResponse
  status: int required
  headers: Map<text,text> required
  body: text required
flow Validate Input -> Result<Input>
  require input.amount > 0 else "amount_invalid"
  return input
flow EchoText text -> text
  return input
flow EchoComposite CompositeInput -> CompositeInput
  return input
capacity HttpAuth
  op authorize text -> Result<bool> idempotent
capacity HttpMiddleware
  op process HttpRequest -> Result<HttpRequest> idempotent
capacity HttpResponseMiddleware
  op process HttpResponse -> Result<HttpResponse> idempotent
capacity RateLimit
  op allow text -> Result<bool> idempotent
skill DemoBearer provides HttpAuth
  native rust axl::auth::bearer
  config token: text = "secret"
skill DemoJwt provides HttpAuth
  native rust axl::auth::jwt
  config secret: text = "demo-only"
  config issuer: text = "axl-demo"
skill DemoClientGate provides HttpMiddleware
  native rust axl::middleware::header_gate
  config header: text = "x-axl-client"
  config value: text = "demo"
skill DemoResponseHeaders provides HttpResponseMiddleware
  native rust axl::middleware::response_headers
  config header: text = "x-axl-middleware"
  config value: text = "ok"
skill DemoRateLimit provides RateLimit
  native rust axl::middleware::rate_limit
  config limit: int = 2
  config window_ms: int = 60000
skill DemoCorsOrigin provides HttpMiddleware
  native rust axl::middleware::cors
  config origin: text = "https://app.example.com"
skill DemoCorsHeaders provides HttpResponseMiddleware
  native rust axl::middleware::cors
  config origin: text = "*"
  config methods: text = "GET,POST,OPTIONS"
  config headers: text = "content-type,authorization"
api DemoApi
  post /validate Input -> Result<Input> = Validate
  get /items/{id} text -> text = EchoText from path.id
  get /search text -> text = EchoText from query.term
  get /me text -> text = EchoText from header.x-user
  get /session text -> text = EchoText from cookie.sid
  put /items/{id} CompositeInput -> CompositeInput = EchoComposite
    bind id = path.id
    bind term = query.term
    bind amount = body.amount
    bind verbose = query.verbose
    bind user = header.x-user
    bind sid = cookie.sid
api SecureApi
  auth bearer: HttpAuth = DemoBearer
  post /secure Input -> Result<Input> = Validate
api JwtSecureApi
  auth bearer: HttpAuth = DemoJwt
  post /jwt Input -> Result<Input> = Validate
api GuardedApi
  middleware request: HttpMiddleware = DemoClientGate
  post /guarded Input -> Result<Input> = Validate
api AnnotatedApi
  middleware response: HttpResponseMiddleware = DemoResponseHeaders
  post /annotated Input -> Result<Input> = Validate
api LimitedApi
  middleware request: RateLimit = DemoRateLimit
  post /limited Input -> Result<Input> = Validate
api CorsApi
  middleware request: HttpMiddleware = DemoCorsOrigin
  middleware response: HttpResponseMiddleware = DemoCorsHeaders
  post /cors Input -> Result<Input> = Validate
"#;

    #[test]
    fn dispatches_graph_routes_to_flows() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let accepted = dispatch(&graph, "POST", "/validate", json!({"amount": 10}));
        assert_eq!(accepted.status, 200);
        assert_eq!(accepted.body["ok"]["amount"], 10);

        let rejected = dispatch(&graph, "post", "/validate", json!({"amount": 0}));
        assert_eq!(rejected.status, 422);
        assert_eq!(rejected.body["error"], "amount_invalid");

        assert_eq!(dispatch(&graph, "get", "/missing", Value::Null).status, 404);
        let from_path = dispatch(&graph, "get", "/items/hello%20world", Value::Null);
        assert_eq!(from_path.status, 200);
        assert_eq!(from_path.body, "hello world");
        let from_query = dispatch(&graph, "get", "/search?term=AXL%204", Value::Null);
        assert_eq!(from_query.status, 200);
        assert_eq!(from_query.body, "AXL 4");
        let missing_query = dispatch(&graph, "get", "/search", Value::Null);
        assert_eq!(missing_query.status, 400);
        assert_eq!(missing_query.body["error"], "missing_query_parameter:term");
        let missing_header = dispatch(&graph, "get", "/me", Value::Null);
        assert_eq!(missing_header.status, 400);
        assert_eq!(missing_header.body["error"], "missing_header:x-user");
        let mut header_map = BTreeMap::new();
        header_map.insert("x-user".into(), "alice".into());
        let from_header = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "get",
            "/me",
            Value::Null,
            &header_map,
        );
        assert_eq!(from_header.status, 200);
        assert_eq!(from_header.body, "alice");
        let missing_cookie = dispatch(&graph, "get", "/session", Value::Null);
        assert_eq!(missing_cookie.status, 400);
        assert_eq!(missing_cookie.body["error"], "missing_cookie:sid");
        header_map.insert("cookie".into(), "other=1; sid=session-42; keep=yes".into());
        let from_cookie = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "get",
            "/session",
            Value::Null,
            &header_map,
        );
        assert_eq!(from_cookie.status, 200);
        assert_eq!(from_cookie.body, "session-42");
        let composite = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "put",
            "/items/item-1?term=ledger&verbose=true",
            json!({"amount": 25}),
            &header_map,
        );
        assert_eq!(composite.status, 200);
        assert_eq!(
            composite.body,
            json!({
                "id": "item-1",
                "term": "ledger",
                "amount": 25,
                "verbose": true,
                "user": "alice",
                "sid": "session-42"
            })
        );

        let mut runtime = BuiltinRuntime::new().unwrap();
        let missing = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/secure",
            json!({"amount": 10}),
            None,
        );
        assert_eq!(missing.status, 401);
        let denied = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/secure",
            json!({"amount": 10}),
            Some("Bearer wrong"),
        );
        assert_eq!(denied.status, 403);
        let authorized = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/secure",
            json!({"amount": 10}),
            Some("Bearer secret"),
        );
        assert_eq!(authorized.status, 200);

        let jwt_missing = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/jwt",
            json!({"amount": 10}),
            None,
        );
        assert_eq!(jwt_missing.status, 401);
        let bad_jwt = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/jwt",
            json!({"amount": 10}),
            Some("Bearer not-a-jwt"),
        );
        assert_eq!(bad_jwt.status, 403);
        let wrong_issuer =
            runtime::encode_hs256_jwt("demo-only", &json!({"sub": "alice", "iss": "other"}))
                .unwrap();
        let denied_jwt = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/jwt",
            json!({"amount": 10}),
            Some(&format!("Bearer {wrong_issuer}")),
        );
        assert_eq!(denied_jwt.status, 403);
        let good_jwt =
            runtime::encode_hs256_jwt("demo-only", &json!({"sub": "alice", "iss": "axl-demo"}))
                .unwrap();
        let accepted_jwt = dispatch_with_authorization(
            &graph,
            &mut runtime,
            "post",
            "/jwt",
            json!({"amount": 10}),
            Some(&format!("Bearer {good_jwt}")),
        );
        assert_eq!(accepted_jwt.status, 200);

        let mut headers = BTreeMap::new();
        let missing_client = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/guarded",
            json!({"amount": 10}),
            &headers,
        );
        assert_eq!(missing_client.status, 403);
        headers.insert("x-axl-client".into(), "wrong".into());
        let denied_client = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/guarded",
            json!({"amount": 10}),
            &headers,
        );
        assert_eq!(denied_client.status, 403);
        headers.insert("x-axl-client".into(), "demo".into());
        let allowed_client = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/guarded",
            json!({"amount": 10}),
            &headers,
        );
        assert_eq!(allowed_client.status, 200);

        let annotated = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/annotated",
            json!({"amount": 10}),
            &BTreeMap::new(),
        );
        assert_eq!(annotated.status, 200);
        assert_eq!(
            annotated
                .headers
                .get("x-axl-middleware")
                .map(String::as_str),
            Some("ok")
        );

        let first = dispatch_with_runtime(
            &graph,
            &mut runtime,
            "post",
            "/limited",
            json!({"amount": 10}),
        );
        assert_eq!(first.status, 200);
        let second = dispatch_with_runtime(
            &graph,
            &mut runtime,
            "post",
            "/limited",
            json!({"amount": 10}),
        );
        assert_eq!(second.status, 200);
        let limited = dispatch_with_runtime(
            &graph,
            &mut runtime,
            "post",
            "/limited",
            json!({"amount": 10}),
        );
        assert_eq!(limited.status, 429);
        assert_eq!(limited.body, json!({ "error": "rate_limit_exceeded" }));

        let mut cors_headers = BTreeMap::new();
        cors_headers.insert("origin".into(), "https://evil.example.com".into());
        let rejected_origin = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/cors",
            json!({"amount": 10}),
            &cors_headers,
        );
        assert_eq!(rejected_origin.status, 403);
        assert_eq!(
            rejected_origin.body,
            json!({ "error": "cors_origin_rejected" })
        );
        cors_headers.insert("origin".into(), "https://app.example.com".into());
        let allowed_cors = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/cors",
            json!({"amount": 10}),
            &cors_headers,
        );
        assert_eq!(allowed_cors.status, 200);
        assert_eq!(
            allowed_cors
                .headers
                .get("access-control-allow-origin")
                .map(String::as_str),
            Some("*")
        );
        assert_eq!(
            allowed_cors
                .headers
                .get("access-control-allow-methods")
                .map(String::as_str),
            Some("GET,POST,OPTIONS")
        );
        let preflight = dispatch_with_headers(
            &graph,
            &mut runtime,
            "options",
            "/cors",
            Value::Null,
            &cors_headers,
        );
        assert_eq!(preflight.status, 204);
        assert_eq!(
            preflight
                .headers
                .get("access-control-allow-origin")
                .map(String::as_str),
            Some("*")
        );
        assert_eq!(
            preflight
                .headers
                .get("access-control-allow-methods")
                .map(String::as_str),
            Some("GET,POST,OPTIONS")
        );
        assert_eq!(
            dispatch_with_headers(
                &graph,
                &mut runtime,
                "options",
                "/validate",
                Value::Null,
                &BTreeMap::new(),
            )
            .status,
            404
        );
    }

    #[test]
    fn form_post_redirects_to_list_page_for_urlencoded_submit() {
        let graph = compile_source(
            r#"axl 4
app FormRedirectDemo
enum Stato
  attivo
entity Cliente
  nome: text required
  email: email required
  budget: money required
  stato: Stato required
flow CreaCliente Cliente -> Result<Cliente>
  return input
api ClienteApi
  post /clienti Cliente -> Result<Cliente> = CreaCliente
ui ClienteScreen
  page /clienti unit -> text = Echo
  form /clienti/new Cliente -> Result<Cliente> = CreaCliente submit /clienti
flow Echo unit -> text
  return "ok"
"#,
        )
        .unwrap()
        .graph;
        let mut headers = BTreeMap::new();
        headers.insert(
            "content-type".into(),
            "application/x-www-form-urlencoded".into(),
        );
        let result = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "post",
            "/clienti",
            json!({
                "nome": "Alice",
                "email": "alice@example.com",
                "budget": "1000",
                "stato": "attivo"
            }),
            &headers,
        );
        assert_eq!(result.status, 303);
        assert_eq!(
            result.headers.get("location").map(String::as_str),
            Some("/clienti")
        );

        let json_only = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "post",
            "/clienti",
            json!({
                "nome": "Bob",
                "email": "bob@example.com",
                "budget": 1000,
                "stato": "attivo"
            }),
            &BTreeMap::new(),
        );
        assert_eq!(json_only.status, 200);
        assert!(!json_only.headers.contains_key("location"));
    }

    #[test]
    fn action_post_redirects_to_templated_detail_page() {
        let graph = compile_source(
            r#"axl 4
app TemplatedRedirectDemo
entity Preventivo
  id: uuid key
  stato: text required
flow InviaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "inviato"
  return p
api PreventivoApi
  post /preventivi/{id}/invia uuid -> Result<Preventivo> = InviaPreventivo from path.id
ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
  action /preventivi/invia POST /preventivi/{id}/invia redirect /preventivi/{id}
flow CercaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "bozza"
  return p
"#,
        )
        .unwrap()
        .graph;
        let mut headers = BTreeMap::new();
        headers.insert(
            "content-type".into(),
            "application/x-www-form-urlencoded".into(),
        );
        headers.insert("accept".into(), "text/html".into());
        let result = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "post",
            "/preventivi/preventivo-001/invia",
            json!({ "id": "preventivo-001" }),
            &headers,
        );
        assert_eq!(result.status, 303);
        assert_eq!(
            result.headers.get("location").map(String::as_str),
            Some("/preventivi/preventivo-001")
        );
    }

    #[test]
    fn serve_get_dispatches_templated_ui_page() {
        let graph = compile_source(
            r#"axl 4
app TemplatedServeDemo
entity Preventivo
  id: uuid key
  stato: text required
flow CercaPreventivo uuid -> Result<Preventivo>
  make p: Preventivo
    id = input
    stato = "bozza"
  return p
ui PreventivoScreen
  page /preventivi/{id} uuid -> Result<Preventivo> = CercaPreventivo from path.id
"#,
        )
        .unwrap()
        .graph;
        let mut headers = BTreeMap::new();
        headers.insert("accept".into(), "text/html".into());
        let result = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "get",
            "/preventivi/preventivo-001",
            Value::Null,
            &headers,
        );
        assert_eq!(result.status, 200);
        assert!(
            result
                .body
                .as_str()
                .is_some_and(|html| html.contains("preventivo-001"))
        );
    }

    #[test]
    fn parses_form_urlencoded_fields() {
        let body = b"nome=Alice%20Rossi&email=alice%40example.com&budget=1000&stato=attivo";
        let parsed = parse_form_urlencoded(body).unwrap();
        assert_eq!(parsed["nome"], "Alice Rossi");
        assert_eq!(parsed["email"], "alice@example.com");
        assert_eq!(parsed["budget"], "1000");
        assert_eq!(parsed["stato"], "attivo");
    }

    #[test]
    fn dispatches_form_encoded_entity_body_to_flow() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let accepted = dispatch(&graph, "POST", "/validate", json!({ "amount": "10" }));
        assert_eq!(accepted.status, 200);
        assert_eq!(accepted.body["ok"]["amount"].as_f64(), Some(10.0));

        let rejected = dispatch(&graph, "post", "/validate", json!({ "amount": "0" }));
        assert_eq!(rejected.status, 422);
        assert_eq!(rejected.body["error"], "amount_invalid");
    }

    #[test]
    fn form_bool_checkbox_on_off_and_absent() {
        assert_eq!(parse_bound_scalar("bool", "on").unwrap(), Value::Bool(true));
        assert_eq!(
            parse_bound_scalar("bool", "off").unwrap(),
            Value::Bool(false)
        );
        let graph = compile_source(SOURCE).unwrap().graph;
        let without_verbose = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "put",
            "/items/item-1?term=ledger",
            json!({"amount": 25}),
            &BTreeMap::new(),
        );
        assert_eq!(without_verbose.status, 200);
        assert!(
            !without_verbose
                .body
                .as_object()
                .unwrap()
                .contains_key("verbose")
        );
    }

    #[test]
    fn route_guards_session_and_can_use_axl_flows() {
        let source = include_str!("../../../../examples/apps/route-guard-demo.axl");
        let graph = compile_source(source).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let missing = dispatch_with_runtime(&graph, &mut runtime, "post", "/echo", json!("hi"));
        assert_eq!(missing.status, 401);
        assert_eq!(missing.body["error"], "session_required");

        let seed = dispatch_with_runtime(&graph, &mut runtime, "post", "/seed", Value::Null);
        assert_eq!(seed.status, 200);
        let mut headers = BTreeMap::new();
        headers.insert("cookie".into(), "sid=sessione-demo".into());
        let ok =
            dispatch_with_headers(&graph, &mut runtime, "post", "/echo", json!("hi"), &headers);
        assert_eq!(ok.status, 200);
        assert_eq!(ok.body, "hi");

        let denied = dispatch_with_headers(
            &graph,
            &mut BuiltinRuntime::new().unwrap(),
            "post",
            "/secure",
            json!("hi"),
            &headers,
        );
        assert_eq!(denied.status, 401);

        let mut runtime = BuiltinRuntime::new().unwrap();
        let _ = dispatch_with_runtime(&graph, &mut runtime, "post", "/seed", Value::Null);
        let allowed = dispatch_with_headers(
            &graph,
            &mut runtime,
            "post",
            "/secure",
            json!("hi"),
            &headers,
        );
        assert_eq!(allowed.status, 200);
        assert_eq!(allowed.body, "hi");
    }
}
