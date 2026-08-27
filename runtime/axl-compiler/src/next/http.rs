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

#[derive(Debug, Clone, PartialEq)]
pub struct HttpResult {
    pub status: u16,
    pub body: Value,
}

#[derive(Clone)]
struct HttpState {
    graph: Arc<GraphIr>,
    runtime: Arc<Mutex<BuiltinRuntime>>,
}

pub fn dispatch(graph: &GraphIr, method: &str, path: &str, input: Value) -> HttpResult {
    let Ok(mut runtime) = BuiltinRuntime::new() else {
        return HttpResult {
            status: 500,
            body: json!({ "error": "provider_runtime_initialization_failed" }),
        };
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
    let mut candidates = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "route" && node.metadata.get("method") == Some(&method));
    let matched = candidates
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
        });
    let Some((route, path_parameters)) = matched else {
        return HttpResult {
            status: 404,
            body: json!({ "error": "route_not_found" }),
        };
    };
    if let Some(result) =
        apply_request_middleware(graph, runtime, route, &method, request_path, headers)
    {
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
    let input = match bind_request_input(graph, route, input, &path_parameters, query) {
        Ok(input) => input,
        Err(message) => {
            return HttpResult {
                status: 400,
                body: json!({ "error": message }),
            };
        }
    };
    let Some(flow) = route.metadata.get("flow") else {
        return HttpResult {
            status: 500,
            body: json!({ "error": "route_has_no_flow" }),
        };
    };
    match runtime::evaluate_flow_with_runtime(graph, flow, input, runtime) {
        Ok(body) => HttpResult {
            status: if body.get("error").is_some() {
                422
            } else {
                200
            },
            body,
        },
        Err(error) => HttpResult {
            status: 400,
            body: json!({ "error": error.to_string() }),
        },
    }
}

fn match_http_path(pattern: &str, request: &str) -> Option<BTreeMap<String, String>> {
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

fn bind_request_input(
    graph: &GraphIr,
    route: &super::ir::GraphNode,
    body: Value,
    path: &BTreeMap<String, String>,
    query: &str,
) -> Result<Value, String> {
    let source = route
        .metadata
        .get("input_source")
        .map(String::as_str)
        .unwrap_or("body");
    if source == "composite" {
        return bind_composite_input(graph, route, body, path, query);
    }
    if source == "body" {
        return Ok(body);
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

fn bind_composite_input(
    graph: &GraphIr,
    route: &super::ir::GraphNode,
    body: Value,
    path: &BTreeMap<String, String>,
    query: &str,
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
        let optional = field
            .metadata
            .get("qualifiers")
            .is_some_and(|values| values.split(',').any(|value| value == "optional"))
            || field_type.starts_with("Option<");
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

fn query_value(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(key).as_deref() == Some(name)).then(|| percent_decode(value))?
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

fn parse_bound_scalar(type_name: &str, value: &str) -> Result<Value, String> {
    match type_name {
        "bool" => value
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|_| format!("invalid_bool:{value}")),
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
    let mut middlewares = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == api)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == "middleware")
        .collect::<Vec<_>>();
    if middlewares.is_empty() {
        return None;
    }
    middlewares.sort_by_key(|middleware| {
        middleware
            .metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
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
            return Some(HttpResult {
                status: 500,
                body: json!({ "error": "middleware_provider_missing" }),
            });
        };
        let Some(implementation) = provider.implementation.as_deref() else {
            return Some(HttpResult {
                status: 500,
                body: json!({ "error": "middleware_provider_has_no_binding" }),
            });
        };
        let config = match runtime::provider_config(graph, &provider.id) {
            Ok(config) => config,
            Err(error) => {
                return Some(HttpResult {
                    status: 500,
                    body: json!({ "error": error.to_string() }),
                });
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
            Ok(Value::Object(object)) if object.contains_key("method") => {
                envelope = Value::Object(object);
            }
            Ok(Value::Object(object)) if object.get("error").is_some() => {
                return Some(HttpResult {
                    status: 403,
                    body: Value::Object(object),
                });
            }
            Ok(_) => {
                return Some(HttpResult {
                    status: 403,
                    body: json!({ "error": "middleware_rejected" }),
                });
            }
            Err(error) => {
                return Some(HttpResult {
                    status: 403,
                    body: json!({ "error": error }),
                });
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
    let unauthorized = || HttpResult {
        status: 401,
        body: json!({ "error": "authorization_required" }),
    };
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
        return Some(HttpResult {
            status: 500,
            body: json!({ "error": "auth_provider_missing" }),
        });
    };
    let Some(implementation) = provider.implementation.as_deref() else {
        return Some(HttpResult {
            status: 500,
            body: json!({ "error": "auth_provider_has_no_binding" }),
        });
    };
    let config = match runtime::provider_config(graph, &provider.id) {
        Ok(config) => config,
        Err(error) => {
            return Some(HttpResult {
                status: 500,
                body: json!({ "error": error.to_string() }),
            });
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
        Ok(Value::Bool(false)) => Some(HttpResult {
            status: 403,
            body: json!({ "error": "authorization_denied" }),
        }),
        Ok(_) | Err(_) => Some(HttpResult {
            status: 403,
            body: json!({ "error": "authorization_failed" }),
        }),
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
    let input = if body.is_empty() {
        Value::Null
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
    };
    let result = match state.runtime.lock() {
        Ok(mut runtime) => {
            let headers = headers
                .iter()
                .filter_map(|(name, value)| {
                    Some((
                        name.as_str().to_ascii_lowercase(),
                        value.to_str().ok()?.to_string(),
                    ))
                })
                .collect::<BTreeMap<_, _>>();
            dispatch_with_headers(
                &state.graph,
                &mut *runtime,
                method.as_str(),
                uri.path_and_query()
                    .map(|value| value.as_str())
                    .unwrap_or_else(|| uri.path()),
                input,
                &headers,
            )
        }
        Err(_) => HttpResult {
            status: 500,
            body: json!({ "error": "provider_runtime_unavailable" }),
        },
    };
    let status = StatusCode::from_u16(result.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(result.body)).into_response()
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
entity HttpRequest
  method: text required
  path: text required
  headers: Map<text,text> required
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
skill DemoBearer provides HttpAuth
  native rust axl::auth::bearer
  config token: text = "secret"
skill DemoClientGate provides HttpMiddleware
  native rust axl::middleware::header_gate
  config header: text = "x-axl-client"
  config value: text = "demo"
api DemoApi
  post /validate Input -> Result<Input> = Validate
  get /items/{id} text -> text = EchoText from path.id
  get /search text -> text = EchoText from query.term
  put /items/{id} CompositeInput -> CompositeInput = EchoComposite
    bind id = path.id
    bind term = query.term
    bind amount = body.amount
    bind verbose = query.verbose
api SecureApi
  auth bearer: HttpAuth = DemoBearer
  post /secure Input -> Result<Input> = Validate
api GuardedApi
  middleware request: HttpMiddleware = DemoClientGate
  post /guarded Input -> Result<Input> = Validate
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
        let composite = dispatch(
            &graph,
            "put",
            "/items/item-1?term=ledger&verbose=true",
            json!({"amount": 25}),
        );
        assert_eq!(composite.status, 200);
        assert_eq!(
            composite.body,
            json!({"id": "item-1", "term": "ledger", "amount": 25, "verbose": true})
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
    }
}
