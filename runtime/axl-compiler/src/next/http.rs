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
    dispatch_with_authorization(graph, runtime, method, path, input, None)
}

pub fn dispatch_with_authorization(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    method: &str,
    path: &str,
    input: Value,
    authorization: Option<&str>,
) -> HttpResult {
    let method = method.to_ascii_lowercase();
    let route = graph.nodes.iter().find(|node| {
        node.kind == "route"
            && node.metadata.get("method") == Some(&method)
            && node.metadata.get("path").is_some_and(|value| value == path)
    });
    let Some(route) = route else {
        return HttpResult {
            status: 404,
            body: json!({ "error": "route_not_found" }),
        };
    };
    if let Some(result) = authorize_request(graph, runtime, route, authorization) {
        return result;
    }
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
        Ok(mut runtime) => dispatch_with_authorization(
            &state.graph,
            &mut *runtime,
            method.as_str(),
            uri.path(),
            input,
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
        ),
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
flow Validate Input -> Result<Input>
  require input.amount > 0 else "amount_invalid"
  return input
capacity HttpAuth
  op authorize text -> Result<bool> idempotent
skill DemoBearer provides HttpAuth
  native rust axl::auth::bearer
  config token: text = "secret"
api DemoApi
  post /validate Input -> Result<Input> = Validate
api SecureApi
  auth bearer: HttpAuth = DemoBearer
  post /secure Input -> Result<Input> = Validate
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
    }
}
