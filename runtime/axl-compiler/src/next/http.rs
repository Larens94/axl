use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde_json::{Value, json};

use super::ir::GraphIr;
use super::runtime;

#[derive(Debug, Clone, PartialEq)]
pub struct HttpResult {
    pub status: u16,
    pub body: Value,
}

#[derive(Clone)]
struct HttpState {
    graph: Arc<GraphIr>,
}

pub fn dispatch(graph: &GraphIr, method: &str, path: &str, input: Value) -> HttpResult {
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
    let Some(flow) = route.metadata.get("flow") else {
        return HttpResult {
            status: 500,
            body: json!({ "error": "route_has_no_flow" }),
        };
    };
    match runtime::evaluate_flow(graph, flow, input) {
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

pub async fn serve(graph: GraphIr, address: &str) -> anyhow::Result<()> {
    let address: SocketAddr = address.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let local = listener.local_addr()?;
    println!("AXL HTTP listening on http://{local}");
    let app = Router::new().fallback(handle).with_state(HttpState {
        graph: Arc::new(graph),
    });
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle(State(state): State<HttpState>, method: Method, uri: Uri, body: Bytes) -> Response {
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
    let result = dispatch(&state.graph, method.as_str(), uri.path(), input);
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
api DemoApi
  post /validate Input -> Result<Input> = Validate
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
    }
}
