use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rusqlite::{Connection, params};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;
type MemoryStoreMap = BTreeMap<String, BTreeMap<String, Value>>;
type MemoryTxStack = Vec<(String, MemoryStoreMap)>;
type MemoryMigrationMap = BTreeMap<String, Vec<String>>;
type DocumentStoreMap = BTreeMap<String, BTreeMap<String, Value>>;

use super::expression;
use super::ir::{GraphIr, GraphNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError(pub String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RuntimeError {}

pub struct ProviderCall<'a> {
    pub provider: &'a str,
    pub capacity: &'a str,
    pub implementation: &'a str,
    pub operation: &'a str,
    pub config: BTreeMap<String, Value>,
    pub input: Value,
}

pub trait ProviderRuntime: Send {
    fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String>;

    fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
        Err("provider runtime does not support concurrent forks".into())
    }
}

#[derive(Clone)]
pub struct BuiltinRuntime {
    memory: Arc<Mutex<MemoryStoreMap>>,
    memory_tx: Arc<Mutex<MemoryTxStack>>,
    migrations: Arc<Mutex<MemoryMigrationMap>>,
    caches: Arc<Mutex<BTreeMap<String, BTreeMap<String, Value>>>>,
    event_logs: Arc<Mutex<BTreeMap<String, Vec<Value>>>>,
    loggers: Arc<Mutex<BTreeMap<String, Vec<Value>>>>,
    emails: Arc<Mutex<BTreeMap<String, Vec<Value>>>>,
    pdfs: Arc<Mutex<BTreeMap<String, Value>>>,
    metrics: Arc<Mutex<BTreeMap<String, BTreeMap<String, i64>>>>,
    tracers: Arc<Mutex<BTreeMap<String, TracerState>>>,
    rate_limits: Arc<Mutex<BTreeMap<String, BTreeMap<String, RateWindow>>>>,
    job_stores: Arc<Mutex<BTreeMap<String, BTreeMap<String, Value>>>>,
    sqlite: Arc<Mutex<BTreeMap<String, Arc<Mutex<Connection>>>>>,
    sqlite_tx: Arc<Mutex<BTreeMap<String, Vec<String>>>>,
    documents: Arc<Mutex<BTreeMap<String, Arc<Mutex<DocumentStoreMap>>>>>,
}

#[derive(Clone, Default)]
struct TracerState {
    next_id: u64,
    open: BTreeMap<String, String>,
    finished: Vec<Value>,
}

#[derive(Clone)]
struct RateWindow {
    started: Instant,
    count: u64,
}

impl BuiltinRuntime {
    pub fn new() -> Result<Self, RuntimeError> {
        Ok(Self {
            memory: Arc::new(Mutex::new(BTreeMap::new())),
            memory_tx: Arc::new(Mutex::new(Vec::new())),
            migrations: Arc::new(Mutex::new(BTreeMap::new())),
            caches: Arc::new(Mutex::new(BTreeMap::new())),
            event_logs: Arc::new(Mutex::new(BTreeMap::new())),
            loggers: Arc::new(Mutex::new(BTreeMap::new())),
            emails: Arc::new(Mutex::new(BTreeMap::new())),
            pdfs: Arc::new(Mutex::new(BTreeMap::new())),
            metrics: Arc::new(Mutex::new(BTreeMap::new())),
            tracers: Arc::new(Mutex::new(BTreeMap::new())),
            rate_limits: Arc::new(Mutex::new(BTreeMap::new())),
            job_stores: Arc::new(Mutex::new(BTreeMap::new())),
            sqlite: Arc::new(Mutex::new(BTreeMap::new())),
            sqlite_tx: Arc::new(Mutex::new(BTreeMap::new())),
            documents: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }

    fn sqlite_connection_key(call: &ProviderCall<'_>) -> String {
        call.config
            .get("path")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!(":memory:{}", call.provider))
    }

    fn document_store_key(call: &ProviderCall<'_>) -> String {
        Self::sqlite_connection_key(call)
    }

    fn document_store(
        &self,
        call: &ProviderCall<'_>,
    ) -> Result<Arc<Mutex<DocumentStoreMap>>, String> {
        let configured_path = call.config.get("path").and_then(Value::as_str);
        let key = Self::document_store_key(call);
        let mut stores = self
            .documents
            .lock()
            .map_err(|_| "document store registry is unavailable".to_string())?;
        if let Some(store) = stores.get(&key) {
            return Ok(store.clone());
        }
        let loaded = match configured_path {
            Some(":memory:") | None => DocumentStoreMap::new(),
            Some(path) => load_document_file(path)?,
        };
        let store = Arc::new(Mutex::new(loaded));
        stores.insert(key, store.clone());
        Ok(store)
    }

    fn sqlite_connection(&self, call: &ProviderCall<'_>) -> Result<Arc<Mutex<Connection>>, String> {
        let configured_path = call.config.get("path").and_then(Value::as_str);
        let key = Self::sqlite_connection_key(call);
        let mut connections = self
            .sqlite
            .lock()
            .map_err(|_| "SQLite connection registry is unavailable".to_string())?;
        if let Some(connection) = connections.get(&key) {
            return Ok(connection.clone());
        }
        let connection = match configured_path {
            Some(":memory:") | None => Connection::open_in_memory(),
            Some(path) => {
                if let Some(parent) = std::path::Path::new(path).parent()
                    && !parent.as_os_str().is_empty()
                {
                    std::fs::create_dir_all(parent)
                        .map_err(|error| format!("cannot create SQLite directory: {error}"))?;
                }
                Connection::open(path)
            }
        }
        .map_err(|error| format!("cannot initialize SQLite provider: {error}"))?;
        initialize_sqlite(&connection)?;
        let connection = Arc::new(Mutex::new(connection));
        connections.insert(key, connection.clone());
        Ok(connection)
    }
}

impl ProviderRuntime for BuiltinRuntime {
    fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
        match call.implementation {
            "rust::axl::store::memory" => {
                let mut memory = self
                    .memory
                    .lock()
                    .map_err(|_| "memory provider state is unavailable".to_string())?;
                memory_store_call(&mut memory, call)
            }
            "rust::axl::store::sqlite" => {
                let connection = self.sqlite_connection(&call)?;
                let sqlite = connection
                    .lock()
                    .map_err(|_| "SQLite provider state is unavailable".to_string())?;
                sqlite_store_call(&sqlite, call)
            }
            "rust::axl::store::document" => {
                let store = self.document_store(&call)?;
                let mut documents = store
                    .lock()
                    .map_err(|_| "document provider state is unavailable".to_string())?;
                let flush_path = call
                    .config
                    .get("path")
                    .and_then(Value::as_str)
                    .filter(|path| *path != ":memory:")
                    .map(str::to_string);
                document_store_call(&mut documents, call, flush_path.as_deref())
            }
            "rust::axl::tx::memory" => {
                let mut memory = self
                    .memory
                    .lock()
                    .map_err(|_| "memory provider state is unavailable".to_string())?;
                let mut stack = self
                    .memory_tx
                    .lock()
                    .map_err(|_| "memory transaction state is unavailable".to_string())?;
                memory_tx_call(&mut memory, &mut stack, call)
            }
            "rust::axl::tx::sqlite" => {
                let key = Self::sqlite_connection_key(&call);
                let connection = self.sqlite_connection(&call)?;
                let sqlite = connection
                    .lock()
                    .map_err(|_| "SQLite provider state is unavailable".to_string())?;
                let mut stack = self
                    .sqlite_tx
                    .lock()
                    .map_err(|_| "SQLite transaction state is unavailable".to_string())?;
                sqlite_tx_call(&sqlite, &mut stack, &key, call)
            }
            "rust::axl::migrate::memory" => {
                let mut migrations = self
                    .migrations
                    .lock()
                    .map_err(|_| "memory migration state is unavailable".to_string())?;
                memory_migrate_call(&mut migrations, call)
            }
            "rust::axl::migrate::sqlite" => {
                let connection = self.sqlite_connection(&call)?;
                let sqlite = connection
                    .lock()
                    .map_err(|_| "SQLite provider state is unavailable".to_string())?;
                sqlite_migrate_call(&sqlite, call)
            }
            "rust::axl::auth::bearer" => bearer_auth_call(call),
            "rust::axl::auth::jwt" => jwt_auth_call(call),
            "rust::axl::auth::jwt_sign" => jwt_sign_call(call),
            "rust::axl::auth::jwt_decode" => jwt_decode_call(call),
            "rust::axl::auth::password" => password_auth_call(call),
            "rust::axl::middleware::header_gate" => header_gate_call(call),
            "rust::axl::middleware::response_headers" => response_headers_call(call),
            "rust::axl::middleware::cors" => cors_call(call),
            "rust::axl::middleware::rate_limit" => {
                let mut rate_limits = self
                    .rate_limits
                    .lock()
                    .map_err(|_| "rate limit provider state is unavailable".to_string())?;
                rate_limit_call(&mut rate_limits, call)
            }
            "rust::axl::event::log" => {
                let mut logs = self
                    .event_logs
                    .lock()
                    .map_err(|_| "event log provider state is unavailable".to_string())?;
                event_log_call(&mut logs, call)
            }
            "rust::axl::job::memory" => {
                let mut stores = self
                    .job_stores
                    .lock()
                    .map_err(|_| "job store provider state is unavailable".to_string())?;
                memory_job_store_call(&mut stores, call)
            }
            "rust::axl::job::sqlite" => {
                let connection = self.sqlite_connection(&call)?;
                let sqlite = connection
                    .lock()
                    .map_err(|_| "SQLite provider state is unavailable".to_string())?;
                sqlite_job_store_call(&sqlite, call)
            }
            "rust::axl::cache::memory" => {
                let mut caches = self
                    .caches
                    .lock()
                    .map_err(|_| "cache provider state is unavailable".to_string())?;
                memory_cache_call(&mut caches, call)
            }
            "rust::axl::cache::sqlite" => {
                let connection = self.sqlite_connection(&call)?;
                let sqlite = connection
                    .lock()
                    .map_err(|_| "SQLite provider state is unavailable".to_string())?;
                sqlite_cache_call(&sqlite, call)
            }
            "rust::axl::telemetry::logger" => {
                let mut loggers = self
                    .loggers
                    .lock()
                    .map_err(|_| "logger provider state is unavailable".to_string())?;
                memory_logger_call(&mut loggers, call)
            }
            "rust::axl::email::memory" => {
                let mut emails = self
                    .emails
                    .lock()
                    .map_err(|_| "email provider state is unavailable".to_string())?;
                memory_email_call(&mut emails, call)
            }
            "rust::axl::pdf::memory" => {
                let mut pdfs = self
                    .pdfs
                    .lock()
                    .map_err(|_| "pdf provider state is unavailable".to_string())?;
                memory_pdf_call(&mut pdfs, call)
            }
            "rust::axl::telemetry::metrics" => {
                let mut metrics = self
                    .metrics
                    .lock()
                    .map_err(|_| "metrics provider state is unavailable".to_string())?;
                memory_metrics_call(&mut metrics, call)
            }
            "rust::axl::telemetry::tracer" => {
                let mut tracers = self
                    .tracers
                    .lock()
                    .map_err(|_| "tracer provider state is unavailable".to_string())?;
                memory_tracer_call(&mut tracers, call)
            }
            implementation => Err(format!(
                "unsupported provider implementation '{implementation}'"
            )),
        }
    }

    fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
        Ok(Box::new(self.clone()))
    }
}

pub fn evaluate_flow(
    graph: &GraphIr,
    flow_name: &str,
    input: Value,
) -> Result<Value, RuntimeError> {
    let mut runtime = BuiltinRuntime::new()?;
    evaluate_flow_with_runtime(graph, flow_name, input, &mut runtime)
}

pub fn evaluate_flow_with_runtime(
    graph: &GraphIr,
    flow_name: &str,
    input: Value,
    runtime: &mut dyn ProviderRuntime,
) -> Result<Value, RuntimeError> {
    evaluate_flow_inner(graph, flow_name, input, runtime, 0)
}

fn evaluate_flow_inner(
    graph: &GraphIr,
    flow_name: &str,
    input: Value,
    runtime: &mut dyn ProviderRuntime,
    depth: usize,
) -> Result<Value, RuntimeError> {
    if depth >= 64 {
        return Err(RuntimeError("flow call depth exceeds 64".into()));
    }
    let flow = graph
        .nodes
        .iter()
        .find(|node| node.kind == "flow" && node.name == flow_name)
        .ok_or_else(|| RuntimeError(format!("unknown flow '{flow_name}'")))?;
    let signature = flow
        .type_name
        .as_deref()
        .and_then(|value| value.split_once("->"))
        .ok_or_else(|| RuntimeError(format!("flow '{flow_name}' has no valid signature")))?;
    validate_value(graph, signature.0, &input, "input")?;

    let mut values = enum_values(graph);
    values.insert("input".into(), input);
    let statements = ordered_children(graph, &flow.id);
    for statement in statements {
        match statement.kind.as_str() {
            "let" => {
                let expression = statement_expression(statement)?;
                let value = expression::evaluate(&expression, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                values.insert(statement.name.clone(), value);
            }
            "require" => {
                let expression = statement_expression(statement)?;
                let value = expression::evaluate(&expression, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let accepted = value.as_bool().ok_or_else(|| {
                    RuntimeError(format!("{} did not evaluate to bool", statement.id))
                })?;
                if !accepted {
                    return Ok(json!({
                        "error": statement.metadata.get("message").cloned().unwrap_or_default()
                    }));
                }
            }
            "call" => {
                let dependency_name = metadata(statement, "dependency")?;
                let operation_name = metadata(statement, "operation")?;
                let argument = metadata(statement, "argument")?;
                let argument = expression::parse(argument)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let argument = expression::evaluate(&argument, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let dependency = children(graph, &flow.id, "input")
                    .into_iter()
                    .find(|node| node.name == dependency_name)
                    .ok_or_else(|| {
                        RuntimeError(format!(
                            "{} references missing dependency '{dependency_name}'",
                            statement.id
                        ))
                    })?;
                let capacity_name = dependency.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no capacity type", dependency.id))
                })?;
                let provider_id = provider_for(graph, &dependency.id).ok_or_else(|| {
                    RuntimeError(format!("{} has no bound provider", dependency.id))
                })?;
                let provider = graph
                    .nodes
                    .iter()
                    .find(|node| node.id == provider_id)
                    .ok_or_else(|| RuntimeError(format!("missing provider '{provider_id}'")))?;
                let implementation = provider.implementation.as_deref().ok_or_else(|| {
                    RuntimeError(format!(
                        "provider '{}' has no native binding",
                        provider.name
                    ))
                })?;
                let capacity = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind == "capacity" && node.name == capacity_name)
                    .ok_or_else(|| RuntimeError(format!("missing capacity '{capacity_name}'")))?;
                let operation = children(graph, &capacity.id, "operation")
                    .into_iter()
                    .find(|node| node.name == operation_name)
                    .ok_or_else(|| {
                        RuntimeError(format!(
                            "capacity '{capacity_name}' has no operation '{operation_name}'"
                        ))
                    })?;
                let (operation_input, operation_output) = operation
                    .type_name
                    .as_deref()
                    .and_then(|value| value.split_once("->"))
                    .ok_or_else(|| {
                        RuntimeError(format!("{} has no valid signature", operation.id))
                    })?;
                validate_value(graph, operation_input, &argument, "call argument")?;
                let result = runtime.invoke(ProviderCall {
                    provider: &provider.name,
                    capacity: capacity_name,
                    implementation,
                    operation: operation_name,
                    config: provider_config(graph, &provider.id)?,
                    input: argument,
                });
                let propagate = metadata(statement, "propagate")? == "true";
                match (result, generic(operation_output, "Result"), propagate) {
                    (Ok(value), Some(inner), true) => {
                        validate_value(graph, inner, &value, "provider result")?;
                        values.insert(statement.name.clone(), value);
                    }
                    (Err(message), Some(_), true) => return Ok(json!({ "error": message })),
                    (Ok(value), None, false) => {
                        validate_value(graph, operation_output, &value, "provider result")?;
                        values.insert(statement.name.clone(), value);
                    }
                    (Err(message), None, false) => {
                        return Err(RuntimeError(format!(
                            "provider '{}' failed: {message}",
                            provider.name
                        )));
                    }
                    _ => {
                        return Err(RuntimeError(format!(
                            "{} has inconsistent Result propagation metadata",
                            statement.id
                        )));
                    }
                }
            }
            "attempt" => {
                let dependency_name = metadata(statement, "dependency")?;
                let operation_name = metadata(statement, "operation")?;
                let argument = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let argument = expression::evaluate(&argument, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let dependency = children(graph, &flow.id, "input")
                    .into_iter()
                    .find(|node| node.name == dependency_name)
                    .ok_or_else(|| {
                        RuntimeError(format!(
                            "{} references missing dependency '{dependency_name}'",
                            statement.id
                        ))
                    })?;
                let capacity_name = dependency.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no capacity type", dependency.id))
                })?;
                let provider_id = provider_for(graph, &dependency.id).ok_or_else(|| {
                    RuntimeError(format!("{} has no bound provider", dependency.id))
                })?;
                let provider = graph
                    .nodes
                    .iter()
                    .find(|node| node.id == provider_id)
                    .ok_or_else(|| RuntimeError(format!("missing provider '{provider_id}'")))?;
                let implementation = provider.implementation.as_deref().ok_or_else(|| {
                    RuntimeError(format!(
                        "provider '{}' has no native binding",
                        provider.name
                    ))
                })?;
                let capacity = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind == "capacity" && node.name == capacity_name)
                    .ok_or_else(|| RuntimeError(format!("missing capacity '{capacity_name}'")))?;
                let operation = children(graph, &capacity.id, "operation")
                    .into_iter()
                    .find(|node| node.name == operation_name)
                    .ok_or_else(|| {
                        RuntimeError(format!(
                            "capacity '{capacity_name}' has no operation '{operation_name}'"
                        ))
                    })?;
                if operation.metadata.get("idempotent").map(String::as_str) != Some("true") {
                    return Err(RuntimeError(format!(
                        "{} resilient operation is not idempotent",
                        statement.id
                    )));
                }
                let (operation_input, operation_output) = operation
                    .type_name
                    .as_deref()
                    .and_then(|value| value.split_once("->"))
                    .ok_or_else(|| {
                        RuntimeError(format!("{} has no valid signature", operation.id))
                    })?;
                validate_value(graph, operation_input, &argument, "attempt argument")?;
                let retry = metadata(statement, "retry")?.parse::<u32>().map_err(|_| {
                    RuntimeError(format!("{} has invalid retry metadata", statement.id))
                })?;
                let timeout_ms =
                    metadata(statement, "timeout_ms")?
                        .parse::<u64>()
                        .map_err(|_| {
                            RuntimeError(format!("{} has invalid timeout metadata", statement.id))
                        })?;
                let result = invoke_with_resilience(
                    runtime,
                    OwnedProviderCall {
                        provider: provider.name.clone(),
                        capacity: capacity_name.into(),
                        implementation: implementation.into(),
                        operation: operation_name.into(),
                        config: provider_config(graph, &provider.id)?,
                        input: argument,
                    },
                    retry,
                    timeout_ms,
                );
                let propagate = metadata(statement, "propagate")? == "true";
                match (result, generic(operation_output, "Result"), propagate) {
                    (Ok(value), Some(inner), true) => {
                        validate_value(graph, inner, &value, "attempt result")?;
                        values.insert(statement.name.clone(), value);
                    }
                    (Err(message), Some(_), true) => return Ok(json!({ "error": message })),
                    (Ok(value), None, false) => {
                        validate_value(graph, operation_output, &value, "attempt result")?;
                        values.insert(statement.name.clone(), value);
                    }
                    (Err(message), None, false) => {
                        return Err(RuntimeError(format!(
                            "resilient provider '{}' failed: {message}",
                            provider.name
                        )));
                    }
                    _ => {
                        return Err(RuntimeError(format!(
                            "{} has inconsistent attempt propagation metadata",
                            statement.id
                        )));
                    }
                }
            }
            "make" => {
                let type_name = statement
                    .type_name
                    .as_deref()
                    .ok_or_else(|| RuntimeError(format!("{} has no record type", statement.id)))?;
                let mut object = Map::new();
                for assignment in children(graph, &statement.id, "assign") {
                    let expression = statement_expression(assignment)?;
                    let value = expression::evaluate(&expression, &values)
                        .map_err(|message| RuntimeError(format!("{}: {message}", assignment.id)))?;
                    object.insert(assignment.name.clone(), value);
                }
                let value = Value::Object(object);
                validate_value(graph, type_name, &value, "constructed record")?;
                values.insert(statement.name.clone(), value);
            }
            "fold" => {
                let type_name = statement
                    .type_name
                    .as_deref()
                    .ok_or_else(|| RuntimeError(format!("{} has no fold type", statement.id)))?;
                let collection = expression::parse(metadata(statement, "collection")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let collection = expression::evaluate(&collection, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let collection = collection.as_array().ok_or_else(|| {
                    RuntimeError(format!(
                        "{} source did not evaluate to a collection",
                        statement.id
                    ))
                })?;
                let initial = expression::parse(metadata(statement, "initial")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let mut accumulator = expression::evaluate(&initial, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                validate_value(graph, type_name, &accumulator, "fold initial value")?;
                let item_name = metadata(statement, "item")?;
                let update = expression::parse(metadata(statement, "update")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                for item in collection {
                    let mut scope = values.clone();
                    scope.insert("value".into(), accumulator);
                    scope.insert(item_name.into(), item.clone());
                    accumulator = expression::evaluate(&update, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    validate_value(graph, type_name, &accumulator, "fold next value")?;
                }
                values.insert(statement.name.clone(), accumulator);
            }
            "run" => {
                let target_name = metadata(statement, "flow")?;
                let argument = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let argument = expression::evaluate(&argument, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let target = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind == "flow" && node.name == target_name)
                    .ok_or_else(|| RuntimeError(format!("unknown flow '{target_name}'")))?;
                let target_output = target
                    .type_name
                    .as_deref()
                    .and_then(|value| value.split_once("->"))
                    .map(|(_, output)| output)
                    .ok_or_else(|| {
                        RuntimeError(format!("flow '{target_name}' has no signature"))
                    })?;
                let result = evaluate_flow_inner(graph, target_name, argument, runtime, depth + 1)?;
                let propagate = metadata(statement, "propagate")? == "true";
                match (generic(target_output, "Result"), propagate) {
                    (Some(_), true) => {
                        let object = result.as_object().ok_or_else(|| {
                            RuntimeError(format!("flow '{target_name}' returned invalid Result"))
                        })?;
                        if let Some(value) = object.get("ok") {
                            values.insert(statement.name.clone(), value.clone());
                        } else if let Some(message) = object.get("error") {
                            return Ok(json!({ "error": message }));
                        } else {
                            return Err(RuntimeError(format!(
                                "flow '{target_name}' returned invalid Result"
                            )));
                        }
                    }
                    (None, false) => {
                        values.insert(statement.name.clone(), result);
                    }
                    _ => {
                        return Err(RuntimeError(format!(
                            "{} has inconsistent flow propagation metadata",
                            statement.id
                        )));
                    }
                }
            }
            "match" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no match result type", statement.id))
                })?;
                let subject = expression::parse(metadata(statement, "subject")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let subject = expression::evaluate(&subject, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let variant = subject.as_str().ok_or_else(|| {
                    RuntimeError(format!("{} subject did not evaluate to enum", statement.id))
                })?;
                let case = children(graph, &statement.id, "case")
                    .into_iter()
                    .find(|case| case.name == variant)
                    .ok_or_else(|| {
                        RuntimeError(format!("{} has no case for '{variant}'", statement.id))
                    })?;
                let expression = statement_expression(case)?;
                let value = expression::evaluate(&expression, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", case.id)))?;
                validate_value(graph, type_name, &value, "match result")?;
                values.insert(statement.name.clone(), value);
            }
            "map" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no map result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                let item_name = metadata(statement, "item")?;
                let mapper = statement_expression(statement)?;
                let mut mapped = Vec::with_capacity(collection.len());
                let produces_set = generic(type_name, "Set").is_some();
                for item in collection {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item);
                    let value = expression::evaluate(&mapper, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    if !produces_set || !mapped.contains(&value) {
                        mapped.push(value);
                    }
                }
                let mapped = Value::Array(mapped);
                validate_value(graph, type_name, &mapped, "map result")?;
                values.insert(statement.name.clone(), mapped);
            }
            "filter" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no filter result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                let item_name = metadata(statement, "item")?;
                let predicate = expression::parse(metadata(statement, "predicate")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let mut filtered = Vec::new();
                for item in collection {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item.clone());
                    let accepted = expression::evaluate(&predicate, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?
                        .as_bool()
                        .ok_or_else(|| {
                            RuntimeError(format!("{} predicate is not boolean", statement.id))
                        })?;
                    if accepted {
                        filtered.push(item);
                    }
                }
                let filtered = Value::Array(filtered);
                validate_value(graph, type_name, &filtered, "filter result")?;
                values.insert(statement.name.clone(), filtered);
            }
            "sort" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no sort result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                let item_name = metadata(statement, "item")?;
                let key_expression = expression::parse(metadata(statement, "key")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let direction = metadata(statement, "direction")?;
                let mut keyed = Vec::with_capacity(collection.len());
                for (index, item) in collection.into_iter().enumerate() {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item.clone());
                    let key = expression::evaluate(&key_expression, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    keyed.push((key, index, item));
                }
                let mut comparison_error = None;
                keyed.sort_by(|left, right| match compare_sort_keys(&left.0, &right.0) {
                    Ok(ordering) => if direction == "desc" {
                        ordering.reverse()
                    } else {
                        ordering
                    }
                    .then_with(|| left.1.cmp(&right.1)),
                    Err(error) => {
                        comparison_error = Some(error);
                        Ordering::Equal
                    }
                });
                if let Some(error) = comparison_error {
                    return Err(RuntimeError(format!("{}: {error}", statement.id)));
                }
                let sorted = Value::Array(keyed.into_iter().map(|(_, _, item)| item).collect());
                validate_value(graph, type_name, &sorted, "sort result")?;
                values.insert(statement.name.clone(), sorted);
            }
            "group" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no group result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                let item_name = metadata(statement, "item")?;
                let key_expression = expression::parse(metadata(statement, "key")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let mut grouped = Map::new();
                for item in collection {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item.clone());
                    let key = expression::evaluate(&key_expression, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    let key = key.as_str().ok_or_else(|| {
                        RuntimeError(format!(
                            "{} group key did not evaluate to a string-like value",
                            statement.id
                        ))
                    })?;
                    let group = grouped
                        .entry(key.to_string())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    let Value::Array(group) = group else {
                        return Err(RuntimeError(format!(
                            "{} produced an invalid group",
                            statement.id
                        )));
                    };
                    group.push(item);
                }
                let grouped = Value::Object(grouped);
                validate_value(graph, type_name, &grouped, "group result")?;
                values.insert(statement.name.clone(), grouped);
            }
            "parallel" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no parallel result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                let item_name = metadata(statement, "item")?;
                let target_name = metadata(statement, "flow")?;
                let argument_expression = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let target = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind == "flow" && node.name == target_name)
                    .ok_or_else(|| RuntimeError(format!("unknown flow '{target_name}'")))?;
                let target_output = target
                    .type_name
                    .as_deref()
                    .and_then(|value| value.split_once("->"))
                    .map(|(_, output)| output)
                    .ok_or_else(|| {
                        RuntimeError(format!("flow '{target_name}' has no signature"))
                    })?;
                let mut tasks = Vec::with_capacity(collection.len());
                for item in collection {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item);
                    let argument = expression::evaluate(&argument_expression, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    let worker = runtime
                        .fork()
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    tasks.push((argument, worker));
                }
                let results = std::thread::scope(|thread_scope| {
                    let handles = tasks
                        .into_iter()
                        .map(|(argument, mut worker)| {
                            thread_scope.spawn(move || {
                                evaluate_flow_inner(
                                    graph,
                                    target_name,
                                    argument,
                                    &mut *worker,
                                    depth + 1,
                                )
                            })
                        })
                        .collect::<Vec<_>>();
                    handles
                        .into_iter()
                        .map(|handle| {
                            handle.join().map_err(|_| {
                                RuntimeError(format!("{} parallel worker panicked", statement.id))
                            })?
                        })
                        .collect::<Result<Vec<_>, _>>()
                })?;
                let propagate = metadata(statement, "propagate")? == "true";
                let mut output = Vec::with_capacity(results.len());
                for result in results {
                    match (generic(target_output, "Result"), propagate) {
                        (Some(_), true) => {
                            let object = result.as_object().ok_or_else(|| {
                                RuntimeError(format!(
                                    "parallel flow '{target_name}' returned invalid Result"
                                ))
                            })?;
                            if let Some(value) = object.get("ok") {
                                output.push(value.clone());
                            } else if let Some(message) = object.get("error") {
                                return Ok(json!({ "error": message }));
                            } else {
                                return Err(RuntimeError(format!(
                                    "parallel flow '{target_name}' returned invalid Result"
                                )));
                            }
                        }
                        (None, false) => output.push(result),
                        _ => {
                            return Err(RuntimeError(format!(
                                "{} has inconsistent parallel propagation metadata",
                                statement.id
                            )));
                        }
                    }
                }
                let output = Value::Array(output);
                validate_value(graph, type_name, &output, "parallel result")?;
                values.insert(statement.name.clone(), output);
            }
            "race" => {
                let type_name = statement.type_name.as_deref().ok_or_else(|| {
                    RuntimeError(format!("{} has no race result type", statement.id))
                })?;
                let collection = evaluated_collection(statement, &values)?;
                if collection.is_empty() {
                    return Err(RuntimeError(format!(
                        "{} race source is empty",
                        statement.id
                    )));
                }
                let item_name = metadata(statement, "item")?;
                let target_name = metadata(statement, "flow")?;
                let argument_expression = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let target = graph
                    .nodes
                    .iter()
                    .find(|node| node.kind == "flow" && node.name == target_name)
                    .ok_or_else(|| RuntimeError(format!("unknown flow '{target_name}'")))?;
                let target_output = target
                    .type_name
                    .as_deref()
                    .and_then(|value| value.split_once("->"))
                    .map(|(_, output)| output)
                    .ok_or_else(|| {
                        RuntimeError(format!("flow '{target_name}' has no signature"))
                    })?;
                let (sender, receiver) = std::sync::mpsc::channel();
                let task_count = collection.len();
                for (index, item) in collection.into_iter().enumerate() {
                    let mut scope = values.clone();
                    scope.insert(item_name.into(), item);
                    let argument = expression::evaluate(&argument_expression, &scope)
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    let mut worker = runtime
                        .fork()
                        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                    let graph = graph.clone();
                    let target_name = target_name.to_string();
                    let sender = sender.clone();
                    std::thread::spawn(move || {
                        let result = evaluate_flow_inner(
                            &graph,
                            &target_name,
                            argument,
                            &mut *worker,
                            depth + 1,
                        );
                        let _ = sender.send((index, result));
                    });
                }
                drop(sender);
                let propagate = metadata(statement, "propagate")? == "true";
                let mut failures = vec![None; task_count];
                let mut winner = None;
                for _ in 0..task_count {
                    let (index, result) = receiver.recv().map_err(|_| {
                        RuntimeError(format!("{} race workers disconnected", statement.id))
                    })?;
                    match (result, generic(target_output, "Result"), propagate) {
                        (Ok(result), Some(inner), true) => {
                            let object = result.as_object().ok_or_else(|| {
                                RuntimeError(format!(
                                    "race flow '{target_name}' returned invalid Result"
                                ))
                            })?;
                            if let Some(value) = object.get("ok") {
                                validate_value(graph, inner, value, "race winner")?;
                                winner = Some(value.clone());
                                break;
                            }
                            failures[index] = object.get("error").map(|error| {
                                error
                                    .as_str()
                                    .map_or_else(|| error.to_string(), str::to_string)
                            });
                        }
                        (Ok(result), None, false) => {
                            validate_value(graph, type_name, &result, "race winner")?;
                            winner = Some(result);
                            break;
                        }
                        (Err(error), _, _) => failures[index] = Some(error.to_string()),
                        _ => {
                            return Err(RuntimeError(format!(
                                "{} has inconsistent race propagation metadata",
                                statement.id
                            )));
                        }
                    }
                }
                if let Some(winner) = winner {
                    values.insert(statement.name.clone(), winner);
                } else {
                    let message = failures
                        .into_iter()
                        .flatten()
                        .next()
                        .unwrap_or_else(|| "race_no_success".into());
                    if generic(target_output, "Result").is_some() && propagate {
                        return Ok(json!({ "error": message }));
                    }
                    return Err(RuntimeError(message));
                }
            }
            "emit" => {
                let event_name = metadata(statement, "event")?;
                let argument = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let argument = expression::evaluate(&argument, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let mut subscriptions = graph
                    .nodes
                    .iter()
                    .filter(|node| node.kind == "subscription" && node.name == event_name)
                    .collect::<Vec<_>>();
                subscriptions.sort_by_key(|node| {
                    node.metadata
                        .get("order")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(usize::MAX)
                });
                for subscription in subscriptions {
                    let flow = metadata(subscription, "flow")?;
                    let result =
                        evaluate_flow_inner(graph, flow, argument.clone(), runtime, depth + 1)?;
                    if result
                        .as_object()
                        .is_some_and(|object| object.contains_key("error"))
                    {
                        return Ok(result);
                    }
                }
            }
            "enqueue" => {
                let job_name = metadata(statement, "job")?;
                let argument = expression::parse(metadata(statement, "argument")?)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                let argument = expression::evaluate(&argument, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                enqueue_job(graph, runtime, job_name, argument, None, 0, now_millis())?;
            }
            "return" => {
                let expression = statement_expression(statement)?;
                let value = expression::evaluate(&expression, &values)
                    .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
                if let Some(inner) = generic(signature.1, "Result") {
                    validate_value(graph, inner, &value, "return")?;
                    return Ok(json!({ "ok": value }));
                }
                validate_value(graph, signature.1, &value, "return")?;
                return Ok(value);
            }
            kind => return Err(RuntimeError(format!("unsupported flow statement '{kind}'"))),
        }
    }
    Err(RuntimeError(format!(
        "flow '{flow_name}' completed without return"
    )))
}

fn metadata<'a>(node: &'a GraphNode, name: &str) -> Result<&'a str, RuntimeError> {
    node.metadata
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| RuntimeError(format!("{} is missing {name} metadata", node.id)))
}

fn statement_expression(statement: &GraphNode) -> Result<expression::Expr, RuntimeError> {
    let source = metadata(statement, "expression")?;
    expression::parse(source)
        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))
}

fn evaluated_collection(
    statement: &GraphNode,
    values: &BTreeMap<String, Value>,
) -> Result<Vec<Value>, RuntimeError> {
    let source = expression::parse(metadata(statement, "collection")?)
        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?;
    expression::evaluate(&source, values)
        .map_err(|message| RuntimeError(format!("{}: {message}", statement.id)))?
        .as_array()
        .cloned()
        .ok_or_else(|| {
            RuntimeError(format!(
                "{} source did not evaluate to a collection",
                statement.id
            ))
        })
}

fn compare_sort_keys(left: &Value, right: &Value) -> Result<Ordering, String> {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .and_then(|left| right.as_f64().and_then(|right| left.partial_cmp(&right)))
            .ok_or_else(|| "sort keys contain a non-finite number".into()),
        (Value::String(left), Value::String(right)) => Ok(left.cmp(right)),
        _ => Err("sort keys must have one ordered scalar type".into()),
    }
}

fn memory_store_call(stores: &mut MemoryStoreMap, call: ProviderCall<'_>) -> Result<Value, String> {
    let store = stores.entry(call.provider.to_string()).or_default();
    match call.operation {
        "save" => {
            let id = record_id(&call.input)?;
            store.insert(id, call.input.clone());
            Ok(call.input)
        }
        "find" => {
            let id = string_input(&call.input, "find")?;
            store.get(id).cloned().ok_or_else(|| "not_found".into())
        }
        "delete" => {
            let id = string_input(&call.input, "delete")?;
            Ok(Value::Bool(store.remove(id).is_some()))
        }
        "list" => Ok(Value::Array(store.values().cloned().collect())),
        "query" => store_query(store.values().cloned().collect(), &call.input),
        "find_by" => store_find_by(store.values().cloned().collect(), &call.input),
        operation => Err(format!(
            "memory store does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn sqlite_store_call(connection: &Connection, call: ProviderCall<'_>) -> Result<Value, String> {
    match call.operation {
        "save" => {
            let id = record_id(&call.input)?;
            let payload = serde_json::to_string(&call.input).map_err(|error| error.to_string())?;
            connection
                .execute(
                    "INSERT INTO axl_records (provider, record_id, payload) VALUES (?1, ?2, ?3) \
                     ON CONFLICT(provider, record_id) DO UPDATE SET payload = excluded.payload",
                    params![call.provider, id, payload],
                )
                .map_err(|error| error.to_string())?;
            Ok(call.input)
        }
        "find" => {
            let id = string_input(&call.input, "find")?;
            let result = connection.query_row(
                "SELECT payload FROM axl_records WHERE provider = ?1 AND record_id = ?2",
                params![call.provider, id],
                |row| row.get::<_, String>(0),
            );
            match result {
                Ok(payload) => serde_json::from_str(&payload).map_err(|error| error.to_string()),
                Err(rusqlite::Error::QueryReturnedNoRows) => Err("not_found".into()),
                Err(error) => Err(error.to_string()),
            }
        }
        "delete" => {
            let id = string_input(&call.input, "delete")?;
            let removed = connection
                .execute(
                    "DELETE FROM axl_records WHERE provider = ?1 AND record_id = ?2",
                    params![call.provider, id],
                )
                .map_err(|error| error.to_string())?;
            Ok(Value::Bool(removed > 0))
        }
        "list" => {
            let mut statement = connection
                .prepare("SELECT payload FROM axl_records WHERE provider = ?1 ORDER BY record_id")
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(params![call.provider], |row| row.get::<_, String>(0))
                .map_err(|error| error.to_string())?;
            let mut values = Vec::new();
            for row in rows {
                let payload = row.map_err(|error| error.to_string())?;
                values.push(serde_json::from_str(&payload).map_err(|error| error.to_string())?);
            }
            Ok(Value::Array(values))
        }
        "query" => {
            let mut statement = connection
                .prepare("SELECT payload FROM axl_records WHERE provider = ?1 ORDER BY record_id")
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(params![call.provider], |row| row.get::<_, String>(0))
                .map_err(|error| error.to_string())?;
            let mut values = Vec::new();
            for row in rows {
                let payload = row.map_err(|error| error.to_string())?;
                values.push(serde_json::from_str(&payload).map_err(|error| error.to_string())?);
            }
            store_query(values, &call.input)
        }
        operation => Err(format!(
            "SQLite store does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn load_document_file(path: &str) -> Result<DocumentStoreMap, String> {
    let file = std::path::Path::new(path);
    if !file.exists() {
        return Ok(DocumentStoreMap::new());
    }
    let text = std::fs::read_to_string(file)
        .map_err(|error| format!("cannot read document store '{path}': {error}"))?;
    if text.trim().is_empty() {
        return Ok(DocumentStoreMap::new());
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("document store '{path}' is not valid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("document store '{path}' must be a JSON object"))?;
    let mut stores = DocumentStoreMap::new();
    for (provider, records) in object {
        let record_object = records
            .as_object()
            .ok_or_else(|| format!("document store provider '{provider}' must be a JSON object"))?;
        stores.insert(
            provider.clone(),
            record_object.clone().into_iter().collect(),
        );
    }
    Ok(stores)
}

fn write_document_file(path: &str, stores: &DocumentStoreMap) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create document store directory: {error}"))?;
    }
    let mut root = Map::new();
    for (provider, records) in stores {
        root.insert(
            provider.clone(),
            Value::Object(records.clone().into_iter().collect()),
        );
    }
    let payload = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|error| format!("cannot serialize document store: {error}"))?;
    std::fs::write(path, payload)
        .map_err(|error| format!("cannot write document store '{path}': {error}"))
}

fn document_store_call(
    stores: &mut DocumentStoreMap,
    call: ProviderCall<'_>,
    flush_path: Option<&str>,
) -> Result<Value, String> {
    let result = match call.operation {
        "save" => {
            let id = record_id(&call.input)?;
            stores
                .entry(call.provider.to_string())
                .or_default()
                .insert(id, call.input.clone());
            Ok(call.input.clone())
        }
        "find" => {
            let id = string_input(&call.input, "find")?;
            stores
                .get(call.provider)
                .and_then(|store| store.get(id).cloned())
                .ok_or_else(|| "not_found".into())
        }
        "delete" => {
            let id = string_input(&call.input, "delete")?;
            let removed = stores
                .get_mut(call.provider)
                .is_some_and(|store| store.remove(id).is_some());
            Ok(Value::Bool(removed))
        }
        "list" => {
            let values = stores
                .get(call.provider)
                .map(|store| store.values().cloned().collect())
                .unwrap_or_default();
            Ok(Value::Array(values))
        }
        "query" => {
            let values = stores
                .get(call.provider)
                .map(|store| store.values().cloned().collect())
                .unwrap_or_default();
            store_query(values, &call.input)
        }
        "find_by" => {
            let values = stores
                .get(call.provider)
                .map(|store| store.values().cloned().collect())
                .unwrap_or_default();
            store_find_by(values, &call.input)
        }
        operation => Err(format!(
            "document store does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }?;
    if matches!(call.operation, "save" | "delete")
        && let Some(path) = flush_path
    {
        write_document_file(path, stores)?;
    }
    Ok(result)
}

fn store_find_by(records: Vec<Value>, input: &Value) -> Result<Value, String> {
    let lookup = input
        .as_object()
        .ok_or_else(|| "store find_by requires a lookup object".to_string())?;
    let field = lookup
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| "store find_by requires field".to_string())?;
    let expected = lookup
        .get("value")
        .ok_or_else(|| "store find_by requires value".to_string())?;
    for record in records {
        if record
            .as_object()
            .and_then(|object| object.get(field))
            .is_some_and(|value| value == expected)
        {
            return Ok(record);
        }
    }
    Err("not_found".into())
}

fn store_query(records: Vec<Value>, input: &Value) -> Result<Value, String> {
    let spec = input
        .as_object()
        .ok_or_else(|| "store query requires an object QuerySpec".to_string())?;
    let owned_filter;
    let filter = match spec.get("filter") {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) if map.is_empty() => None,
        Some(Value::Object(map)) => Some(map),
        Some(Value::String(text)) if text.is_empty() || text == "{}" => None,
        Some(Value::String(text)) => {
            owned_filter = serde_json::from_str::<Value>(text)
                .map_err(|error| format!("query filter text must be a JSON object: {error}"))?;
            match &owned_filter {
                Value::Object(map) if map.is_empty() => None,
                Value::Object(_) => owned_filter.as_object(),
                _ => {
                    return Err("query filter text must be a JSON object".into());
                }
            }
        }
        Some(_) => return Err("query filter must be a map or JSON object text".into()),
    };

    let mut items: Vec<Value> = records
        .into_iter()
        .filter(|record| record_matches_query_filter(record, filter))
        .collect();

    if let Some(order_by) = spec
        .get("order_by")
        .and_then(Value::as_str)
        .filter(|field| !field.is_empty())
    {
        let descending = matches!(
            spec.get("direction").and_then(Value::as_str),
            Some("desc") | Some("DESC")
        );
        items.sort_by(|left, right| {
            let ordering =
                compare_query_field_keys(left, right, order_by).unwrap_or(Ordering::Equal);
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    let total = items.len() as i64;
    let offset = spec
        .get("offset")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as usize;
    let limit = spec
        .get("limit")
        .and_then(Value::as_i64)
        .filter(|value| *value >= 0)
        .map(|value| value as usize);
    let page: Vec<Value> = items
        .into_iter()
        .skip(offset)
        .take(limit.unwrap_or(usize::MAX))
        .collect();
    let limit_out = limit.map(|value| value as i64).unwrap_or(total);

    Ok(json!({
        "items": page,
        "total": total,
        "limit": limit_out,
        "offset": offset as i64,
    }))
}

fn record_matches_query_filter(
    record: &Value,
    filter: Option<&serde_json::Map<String, Value>>,
) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let Some(object) = record.as_object() else {
        return false;
    };
    filter.iter().all(|(key, expected)| {
        object
            .get(key)
            .is_some_and(|actual| values_equal_for_query_filter(actual, expected))
    })
}

fn values_equal_for_query_filter(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    let Some(expected_text) = expected.as_str() else {
        return false;
    };
    match actual {
        Value::String(value) => value == expected_text,
        Value::Bool(value) => {
            (*value && expected_text == "true") || (!*value && expected_text == "false")
        }
        Value::Number(value) => {
            value.to_string() == expected_text
                || value
                    .as_i64()
                    .is_some_and(|number| number.to_string() == expected_text)
                || value
                    .as_f64()
                    .is_some_and(|number| number.to_string() == expected_text)
        }
        _ => false,
    }
}

fn compare_query_field_keys(left: &Value, right: &Value, field: &str) -> Result<Ordering, String> {
    let left_key = left.as_object().and_then(|object| object.get(field));
    let right_key = right.as_object().and_then(|object| object.get(field));
    match (left_key, right_key) {
        (None, None) => Ok(Ordering::Equal),
        (None, Some(_)) => Ok(Ordering::Less),
        (Some(_), None) => Ok(Ordering::Greater),
        (Some(left), Some(right)) => compare_sort_keys(left, right),
    }
}

fn memory_tx_call(
    stores: &mut MemoryStoreMap,
    stack: &mut MemoryTxStack,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    match call.operation {
        "begin" => {
            let tid = string_input(&call.input, "begin")?.to_string();
            if stack.iter().any(|(open, _)| open == &tid) {
                return Err("tx_already_open".into());
            }
            stack.push((tid.clone(), stores.clone()));
            Ok(Value::String(tid))
        }
        "commit" => {
            let tid = string_input(&call.input, "commit")?;
            let Some((open, _)) = stack.last() else {
                return Err("tx_not_open".into());
            };
            if open != tid {
                return Err("tx_mismatch".into());
            }
            stack.pop();
            Ok(Value::Null)
        }
        "rollback" => {
            let tid = string_input(&call.input, "rollback")?;
            let Some((open, _)) = stack.last() else {
                return Err("tx_not_open".into());
            };
            if open != tid {
                return Err("tx_mismatch".into());
            }
            let (_, snapshot) = stack.pop().expect("checked");
            *stores = snapshot;
            Ok(Value::Null)
        }
        operation => Err(format!(
            "memory transaction does not implement operation '{operation}'"
        )),
    }
}

fn sqlite_tx_call(
    connection: &Connection,
    stacks: &mut BTreeMap<String, Vec<String>>,
    key: &str,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let stack = stacks.entry(key.to_string()).or_default();
    match call.operation {
        "begin" => {
            let tid = string_input(&call.input, "begin")?.to_string();
            if stack.iter().any(|open| open == &tid) {
                return Err("tx_already_open".into());
            }
            if stack.is_empty() {
                connection
                    .execute_batch("BEGIN")
                    .map_err(|error| error.to_string())?;
            } else {
                let savepoint = sqlite_savepoint_name(&tid);
                connection
                    .execute_batch(&format!("SAVEPOINT {savepoint}"))
                    .map_err(|error| error.to_string())?;
            }
            stack.push(tid.clone());
            Ok(Value::String(tid))
        }
        "commit" => {
            let tid = string_input(&call.input, "commit")?;
            let Some(open) = stack.last() else {
                return Err("tx_not_open".into());
            };
            if open != tid {
                return Err("tx_mismatch".into());
            }
            if stack.len() == 1 {
                connection
                    .execute_batch("COMMIT")
                    .map_err(|error| error.to_string())?;
            } else {
                let savepoint = sqlite_savepoint_name(tid);
                connection
                    .execute_batch(&format!("RELEASE {savepoint}"))
                    .map_err(|error| error.to_string())?;
            }
            stack.pop();
            Ok(Value::Null)
        }
        "rollback" => {
            let tid = string_input(&call.input, "rollback")?;
            let Some(open) = stack.last() else {
                return Err("tx_not_open".into());
            };
            if open != tid {
                return Err("tx_mismatch".into());
            }
            if stack.len() == 1 {
                connection
                    .execute_batch("ROLLBACK")
                    .map_err(|error| error.to_string())?;
            } else {
                let savepoint = sqlite_savepoint_name(tid);
                connection
                    .execute_batch(&format!("ROLLBACK TO {savepoint}; RELEASE {savepoint}"))
                    .map_err(|error| error.to_string())?;
            }
            stack.pop();
            Ok(Value::Null)
        }
        operation => Err(format!(
            "SQLite transaction does not implement operation '{operation}'"
        )),
    }
}

fn sqlite_savepoint_name(tid: &str) -> String {
    let safe: String = tid
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("axl_sp_{safe}")
}

fn sqlite_migration_marker(version: &str) -> String {
    let safe: String = version
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("axl_schema_{safe}")
}

fn memory_migrate_call(
    history: &mut MemoryMigrationMap,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let versions = history.entry(call.provider.to_string()).or_default();
    match call.operation {
        "up" => {
            let version = string_input(&call.input, "up")?.to_string();
            if version.is_empty() || version == "0" {
                return Err("invalid_version".into());
            }
            if versions.iter().any(|applied| applied == &version) {
                return Err("already_applied".into());
            }
            versions.push(version.clone());
            Ok(Value::String(version))
        }
        "down" => {
            let version = string_input(&call.input, "down")?;
            let Some(head) = versions.last() else {
                return Err("nothing_to_rollback".into());
            };
            if head != version {
                return Err("not_head".into());
            }
            let rolled = versions.pop().expect("checked");
            Ok(Value::String(rolled))
        }
        "status" => {
            if !call.input.is_null() {
                return Err("migration status requires unit".into());
            }
            Ok(Value::String(
                versions.last().cloned().unwrap_or_else(|| "0".to_string()),
            ))
        }
        operation => Err(format!(
            "memory migration does not implement operation '{operation}'"
        )),
    }
}

fn sqlite_migrate_call(connection: &Connection, call: ProviderCall<'_>) -> Result<Value, String> {
    match call.operation {
        "up" => {
            let version = string_input(&call.input, "up")?.to_string();
            if version.is_empty() || version == "0" {
                return Err("invalid_version".into());
            }
            let already = connection.query_row(
                "SELECT 1 FROM axl_schema_history WHERE provider = ?1 AND version = ?2",
                params![call.provider, version],
                |_| Ok(true),
            );
            match already {
                Ok(true) => return Err("already_applied".into()),
                Err(rusqlite::Error::QueryReturnedNoRows) => {}
                Err(error) => return Err(error.to_string()),
                Ok(false) => {}
            }
            let next_seq: i64 = connection
                .query_row(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM axl_schema_history WHERE provider = ?1",
                    params![call.provider],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            connection
                .execute(
                    "INSERT INTO axl_schema_history (provider, version, seq) VALUES (?1, ?2, ?3)",
                    params![call.provider, version, next_seq],
                )
                .map_err(|error| error.to_string())?;
            let marker = sqlite_migration_marker(&version);
            connection
                .execute_batch(&format!(
                    "CREATE TABLE IF NOT EXISTS {marker} (applied INTEGER NOT NULL DEFAULT 1)"
                ))
                .map_err(|error| error.to_string())?;
            Ok(Value::String(version))
        }
        "down" => {
            let version = string_input(&call.input, "down")?.to_string();
            let head = match connection.query_row(
                "SELECT version FROM axl_schema_history WHERE provider = ?1 \
                 ORDER BY seq DESC LIMIT 1",
                params![call.provider],
                |row| row.get::<_, String>(0),
            ) {
                Ok(head) => head,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Err("nothing_to_rollback".into());
                }
                Err(error) => return Err(error.to_string()),
            };
            if head != version {
                return Err("not_head".into());
            }
            connection
                .execute(
                    "DELETE FROM axl_schema_history WHERE provider = ?1 AND version = ?2",
                    params![call.provider, version],
                )
                .map_err(|error| error.to_string())?;
            let marker = sqlite_migration_marker(&version);
            connection
                .execute_batch(&format!("DROP TABLE IF EXISTS {marker}"))
                .map_err(|error| error.to_string())?;
            Ok(Value::String(version))
        }
        "status" => {
            if !call.input.is_null() {
                return Err("migration status requires unit".into());
            }
            match connection.query_row(
                "SELECT version FROM axl_schema_history WHERE provider = ?1 \
                 ORDER BY seq DESC LIMIT 1",
                params![call.provider],
                |row| row.get::<_, String>(0),
            ) {
                Ok(version) => Ok(Value::String(version)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Value::String("0".into())),
                Err(error) => Err(error.to_string()),
            }
        }
        operation => Err(format!(
            "SQLite migration does not implement operation '{operation}'"
        )),
    }
}

fn bearer_auth_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "authorize" {
        return Err(format!(
            "bearer auth does not implement operation '{}'",
            call.operation
        ));
    }
    let expected = call
        .config
        .get("token")
        .and_then(Value::as_str)
        .ok_or_else(|| "bearer_token_not_configured".to_string())?;
    let supplied = call
        .input
        .as_str()
        .ok_or_else(|| "bearer authorize requires a text token".to_string())?;
    Ok(Value::Bool(supplied == expected))
}

/// Mint a compact HS256 JWT for tests and demo fixtures.
///
/// Demo config secrets belong in skill config (same honesty rule as static
/// bearer). Gate 8 secret references are a separate primitive.
pub fn encode_hs256_jwt(secret: &str, claims: &Value) -> Result<String, String> {
    let header = json!({ "alg": "HS256", "typ": "JWT" });
    let header_b64 = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&header).map_err(|error| format!("jwt header encode: {error}"))?,
    );
    let payload_b64 = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(claims).map_err(|error| format!("jwt payload encode: {error}"))?,
    );
    let signing_input = format!("{header_b64}.{payload_b64}");
    let signature = hmac_sha256_b64(secret.as_bytes(), signing_input.as_bytes())?;
    Ok(format!("{signing_input}.{signature}"))
}

fn jwt_auth_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "authorize" {
        return Err(format!(
            "jwt auth does not implement operation '{}'",
            call.operation
        ));
    }
    let secret = call
        .config
        .get("secret")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_secret_not_configured".to_string())?;
    let issuer = call
        .config
        .get("issuer")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_issuer_not_configured".to_string())?;
    let token = call
        .input
        .as_str()
        .ok_or_else(|| "jwt authorize requires a text token".to_string())?;
    Ok(Value::Bool(verify_hs256_jwt(token, secret, issuer)))
}

fn sha256_hex(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

fn password_auth_call(call: ProviderCall<'_>) -> Result<Value, String> {
    let pepper = call
        .config
        .get("pepper")
        .and_then(Value::as_str)
        .unwrap_or("axl-demo-pepper");
    match call.operation {
        "hash" => {
            let password = call
                .input
                .as_str()
                .ok_or_else(|| "password hash requires a text password".to_string())?;
            Ok(Value::String(sha256_hex(&format!("{pepper}:{password}"))))
        }
        "verify" => {
            let object = call
                .input
                .as_object()
                .ok_or_else(|| "password verify requires PasswordCheck".to_string())?;
            let password = object
                .get("password")
                .and_then(Value::as_str)
                .ok_or_else(|| "password verify requires password".to_string())?;
            let hash = object
                .get("hash")
                .and_then(Value::as_str)
                .ok_or_else(|| "password verify requires hash".to_string())?;
            Ok(Value::Bool(constant_time_eq(
                hash.as_bytes(),
                sha256_hex(&format!("{pepper}:{password}")).as_bytes(),
            )))
        }
        operation => Err(format!(
            "password auth does not implement operation '{operation}'"
        )),
    }
}

fn jwt_sign_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "sign" {
        return Err(format!(
            "jwt sign does not implement operation '{}'",
            call.operation
        ));
    }
    let secret = call
        .config
        .get("secret")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_secret_not_configured".to_string())?;
    let issuer = call
        .config
        .get("issuer")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_issuer_not_configured".to_string())?;
    let claims_input = call
        .input
        .as_object()
        .ok_or_else(|| "jwt sign requires JwtClaims".to_string())?;
    let sub = claims_input
        .get("sub")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt sign requires sub".to_string())?;
    let mut claims = json!({ "sub": sub, "iss": issuer });
    if let Some(roles) = claims_input.get("roles") {
        claims
            .as_object_mut()
            .expect("claims object")
            .insert("roles".into(), roles.clone());
    }
    encode_hs256_jwt(secret, &claims).map(Value::String)
}

fn jwt_decode_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "decode" {
        return Err(format!(
            "jwt decode does not implement operation '{}'",
            call.operation
        ));
    }
    let secret = call
        .config
        .get("secret")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_secret_not_configured".to_string())?;
    let issuer = call
        .config
        .get("issuer")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_issuer_not_configured".to_string())?;
    let token = call
        .input
        .as_str()
        .ok_or_else(|| "jwt decode requires a text token".to_string())?;
    let token = token
        .strip_prefix("Bearer ")
        .or_else(|| token.strip_prefix("bearer "))
        .unwrap_or(token);
    decode_hs256_jwt(token, secret, issuer)
}

fn decode_hs256_jwt(token: &str, secret: &str, expected_issuer: &str) -> Result<Value, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("jwt_invalid".into());
    }
    let (header_b64, payload_b64, signature_b64) = (parts[0], parts[1], parts[2]);
    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_b64)
        .map_err(|_| "jwt_invalid".to_string())?;
    let header: Value =
        serde_json::from_slice(&header_bytes).map_err(|_| "jwt_invalid".to_string())?;
    if header.get("alg").and_then(Value::as_str) != Some("HS256") {
        return Err("jwt_invalid".into());
    }
    let signing_input = format!("{header_b64}.{payload_b64}");
    let expected_sig = hmac_sha256_b64(secret.as_bytes(), signing_input.as_bytes())?;
    if !constant_time_eq(expected_sig.as_bytes(), signature_b64.as_bytes()) {
        return Err("jwt_invalid".into());
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| "jwt_invalid".to_string())?;
    let claims: Value =
        serde_json::from_slice(&payload_bytes).map_err(|_| "jwt_invalid".to_string())?;
    let sub = claims
        .get("sub")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "jwt_missing_sub".to_string())?;
    let iss = claims
        .get("iss")
        .and_then(Value::as_str)
        .ok_or_else(|| "jwt_missing_iss".to_string())?;
    if iss != expected_issuer {
        return Err("jwt_invalid_issuer".into());
    }
    let mut decoded = Map::new();
    decoded.insert("sub".into(), Value::String(sub.into()));
    decoded.insert("iss".into(), Value::String(iss.into()));
    if let Some(roles) = claims.get("roles") {
        decoded.insert("roles".into(), roles.clone());
    }
    Ok(Value::Object(decoded))
}

fn verify_hs256_jwt(token: &str, secret: &str, expected_issuer: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    let (header_b64, payload_b64, signature_b64) = (parts[0], parts[1], parts[2]);
    let Ok(header_bytes) = URL_SAFE_NO_PAD.decode(header_b64) else {
        return false;
    };
    let Ok(header) = serde_json::from_slice::<Value>(&header_bytes) else {
        return false;
    };
    if header.get("alg").and_then(Value::as_str) != Some("HS256") {
        return false;
    }
    let signing_input = format!("{header_b64}.{payload_b64}");
    let Ok(expected_sig) = hmac_sha256_b64(secret.as_bytes(), signing_input.as_bytes()) else {
        return false;
    };
    if !constant_time_eq(expected_sig.as_bytes(), signature_b64.as_bytes()) {
        return false;
    }
    let Ok(payload_bytes) = URL_SAFE_NO_PAD.decode(payload_b64) else {
        return false;
    };
    let Ok(claims) = serde_json::from_slice::<Value>(&payload_bytes) else {
        return false;
    };
    let Some(sub) = claims.get("sub").and_then(Value::as_str) else {
        return false;
    };
    if sub.is_empty() {
        return false;
    }
    let Some(iss) = claims.get("iss").and_then(Value::as_str) else {
        return false;
    };
    if iss != expected_issuer {
        return false;
    }
    true
}

fn hmac_sha256_b64(secret: &[u8], message: &[u8]) -> Result<String, String> {
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|error| format!("jwt hmac key: {error}"))?;
    mac.update(message);
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn header_gate_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "process" {
        return Err(format!(
            "header gate does not implement operation '{}'",
            call.operation
        ));
    }
    let header = call
        .config
        .get("header")
        .and_then(Value::as_str)
        .ok_or_else(|| "middleware_header_not_configured".to_string())?
        .to_ascii_lowercase();
    let expected = call
        .config
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "middleware_value_not_configured".to_string())?;
    let headers = call
        .input
        .get("headers")
        .and_then(Value::as_object)
        .ok_or_else(|| "middleware_request_missing_headers".to_string())?;
    let supplied = headers
        .get(&header)
        .and_then(Value::as_str)
        .unwrap_or_default();
    if supplied == expected {
        Ok(call.input.clone())
    } else {
        Err("middleware_rejected".into())
    }
}

fn response_headers_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "process" {
        return Err(format!(
            "response headers does not implement operation '{}'",
            call.operation
        ));
    }
    let header = call
        .config
        .get("header")
        .and_then(Value::as_str)
        .ok_or_else(|| "middleware_header_not_configured".to_string())?
        .to_ascii_lowercase();
    let value = call
        .config
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "middleware_value_not_configured".to_string())?;
    let status = call
        .input
        .get("status")
        .cloned()
        .ok_or_else(|| "middleware_response_missing_status".to_string())?;
    let body = call
        .input
        .get("body")
        .cloned()
        .ok_or_else(|| "middleware_response_missing_body".to_string())?;
    let mut headers = call
        .input
        .get("headers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    headers.insert(header, Value::String(value.into()));
    Ok(json!({
        "status": status,
        "headers": headers,
        "body": body,
    }))
}

fn cors_call(call: ProviderCall<'_>) -> Result<Value, String> {
    if call.operation != "process" {
        return Err(format!(
            "cors middleware does not implement operation '{}'",
            call.operation
        ));
    }
    let allowed_origin = call
        .config
        .get("origin")
        .and_then(Value::as_str)
        .unwrap_or("*");
    if call.input.get("status").is_some() {
        let status = call
            .input
            .get("status")
            .cloned()
            .ok_or_else(|| "middleware_response_missing_status".to_string())?;
        let body = call
            .input
            .get("body")
            .cloned()
            .ok_or_else(|| "middleware_response_missing_body".to_string())?;
        let mut headers = call
            .input
            .get("headers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let methods = call
            .config
            .get("methods")
            .and_then(Value::as_str)
            .unwrap_or("GET,POST,OPTIONS");
        let allow_headers = call
            .config
            .get("headers")
            .and_then(Value::as_str)
            .unwrap_or("content-type,authorization");
        headers.insert(
            "access-control-allow-origin".into(),
            Value::String(allowed_origin.into()),
        );
        headers.insert(
            "access-control-allow-methods".into(),
            Value::String(methods.into()),
        );
        headers.insert(
            "access-control-allow-headers".into(),
            Value::String(allow_headers.into()),
        );
        return Ok(json!({
            "status": status,
            "headers": headers,
            "body": body,
        }));
    }
    if call.input.get("method").is_some() {
        if allowed_origin == "*" {
            return Ok(call.input.clone());
        }
        let headers = call
            .input
            .get("headers")
            .and_then(Value::as_object)
            .ok_or_else(|| "middleware_request_missing_headers".to_string())?;
        let request_origin = headers
            .get("origin")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if request_origin.is_empty() || request_origin == allowed_origin {
            return Ok(call.input.clone());
        }
        return Err("cors_origin_rejected".into());
    }
    Err("cors_middleware_invalid_envelope".into())
}

fn rate_limit_call(
    stores: &mut BTreeMap<String, BTreeMap<String, RateWindow>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    if call.operation != "allow" {
        return Err(format!(
            "rate limit does not implement operation '{}'",
            call.operation
        ));
    }
    let limit = call
        .config
        .get("limit")
        .and_then(Value::as_i64)
        .ok_or_else(|| "rate_limit_not_configured".to_string())?;
    let window_ms = call
        .config
        .get("window_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| "rate_limit_window_not_configured".to_string())?;
    if limit < 0 || window_ms <= 0 {
        return Err("rate_limit_invalid_config".into());
    }
    let key = string_input(&call.input, "allow")?;
    let limit = limit as u64;
    let window = Duration::from_millis(window_ms as u64);
    let provider = stores.entry(call.provider.to_string()).or_default();
    let now = Instant::now();
    let entry = provider.entry(key.to_string()).or_insert(RateWindow {
        started: now,
        count: 0,
    });
    if now.duration_since(entry.started) >= window {
        entry.started = now;
        entry.count = 0;
    }
    if entry.count >= limit {
        return Err("rate_limit_exceeded".into());
    }
    entry.count += 1;
    Ok(Value::Bool(true))
}

fn event_log_call(
    logs: &mut BTreeMap<String, Vec<Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let log = logs.entry(call.provider.to_string()).or_default();
    match call.operation {
        "append" => {
            let entry = call
                .input
                .as_str()
                .ok_or_else(|| "event log append requires text".to_string())?
                .to_string();
            log.push(Value::String(entry.clone()));
            Ok(Value::String(entry))
        }
        "list" => {
            if !call.input.is_null() {
                return Err("event log list requires unit".into());
            }
            Ok(Value::Array(log.clone()))
        }
        operation => Err(format!(
            "event log does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_cache_call(
    caches: &mut BTreeMap<String, BTreeMap<String, Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let cache = caches.entry(call.provider.to_string()).or_default();
    match call.operation {
        "get" => {
            let key = string_input(&call.input, "get")?;
            cache.get(key).cloned().ok_or_else(|| "cache_miss".into())
        }
        "put" => {
            let (key, value) = cache_entry(&call.input)?;
            cache.insert(key, Value::String(value));
            Ok(Value::Null)
        }
        "invalidate" => {
            let key = string_input(&call.input, "invalidate")?;
            Ok(Value::Bool(cache.remove(key).is_some()))
        }
        operation => Err(format!(
            "memory cache does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn sqlite_cache_call(connection: &Connection, call: ProviderCall<'_>) -> Result<Value, String> {
    match call.operation {
        "get" => {
            let key = string_input(&call.input, "get")?;
            let result = connection.query_row(
                "SELECT value FROM axl_cache WHERE provider = ?1 AND cache_key = ?2",
                params![call.provider, key],
                |row| row.get::<_, String>(0),
            );
            match result {
                Ok(value) => Ok(Value::String(value)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Err("cache_miss".into()),
                Err(error) => Err(error.to_string()),
            }
        }
        "put" => {
            let (key, value) = cache_entry(&call.input)?;
            connection
                .execute(
                    "INSERT INTO axl_cache (provider, cache_key, value) VALUES (?1, ?2, ?3) \
                     ON CONFLICT(provider, cache_key) DO UPDATE SET value = excluded.value",
                    params![call.provider, key, value],
                )
                .map_err(|error| error.to_string())?;
            Ok(Value::Null)
        }
        "invalidate" => {
            let key = string_input(&call.input, "invalidate")?;
            let removed = connection
                .execute(
                    "DELETE FROM axl_cache WHERE provider = ?1 AND cache_key = ?2",
                    params![call.provider, key],
                )
                .map_err(|error| error.to_string())?;
            Ok(Value::Bool(removed > 0))
        }
        operation => Err(format!(
            "SQLite cache does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn cache_entry(value: &Value) -> Result<(String, String), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "cache put requires a CacheEntry object".to_string())?;
    let key = object
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| "cache put requires text field 'key'".to_string())?
        .to_string();
    let entry_value = object
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| "cache put requires text field 'value'".to_string())?
        .to_string();
    Ok((key, entry_value))
}

fn memory_logger_call(
    loggers: &mut BTreeMap<String, Vec<Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let log = loggers.entry(call.provider.to_string()).or_default();
    match call.operation {
        "write" => {
            let entry = call
                .input
                .as_str()
                .ok_or_else(|| "logger write requires text".to_string())?
                .to_string();
            log.push(Value::String(entry));
            Ok(Value::Null)
        }
        "list" => {
            if !call.input.is_null() {
                return Err("logger list requires unit".into());
            }
            Ok(Value::Array(log.clone()))
        }
        operation => Err(format!(
            "logger does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_email_call(
    emails: &mut BTreeMap<String, Vec<Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let mailbox = emails.entry(call.provider.to_string()).or_default();
    match call.operation {
        "send" => {
            let message = call
                .input
                .as_object()
                .ok_or_else(|| "email send requires EmailMessage object".to_string())?
                .clone();
            let to = message
                .get("to")
                .and_then(Value::as_str)
                .ok_or_else(|| "email send requires field 'to'".to_string())?;
            if to.is_empty() {
                return Err("email send requires non-empty 'to'".into());
            }
            let id = format!("email-{}", mailbox.len() + 1);
            let mut stored = message;
            stored.insert("id".into(), Value::String(id.clone()));
            mailbox.push(Value::Object(stored));
            Ok(Value::String(id))
        }
        "list" => {
            if !call.input.is_null() {
                return Err("email list requires unit".into());
            }
            let summaries = mailbox
                .iter()
                .filter_map(|entry| {
                    entry.as_object().map(|message| {
                        Value::String(format!(
                            "{}:{}",
                            message.get("to").and_then(Value::as_str).unwrap_or(""),
                            message.get("subject").and_then(Value::as_str).unwrap_or("")
                        ))
                    })
                })
                .collect::<Vec<_>>();
            Ok(Value::Array(summaries))
        }
        operation => Err(format!(
            "email does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_pdf_call(
    pdfs: &mut BTreeMap<String, Value>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    match call.operation {
        "render" => {
            let document = call
                .input
                .as_object()
                .ok_or_else(|| "pdf render requires PdfDocument object".to_string())?;
            let id = document
                .get("id")
                .map(json_scalar_to_string)
                .ok_or_else(|| "pdf render requires field 'id'".to_string())?;
            if id.is_empty() {
                return Err("pdf render requires non-empty 'id'".into());
            }
            let title = document
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("document");
            let totale = document
                .get("totale")
                .map(json_scalar_to_string)
                .unwrap_or_default();
            let reference = format!("pdf-{id}");
            let rendered = format!("%PDF-1.4 stub\n% {title}\n{totale}");
            pdfs.insert(reference.clone(), Value::String(rendered));
            Ok(Value::String(reference))
        }
        "get" => {
            let reference = string_input(&call.input, "get")?;
            pdfs.get(reference)
                .cloned()
                .ok_or_else(|| "pdf_not_found".to_string())
        }
        operation => Err(format!(
            "pdf does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_metrics_call(
    metrics: &mut BTreeMap<String, BTreeMap<String, i64>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let counters = metrics.entry(call.provider.to_string()).or_default();
    match call.operation {
        "increment" => {
            let key = string_input(&call.input, "increment")?;
            let value = counters.entry(key.to_string()).or_insert(0);
            *value += 1;
            Ok(Value::Number((*value).into()))
        }
        "get" => {
            let key = string_input(&call.input, "get")?;
            let value = counters.get(key).copied().unwrap_or(0);
            Ok(Value::Number(value.into()))
        }
        operation => Err(format!(
            "metrics does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_tracer_call(
    tracers: &mut BTreeMap<String, TracerState>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let tracer = tracers.entry(call.provider.to_string()).or_default();
    match call.operation {
        "start" => {
            let name = string_input(&call.input, "start")?.to_string();
            tracer.next_id += 1;
            let id = format!("span-{}", tracer.next_id);
            tracer.open.insert(id.clone(), name);
            Ok(Value::String(id))
        }
        "finish" => {
            let id = string_input(&call.input, "finish")?.to_string();
            let name = tracer
                .open
                .remove(&id)
                .ok_or_else(|| "span_not_found".to_string())?;
            tracer.finished.push(Value::String(name));
            Ok(Value::Null)
        }
        "list" => {
            if !call.input.is_null() {
                return Err("tracer list requires unit".into());
            }
            Ok(Value::Array(tracer.finished.clone()))
        }
        operation => Err(format!(
            "tracer does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn memory_job_store_call(
    stores: &mut BTreeMap<String, BTreeMap<String, Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
    let store = stores.entry(call.provider.to_string()).or_default();
    match call.operation {
        "enqueue" => {
            let envelope = job_envelope_from_text(&call.input)?;
            let id = envelope_id(&envelope)?;
            if store.contains_key(&id) {
                return Ok(Value::String(id));
            }
            store.insert(id.clone(), envelope);
            Ok(Value::String(id))
        }
        "claim" => {
            if !call.input.is_null() {
                return Err("job store claim requires unit".into());
            }
            let now = now_millis();
            let mut due = Vec::new();
            let mut remaining = BTreeMap::new();
            for (id, envelope) in std::mem::take(store) {
                let run_at = envelope
                    .get("run_at")
                    .and_then(Value::as_u64)
                    .unwrap_or(u64::MAX);
                if run_at <= now {
                    let text =
                        serde_json::to_string(&envelope).map_err(|error| error.to_string())?;
                    due.push(Value::String(text));
                } else {
                    remaining.insert(id, envelope);
                }
            }
            *store = remaining;
            Ok(Value::Array(due))
        }
        "finish" => {
            let id = string_input(&call.input, "finish")?;
            store.remove(id);
            Ok(Value::String(id.to_string()))
        }
        operation => Err(format!(
            "job store does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn sqlite_job_store_call(connection: &Connection, call: ProviderCall<'_>) -> Result<Value, String> {
    match call.operation {
        "enqueue" => {
            let envelope = job_envelope_from_text(&call.input)?;
            let id = envelope_id(&envelope)?;
            let run_at = envelope
                .get("run_at")
                .and_then(Value::as_u64)
                .ok_or_else(|| "job envelope requires run_at".to_string())?;
            let payload = serde_json::to_string(&envelope).map_err(|error| error.to_string())?;
            let existing = connection.query_row(
                "SELECT job_id FROM axl_jobs WHERE provider = ?1 AND job_id = ?2",
                params![call.provider, id],
                |row| row.get::<_, String>(0),
            );
            match existing {
                Ok(existing_id) => Ok(Value::String(existing_id)),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    connection
                        .execute(
                            "INSERT INTO axl_jobs (provider, job_id, envelope, run_at) \
                             VALUES (?1, ?2, ?3, ?4)",
                            params![call.provider, id, payload, run_at as i64],
                        )
                        .map_err(|error| error.to_string())?;
                    Ok(Value::String(id))
                }
                Err(error) => Err(error.to_string()),
            }
        }
        "claim" => {
            if !call.input.is_null() {
                return Err("job store claim requires unit".into());
            }
            let now = now_millis() as i64;
            let mut statement = connection
                .prepare(
                    "SELECT job_id, envelope FROM axl_jobs \
                     WHERE provider = ?1 AND run_at <= ?2 ORDER BY run_at, job_id",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(params![call.provider, now], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| error.to_string())?;
            let mut due = Vec::new();
            let mut ids = Vec::new();
            for row in rows {
                let (id, envelope) = row.map_err(|error| error.to_string())?;
                ids.push(id);
                due.push(Value::String(envelope));
            }
            for id in ids {
                connection
                    .execute(
                        "DELETE FROM axl_jobs WHERE provider = ?1 AND job_id = ?2",
                        params![call.provider, id],
                    )
                    .map_err(|error| error.to_string())?;
            }
            Ok(Value::Array(due))
        }
        "finish" => {
            let id = string_input(&call.input, "finish")?;
            connection
                .execute(
                    "DELETE FROM axl_jobs WHERE provider = ?1 AND job_id = ?2",
                    params![call.provider, id],
                )
                .map_err(|error| error.to_string())?;
            Ok(Value::String(id.to_string()))
        }
        operation => Err(format!(
            "job store does not implement operation '{operation}' for {}",
            call.capacity
        )),
    }
}

fn job_envelope_from_text(value: &Value) -> Result<Value, String> {
    let text = value
        .as_str()
        .ok_or_else(|| "job store enqueue requires text".to_string())?;
    serde_json::from_str(text).map_err(|error| error.to_string())
}

fn envelope_id(envelope: &Value) -> Result<String, String> {
    envelope
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "job envelope requires id".into())
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

fn new_job_id(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0)
    )
}

fn job_node<'a>(graph: &'a GraphIr, job_name: &str) -> Result<&'a GraphNode, RuntimeError> {
    graph
        .nodes
        .iter()
        .find(|node| node.kind == "job" && node.name == job_name)
        .ok_or_else(|| RuntimeError(format!("unknown job '{job_name}'")))
}

fn invoke_job_store(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    job: &GraphNode,
    operation: &str,
    input: Value,
) -> Result<Value, RuntimeError> {
    let capacity_name = job
        .type_name
        .as_deref()
        .ok_or_else(|| RuntimeError(format!("job '{}' has no store capacity", job.name)))?;
    let provider_name = metadata(job, "provider")?;
    let provider_id = format!("skill.{provider_name}");
    let provider = graph
        .nodes
        .iter()
        .find(|node| node.id == provider_id)
        .ok_or_else(|| RuntimeError(format!("missing job store provider '{provider_name}'")))?;
    let implementation = provider.implementation.as_deref().ok_or_else(|| {
        RuntimeError(format!(
            "provider '{}' has no native binding",
            provider.name
        ))
    })?;
    runtime
        .invoke(ProviderCall {
            provider: &provider.name,
            capacity: capacity_name,
            implementation,
            operation,
            config: provider_config(graph, &provider.id)?,
            input,
        })
        .map_err(RuntimeError)
}

fn enqueue_job(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
    job_name: &str,
    payload: Value,
    id: Option<String>,
    attempt: u32,
    run_at: u64,
) -> Result<String, RuntimeError> {
    let job = job_node(graph, job_name)?;
    let id = id.unwrap_or_else(|| new_job_id(job_name));
    let envelope = json!({
        "id": id,
        "job": job_name,
        "payload": payload,
        "attempt": attempt,
        "run_at": run_at,
    });
    let text = serde_json::to_string(&envelope)
        .map_err(|error| RuntimeError(format!("cannot encode job envelope: {error}")))?;
    let result = invoke_job_store(graph, runtime, job, "enqueue", Value::String(text))?;
    result
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| RuntimeError("job enqueue must return text id".into()))
}

fn ensure_scheduled_jobs(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
) -> Result<usize, RuntimeError> {
    let mut ensured = 0usize;
    for job in graph.nodes.iter().filter(|node| node.kind == "job") {
        let Some(schedule) = job.metadata.get("schedule") else {
            continue;
        };
        let Some(_interval) = super::analyzer::parse_schedule_millis(schedule) else {
            return Err(RuntimeError(format!(
                "job '{}' has invalid schedule '{schedule}'",
                job.name
            )));
        };
        let id = format!("schedule:{}", job.name);
        enqueue_job(
            graph,
            runtime,
            &job.name,
            Value::Null,
            Some(id),
            0,
            now_millis(),
        )?;
        ensured += 1;
    }
    Ok(ensured)
}

/// Register due scheduled jobs, claim pending work and execute bound flows with retry.
pub fn run_due_jobs(
    graph: &GraphIr,
    runtime: &mut dyn ProviderRuntime,
) -> Result<usize, RuntimeError> {
    ensure_scheduled_jobs(graph, runtime)?;
    let mut providers = BTreeMap::new();
    for job in graph.nodes.iter().filter(|node| node.kind == "job") {
        let provider = metadata(job, "provider")?;
        providers
            .entry(provider.to_string())
            .or_insert_with(|| job.clone());
    }
    let mut executed = 0usize;
    for job in providers.values() {
        let claimed = invoke_job_store(graph, runtime, job, "claim", Value::Null)?;
        let envelopes = claimed
            .as_array()
            .ok_or_else(|| RuntimeError("job claim must return a list".into()))?
            .clone();
        for envelope_text in envelopes {
            let text = envelope_text
                .as_str()
                .ok_or_else(|| RuntimeError("job claim entries must be text".into()))?;
            let envelope: Value = serde_json::from_str(text)
                .map_err(|error| RuntimeError(format!("invalid job envelope: {error}")))?;
            let job_name = envelope
                .get("job")
                .and_then(Value::as_str)
                .ok_or_else(|| RuntimeError("job envelope missing job".into()))?
                .to_string();
            let target = job_node(graph, &job_name)?;
            let id = envelope
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| RuntimeError("job envelope missing id".into()))?
                .to_string();
            let attempt = envelope.get("attempt").and_then(Value::as_u64).unwrap_or(0) as u32;
            let payload = envelope.get("payload").cloned().unwrap_or(Value::Null);
            let flow = metadata(target, "flow")?;
            let retry = metadata(target, "retry")?
                .parse::<u32>()
                .map_err(|_| RuntimeError(format!("job '{job_name}' has invalid retry")))?;
            let result = evaluate_flow_inner(graph, flow, payload.clone(), runtime, 0);
            let failed = match &result {
                Ok(value) => value
                    .as_object()
                    .is_some_and(|object| object.contains_key("error")),
                Err(_) => true,
            };
            let _ = invoke_job_store(graph, runtime, target, "finish", Value::String(id.clone()));
            if failed {
                if attempt < retry {
                    let delay = 10u64.saturating_mul(1u64 << attempt.min(10));
                    enqueue_job(
                        graph,
                        runtime,
                        &job_name,
                        payload,
                        Some(new_job_id(&job_name)),
                        attempt + 1,
                        now_millis().saturating_add(delay),
                    )?;
                }
            } else {
                executed += 1;
                if let Some(schedule) = target.metadata.get("schedule") {
                    let interval =
                        super::analyzer::parse_schedule_millis(schedule).ok_or_else(|| {
                            RuntimeError(format!(
                                "job '{job_name}' has invalid schedule '{schedule}'"
                            ))
                        })?;
                    enqueue_job(
                        graph,
                        runtime,
                        &job_name,
                        Value::Null,
                        Some(format!("schedule:{job_name}")),
                        0,
                        now_millis().saturating_add(interval),
                    )?;
                }
            }
        }
    }
    Ok(executed)
}

fn initialize_sqlite(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS axl_records (\
             provider TEXT NOT NULL, \
             record_id TEXT NOT NULL, \
             payload TEXT NOT NULL, \
             PRIMARY KEY (provider, record_id));\
             CREATE TABLE IF NOT EXISTS axl_jobs (\
             provider TEXT NOT NULL, \
             job_id TEXT NOT NULL, \
             envelope TEXT NOT NULL, \
             run_at INTEGER NOT NULL, \
             PRIMARY KEY (provider, job_id));\
             CREATE TABLE IF NOT EXISTS axl_cache (\
             provider TEXT NOT NULL, \
             cache_key TEXT NOT NULL, \
             value TEXT NOT NULL, \
             PRIMARY KEY (provider, cache_key));\
             CREATE TABLE IF NOT EXISTS axl_schema_history (\
             provider TEXT NOT NULL, \
             version TEXT NOT NULL, \
             seq INTEGER NOT NULL, \
             PRIMARY KEY (provider, version));\
             CREATE UNIQUE INDEX IF NOT EXISTS axl_schema_history_seq \
             ON axl_schema_history (provider, seq);",
        )
        .map_err(|error| format!("cannot initialize SQLite schema: {error}"))
}

fn record_id(value: &Value) -> Result<String, String> {
    value
        .as_object()
        .and_then(|object| object.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "store save requires a string 'id' field".into())
}

fn json_scalar_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn string_input<'a>(value: &'a Value, operation: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("store {operation} requires a string id"))
}

fn validate_value(
    graph: &GraphIr,
    type_name: &str,
    value: &Value,
    path: &str,
) -> Result<(), RuntimeError> {
    if let Some(inner) = generic(type_name, "Option") {
        if value.is_null() {
            return Ok(());
        }
        return validate_value(graph, inner, value, path);
    }
    if let Some(inner) = generic(type_name, "List").or_else(|| generic(type_name, "Set")) {
        let values = value
            .as_array()
            .ok_or_else(|| RuntimeError(format!("{path} must be an array of {inner}")))?;
        for (index, value) in values.iter().enumerate() {
            validate_value(graph, inner, value, &format!("{path}[{index}]"))?;
        }
        if generic(type_name, "Set").is_some() {
            for (index, value) in values.iter().enumerate() {
                if values[..index].contains(value) {
                    return Err(RuntimeError(format!(
                        "{path} contains duplicate set values"
                    )));
                }
            }
        }
        return Ok(());
    }
    if let Some((key_type, value_type)) = generic_pair(type_name, "Map") {
        let object = value
            .as_object()
            .ok_or_else(|| RuntimeError(format!("{path} must be a map")))?;
        for (key, value) in object {
            validate_value(
                graph,
                key_type,
                &Value::String(key.clone()),
                &format!("{path}.key"),
            )?;
            validate_value(graph, value_type, value, &format!("{path}.{key}"))?;
        }
        return Ok(());
    }
    match type_name {
        "unit" if value.is_null() => return Ok(()),
        "bool" if value.is_boolean() => return Ok(()),
        "int" if value.as_i64().is_some() => return Ok(()),
        "float" | "money" if value.is_number() => return Ok(()),
        "text" | "string" | "email" | "uuid" | "datetime" | "duration" if value.is_string() => {
            return Ok(());
        }
        _ => {}
    }

    if let Some(value_enum) = graph
        .nodes
        .iter()
        .find(|node| node.kind == "enum" && node.name == type_name)
    {
        let Some(variant) = value.as_str() else {
            return Err(RuntimeError(format!("{path} must be enum {type_name}")));
        };
        if children(graph, &value_enum.id, "variant")
            .iter()
            .any(|node| node.name == variant)
        {
            return Ok(());
        }
        return Err(RuntimeError(format!(
            "{path} has unknown {type_name} variant '{variant}'"
        )));
    }

    if let Some(entity) = graph
        .nodes
        .iter()
        .find(|node| node.kind == "entity" && node.name == type_name)
    {
        let object = value
            .as_object()
            .ok_or_else(|| RuntimeError(format!("{path} must be object {type_name}")))?;
        for field in children(graph, &entity.id, "field") {
            let field_type = field.type_name.as_deref().unwrap_or("unit");
            let qualifiers = field
                .metadata
                .get("qualifiers")
                .map(|value| value.split(',').collect::<Vec<_>>())
                .unwrap_or_default();
            match object.get(&field.name) {
                Some(value) => {
                    validate_value(graph, field_type, value, &format!("{path}.{}", field.name))?
                }
                None if qualifiers.contains(&"optional")
                    || generic(field_type, "Option").is_some() => {}
                None => {
                    return Err(RuntimeError(format!(
                        "{path} is missing field '{}.{}'",
                        entity.name, field.name
                    )));
                }
            }
        }
        return Ok(());
    }

    Err(RuntimeError(format!(
        "{path} does not match AXL type '{type_name}'"
    )))
}

fn enum_values(graph: &GraphIr) -> BTreeMap<String, Value> {
    graph
        .nodes
        .iter()
        .filter(|node| node.kind == "enum")
        .map(|value| {
            let variants = children(graph, &value.id, "variant")
                .into_iter()
                .map(|variant| (variant.name.clone(), Value::String(variant.name.clone())))
                .collect::<Map<_, _>>();
            (value.name.clone(), Value::Object(variants))
        })
        .collect()
}

fn ordered_children<'a>(graph: &'a GraphIr, owner: &str) -> Vec<&'a GraphNode> {
    let mut values = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == owner)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| {
            matches!(
                node.kind.as_str(),
                "let"
                    | "require"
                    | "call"
                    | "attempt"
                    | "make"
                    | "fold"
                    | "run"
                    | "match"
                    | "map"
                    | "filter"
                    | "sort"
                    | "group"
                    | "parallel"
                    | "race"
                    | "emit"
                    | "enqueue"
                    | "return"
            )
        })
        .collect::<Vec<_>>();
    values.sort_by_key(|node| {
        node.metadata
            .get("order")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX)
    });
    values
}

fn children<'a>(graph: &'a GraphIr, owner: &str, kind: &str) -> Vec<&'a GraphNode> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == owner)
        .filter_map(|edge| graph.nodes.iter().find(|node| node.id == edge.to))
        .filter(|node| node.kind == kind)
        .collect()
}

fn provider_for(graph: &GraphIr, dependency: &str) -> Option<String> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.from == dependency)
        .find(|edge| edge.kind == "bind")
        .or_else(|| {
            graph
                .edges
                .iter()
                .find(|edge| edge.from == dependency && edge.kind == "default")
        })
        .map(|edge| edge.to.clone())
}

pub(crate) fn provider_config(
    graph: &GraphIr,
    provider: &str,
) -> Result<BTreeMap<String, Value>, RuntimeError> {
    children(graph, provider, "config")
        .into_iter()
        .map(|config| {
            let raw = config
                .metadata
                .get("value")
                .ok_or_else(|| RuntimeError(format!("{} has no config value", config.id)))?;
            let value = serde_json::from_str(raw)
                .map_err(|error| RuntimeError(format!("{}: {error}", config.id)))?;
            Ok((config.name.clone(), value))
        })
        .collect()
}

struct OwnedProviderCall {
    provider: String,
    capacity: String,
    implementation: String,
    operation: String,
    config: BTreeMap<String, Value>,
    input: Value,
}

fn invoke_with_resilience(
    runtime: &mut dyn ProviderRuntime,
    call: OwnedProviderCall,
    retry: u32,
    timeout_ms: u64,
) -> Result<Value, String> {
    let mut last_error = "attempt did not run".to_string();
    for _ in 0..=retry {
        let mut worker = runtime.fork()?;
        let provider = call.provider.clone();
        let capacity = call.capacity.clone();
        let implementation = call.implementation.clone();
        let operation = call.operation.clone();
        let config = call.config.clone();
        let input = call.input.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let result = worker.invoke(ProviderCall {
                provider: &provider,
                capacity: &capacity,
                implementation: &implementation,
                operation: &operation,
                config,
                input,
            });
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(message)) => last_error = message,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                last_error = format!("timeout_after_{timeout_ms}ms")
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                last_error = "provider_worker_disconnected".into()
            }
        }
    }
    Err(last_error)
}

fn generic<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value
        .strip_prefix(name)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

fn generic_pair<'a>(value: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let inner = generic(value, name)?;
    let mut depth = 0usize;
    for (index, character) in inner.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                let left = inner[..index].trim();
                let right = inner[index + 1..].trim();
                return (!left.is_empty() && !right.is_empty()).then_some((left, right));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::compile_source;

    const SOURCE: &str = r#"axl 4
app CashflowCore
enum MovementKind
  income
  expense
entity Movement
  id: uuid required
  kind: MovementKind required
  amount: money required
entity BalanceInput
  income: money required
  expense: money required
capacity Echo
  op invoke Movement -> Result<Movement>
skill EchoSkill provides Echo
  native rust test::echo
flow ValidateMovement Movement -> Result<Movement>
  let positive = input.amount > 0
  require positive else "amount_must_be_positive"
  return input
flow CalculateBalance BalanceInput -> money
  let balance = input.income - input.expense
  return balance
flow EchoMovement Movement -> Result<Movement>
  in echo: Echo = EchoSkill
  call output = echo.invoke(input)?
  return output
"#;

    #[test]
    fn executes_validation_and_result_propagation() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let accepted = evaluate_flow(
            &graph,
            "ValidateMovement",
            json!({"id": "m1", "kind": "income", "amount": 25}),
        )
        .unwrap();
        assert_eq!(accepted["ok"]["amount"], 25);

        let rejected = evaluate_flow(
            &graph,
            "ValidateMovement",
            json!({"id": "m2", "kind": "expense", "amount": 0}),
        )
        .unwrap();
        assert_eq!(rejected, json!({"error": "amount_must_be_positive"}));
    }

    #[test]
    fn executes_money_arithmetic() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let result = evaluate_flow(
            &graph,
            "CalculateBalance",
            json!({"income": 120, "expense": 45}),
        )
        .unwrap();
        assert_eq!(result, 75);
    }

    #[test]
    fn executes_a_replaceable_capacity_runtime() {
        struct EchoRuntime;

        impl ProviderRuntime for EchoRuntime {
            fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
                assert_eq!(call.capacity, "Echo");
                assert_eq!(call.operation, "invoke");
                Ok(call.input)
            }
        }

        let graph = compile_source(SOURCE).unwrap().graph;
        let movement = json!({"id": "m3", "kind": "income", "amount": 80});
        let result =
            evaluate_flow_with_runtime(&graph, "EchoMovement", movement, &mut EchoRuntime).unwrap();
        assert_eq!(result["ok"]["id"], "m3");
    }

    #[test]
    fn executes_parallel_workers_concurrently() {
        const PARALLEL_SOURCE: &str = r#"axl 4
app ParallelDemo
entity Batch
  values: List<int> required
capacity Delay
  op wait int -> int
skill DelaySkill provides Delay
  native rust test::delay
flow Wait int -> int
  in delay: Delay = DelaySkill
  call output = delay.wait(input)
  return output
flow Concurrent Batch -> List<int>
  parallel output: List<int> = input.values as item
    run = Wait(item)
  return output
"#;

        #[derive(Clone)]
        struct ConcurrentRuntime {
            active: Arc<std::sync::atomic::AtomicUsize>,
            maximum: Arc<std::sync::atomic::AtomicUsize>,
        }

        impl ProviderRuntime for ConcurrentRuntime {
            fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
                let active = self
                    .active
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                    + 1;
                self.maximum
                    .fetch_max(active, std::sync::atomic::Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(30));
                self.active
                    .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                Ok(call.input)
            }

            fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
                Ok(Box::new(self.clone()))
            }
        }

        let graph = compile_source(PARALLEL_SOURCE).unwrap().graph;
        let runtime = ConcurrentRuntime {
            active: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            maximum: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };
        let maximum = runtime.maximum.clone();
        let mut runtime = runtime;
        let result = evaluate_flow_with_runtime(
            &graph,
            "Concurrent",
            json!({"values": [1, 2, 3, 4]}),
            &mut runtime,
        )
        .unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
        assert!(maximum.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[test]
    fn retries_transient_errors_and_enforces_timeouts() {
        const ATTEMPT_SOURCE: &str = r#"axl 4
app AttemptDemo
capacity Remote
  op fetch int -> Result<int> idempotent
skill RemoteSkill provides Remote
  native rust test::remote
flow Retry int -> Result<int>
  in remote: Remote = RemoteSkill
  attempt output = remote.fetch(input)?
    retry = 2
    timeout_ms = 100
  return output
flow Timeout int -> Result<int>
  in remote: Remote = RemoteSkill
  attempt output = remote.fetch(input)?
    retry = 1
    timeout_ms = 5
  return output
"#;

        #[derive(Clone)]
        struct RetryRuntime {
            attempts: Arc<std::sync::atomic::AtomicUsize>,
        }

        impl ProviderRuntime for RetryRuntime {
            fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
                let attempt = self
                    .attempts
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt < 2 {
                    Err("transient".into())
                } else {
                    Ok(call.input)
                }
            }

            fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
                Ok(Box::new(self.clone()))
            }
        }

        #[derive(Clone)]
        struct SlowRuntime;

        impl ProviderRuntime for SlowRuntime {
            fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
                std::thread::sleep(std::time::Duration::from_millis(30));
                Ok(call.input)
            }

            fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
                Ok(Box::new(self.clone()))
            }
        }

        let graph = compile_source(ATTEMPT_SOURCE).unwrap().graph;
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut retry_runtime = RetryRuntime {
            attempts: attempts.clone(),
        };
        let retried =
            evaluate_flow_with_runtime(&graph, "Retry", json!(42), &mut retry_runtime).unwrap();
        assert_eq!(retried, json!({"ok": 42}));
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);

        let mut slow_runtime = SlowRuntime;
        let timed_out =
            evaluate_flow_with_runtime(&graph, "Timeout", json!(42), &mut slow_runtime).unwrap();
        assert_eq!(timed_out, json!({"error": "timeout_after_5ms"}));
    }

    #[test]
    fn race_returns_the_first_successful_worker() {
        const RACE_SOURCE: &str = r#"axl 4
app RaceDemo
entity Batch
  values: List<int> required
capacity Remote
  op fetch int -> Result<int> idempotent
skill RemoteSkill provides Remote
  native rust test::remote
flow Fetch int -> Result<int>
  in remote: Remote = RemoteSkill
  call output = remote.fetch(input)?
  return output
flow Fastest Batch -> Result<int>
  race output: int = input.values as item
    run = Fetch(item)?
  return output
"#;

        #[derive(Clone)]
        struct VariableDelayRuntime;

        impl ProviderRuntime for VariableDelayRuntime {
            fn invoke(&mut self, call: ProviderCall<'_>) -> Result<Value, String> {
                let delay = if call.input == json!(1) { 50 } else { 5 };
                std::thread::sleep(std::time::Duration::from_millis(delay));
                Ok(call.input)
            }

            fn fork(&self) -> Result<Box<dyn ProviderRuntime>, String> {
                Ok(Box::new(self.clone()))
            }
        }

        let graph = compile_source(RACE_SOURCE).unwrap().graph;
        let mut runtime = VariableDelayRuntime;
        let result =
            evaluate_flow_with_runtime(&graph, "Fastest", json!({"values": [1, 2]}), &mut runtime)
                .unwrap();
        assert_eq!(result, json!({"ok": 2}));
    }

    #[test]
    fn sqlite_config_persists_across_independent_runtimes() {
        let database = std::env::temp_dir().join(format!(
            "axl-durable-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableStore
entity Record
  id: uuid required
  value: text required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
skill DurableRecords provides RecordStore
  native rust axl::store::sqlite
  config path: text = {path}
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = DurableRecords
  call saved = store.save(input)?
  return saved
flow Find uuid -> Result<Record>
  in store: RecordStore = DurableRecords
  call found = store.find(input)?
  return found
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let record = json!({"id": "record-1", "value": "survives restart"});

        {
            let mut first_runtime = BuiltinRuntime::new().unwrap();
            let saved =
                evaluate_flow_with_runtime(&graph, "Save", record.clone(), &mut first_runtime)
                    .unwrap();
            assert_eq!(saved, json!({"ok": record}));
        }

        let mut second_runtime = BuiltinRuntime::new().unwrap();
        let found =
            evaluate_flow_with_runtime(&graph, "Find", json!("record-1"), &mut second_runtime)
                .unwrap();
        assert_eq!(found["ok"]["value"], "survives restart");
        drop(second_runtime);
        std::fs::remove_file(database).unwrap();
    }

    #[test]
    fn emit_invokes_subscribers_in_declaration_order() {
        const SOURCE: &str = r#"axl 4
app EventDemo
entity Item
  id: text required
capacity EventLog
  op append text -> Result<text>
  op list unit -> Result<List<text>>
skill MemoryEventLog provides EventLog
  native rust axl::event::log
event ItemSaved: Item
flow TagPersisted Item -> Result<text>
  in log: EventLog = MemoryEventLog
  call tagged = log.append("persisted")?
  return tagged
flow TagAnnounced Item -> Result<text>
  in log: EventLog = MemoryEventLog
  call tagged = log.append("announced")?
  return tagged
on ItemSaved Item = TagPersisted
on ItemSaved Item = TagAnnounced
flow SaveAndAnnounce Item -> Result<Item>
  emit ItemSaved(input)
  return input
flow ReadTags unit -> Result<List<text>>
  in log: EventLog = MemoryEventLog
  call tags = log.list(input)?
  return tags
"#;

        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let saved = evaluate_flow_with_runtime(
            &graph,
            "SaveAndAnnounce",
            json!({"id": "item-1"}),
            &mut runtime,
        )
        .unwrap();
        assert_eq!(saved, json!({"ok": {"id": "item-1"}}));
        let tags =
            evaluate_flow_with_runtime(&graph, "ReadTags", Value::Null, &mut runtime).unwrap();
        assert_eq!(tags, json!({"ok": ["persisted", "announced"]}));
    }

    #[test]
    fn durable_jobs_survive_independent_runtimes_with_retry() {
        let jobs_db = std::env::temp_dir().join(format!(
            "axl-jobs-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let records_db = std::env::temp_dir().join(format!(
            "axl-job-records-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let jobs_path = serde_json::to_string(jobs_db.to_str().unwrap()).unwrap();
        let records_path = serde_json::to_string(records_db.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableJobs
entity Record
  id: uuid required
  value: text required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
capacity JobStore
  op enqueue text -> Result<text> idempotent
  op claim unit -> Result<List<text>> idempotent
  op finish text -> Result<text>
skill DurableRecords provides RecordStore
  native rust axl::store::sqlite
  config path: text = {records_path}
  effect db.read
  effect db.write
skill DurableJobStore provides JobStore
  native rust axl::job::sqlite
  config path: text = {jobs_path}
flow Save Record -> Result<Record>
  in store: RecordStore = DurableRecords
  call saved = store.save(input)?
  return saved
flow Find uuid -> Result<Record>
  in store: RecordStore = DurableRecords
  call found = store.find(input)?
  return found
job PersistRecord
  run Save
  retry 3
  idempotent
  in store: JobStore = DurableJobStore
flow Schedule Record -> Result<Record>
  enqueue PersistRecord(input)
  return input
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let record = json!({"id": "record-1", "value": "from-job"});

        {
            let mut first = BuiltinRuntime::new().unwrap();
            let scheduled =
                evaluate_flow_with_runtime(&graph, "Schedule", record.clone(), &mut first).unwrap();
            assert_eq!(scheduled, json!({"ok": record}));
        }

        let mut second = BuiltinRuntime::new().unwrap();
        let executed = run_due_jobs(&graph, &mut second).unwrap();
        assert_eq!(executed, 1);
        let found =
            evaluate_flow_with_runtime(&graph, "Find", json!("record-1"), &mut second).unwrap();
        assert_eq!(found["ok"]["value"], "from-job");
        drop(second);
        let _ = std::fs::remove_file(jobs_db);
        let _ = std::fs::remove_file(records_db);
    }

    #[test]
    fn scheduled_jobs_run_and_requeue() {
        const SOURCE: &str = r#"axl 4
app ScheduledJobs
capacity EventLog
  op append text -> Result<text>
  op list unit -> Result<List<text>>
capacity JobStore
  op enqueue text -> Result<text> idempotent
  op claim unit -> Result<List<text>> idempotent
  op finish text -> Result<text>
skill MemoryEventLog provides EventLog
  native rust axl::event::log
skill MemoryJobs provides JobStore
  native rust axl::job::memory
flow Tick unit -> Result<text>
  in log: EventLog = MemoryEventLog
  call tagged = log.append("tick")?
  return tagged
flow Read unit -> Result<List<text>>
  in log: EventLog = MemoryEventLog
  call tags = log.list(input)?
  return tags
job TickJob
  schedule "every 60s"
  run Tick
  retry 1
  idempotent
  in store: JobStore = MemoryJobs
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let first = run_due_jobs(&graph, &mut runtime).unwrap();
        assert_eq!(first, 1);
        let tags = evaluate_flow_with_runtime(&graph, "Read", Value::Null, &mut runtime).unwrap();
        assert_eq!(tags, json!({"ok": ["tick"]}));
        let second = run_due_jobs(&graph, &mut runtime).unwrap();
        assert_eq!(second, 0);
    }

    #[test]
    fn memory_cache_put_get_and_invalidate() {
        const SOURCE: &str = r#"axl 4
app MemoryCacheDemo
entity CacheEntry
  key: text required
  value: text required
capacity Cache
  op get text -> Result<text> idempotent
  op put CacheEntry -> Result<unit>
  op invalidate text -> Result<bool>
skill MemoryCache provides Cache
  native rust axl::cache::memory
flow PutAndGet CacheEntry -> Result<text>
  in cache: Cache = MemoryCache
  call stored = cache.put(input)?
  call loaded = cache.get(input.key)?
  return loaded
flow Get text -> Result<text>
  in cache: Cache = MemoryCache
  call loaded = cache.get(input)?
  return loaded
flow Invalidate text -> Result<bool>
  in cache: Cache = MemoryCache
  call removed = cache.invalidate(input)?
  return removed
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let entry = json!({"key": "ledger:demo", "value": "80000"});
        let loaded = evaluate_flow_with_runtime(&graph, "PutAndGet", entry, &mut runtime).unwrap();
        assert_eq!(loaded, json!({"ok": "80000"}));
        let hit =
            evaluate_flow_with_runtime(&graph, "Get", json!("ledger:demo"), &mut runtime).unwrap();
        assert_eq!(hit, json!({"ok": "80000"}));
        let removed =
            evaluate_flow_with_runtime(&graph, "Invalidate", json!("ledger:demo"), &mut runtime)
                .unwrap();
        assert_eq!(removed, json!({"ok": true}));
        let miss =
            evaluate_flow_with_runtime(&graph, "Get", json!("ledger:demo"), &mut runtime).unwrap();
        assert_eq!(miss, json!({"error": "cache_miss"}));
        let again =
            evaluate_flow_with_runtime(&graph, "Invalidate", json!("ledger:demo"), &mut runtime)
                .unwrap();
        assert_eq!(again, json!({"ok": false}));
    }

    #[test]
    fn durable_cache_survives_runtime_recreate() {
        let cache_db = std::env::temp_dir().join(format!(
            "axl-cache-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache_path = serde_json::to_string(cache_db.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableCacheDemo
entity CacheEntry
  key: text required
  value: text required
capacity Cache
  op get text -> Result<text> idempotent
  op put CacheEntry -> Result<unit>
  op invalidate text -> Result<bool>
skill DurableCache provides Cache
  native rust axl::cache::sqlite
  config path: text = {cache_path}
flow Put CacheEntry -> Result<unit>
  in cache: Cache = DurableCache
  call stored = cache.put(input)?
  return stored
flow Get text -> Result<text>
  in cache: Cache = DurableCache
  call loaded = cache.get(input)?
  return loaded
flow Invalidate text -> Result<bool>
  in cache: Cache = DurableCache
  call removed = cache.invalidate(input)?
  return removed
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let entry = json!({"key": "ledger:demo", "value": "80000"});

        {
            let mut first = BuiltinRuntime::new().unwrap();
            let stored =
                evaluate_flow_with_runtime(&graph, "Put", entry.clone(), &mut first).unwrap();
            assert_eq!(stored, json!({"ok": null}));
        }

        let mut second = BuiltinRuntime::new().unwrap();
        let found =
            evaluate_flow_with_runtime(&graph, "Get", json!("ledger:demo"), &mut second).unwrap();
        assert_eq!(found, json!({"ok": "80000"}));
        let removed =
            evaluate_flow_with_runtime(&graph, "Invalidate", json!("ledger:demo"), &mut second)
                .unwrap();
        assert_eq!(removed, json!({"ok": true}));
        let miss =
            evaluate_flow_with_runtime(&graph, "Get", json!("ledger:demo"), &mut second).unwrap();
        assert_eq!(miss, json!({"error": "cache_miss"}));
        drop(second);
        let _ = std::fs::remove_file(cache_db);
    }

    #[test]
    fn memory_observability_logs_metrics_and_spans() {
        const SOURCE: &str = r#"axl 4
app ObservabilityDemo
capacity Logger
  op write text -> Result<unit>
  op list unit -> Result<List<text>>
skill MemoryLogger provides Logger
  native rust axl::telemetry::logger
capacity Metrics
  op increment text -> Result<int>
  op get text -> Result<int> idempotent
skill MemoryMetrics provides Metrics
  native rust axl::telemetry::metrics
capacity Tracer
  op start text -> Result<text>
  op finish text -> Result<unit>
  op list unit -> Result<List<text>>
skill MemoryTracer provides Tracer
  native rust axl::telemetry::tracer
flow RecordTwo unit -> Result<List<text>>
  in logger: Logger = MemoryLogger
  call first = logger.write("ledger.balance")?
  call second = logger.write("ledger.balance")?
  call lines = logger.list(input)?
  return lines
flow ObserveTwice unit -> Result<int>
  in metrics: Metrics = MemoryMetrics
  call first = metrics.increment("ledger.balance")?
  call second = metrics.increment("ledger.balance")?
  call value = metrics.get("ledger.balance")?
  return value
flow TraceOnce unit -> Result<List<text>>
  in tracer: Tracer = MemoryTracer
  call span = tracer.start("CalculateLedgerBalance")?
  call done = tracer.finish(span)?
  call spans = tracer.list(input)?
  return spans
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let lines =
            evaluate_flow_with_runtime(&graph, "RecordTwo", Value::Null, &mut runtime).unwrap();
        assert_eq!(lines, json!({"ok": ["ledger.balance", "ledger.balance"]}));
        let metric =
            evaluate_flow_with_runtime(&graph, "ObserveTwice", Value::Null, &mut runtime).unwrap();
        assert_eq!(metric, json!({"ok": 2}));
        let spans =
            evaluate_flow_with_runtime(&graph, "TraceOnce", Value::Null, &mut runtime).unwrap();
        assert_eq!(spans, json!({"ok": ["CalculateLedgerBalance"]}));
    }

    #[test]
    fn password_hash_and_verify_round_trip() {
        let mut runtime = BuiltinRuntime::new().unwrap();
        let mut config = BTreeMap::new();
        config.insert("pepper".into(), Value::String("test-pepper".into()));
        let hash = runtime
            .invoke(ProviderCall {
                provider: "DemoPassword",
                capacity: "PasswordHasher",
                implementation: "rust::axl::auth::password",
                operation: "hash",
                config: config.clone(),
                input: Value::String("secret".into()),
            })
            .unwrap();
        let verified = runtime
            .invoke(ProviderCall {
                provider: "DemoPassword",
                capacity: "PasswordHasher",
                implementation: "rust::axl::auth::password",
                operation: "verify",
                config,
                input: json!({"password": "secret", "hash": hash}),
            })
            .unwrap();
        assert_eq!(verified, Value::Bool(true));
    }

    #[test]
    fn jwt_auth_validates_hs256_sub_and_iss() {
        let secret = "demo-only";
        let issuer = "axl-demo";
        let good = encode_hs256_jwt(secret, &json!({"sub": "alice", "iss": issuer})).unwrap();
        let missing_sub = encode_hs256_jwt(secret, &json!({"iss": issuer})).unwrap();
        let wrong_iss = encode_hs256_jwt(secret, &json!({"sub": "alice", "iss": "other"})).unwrap();

        let mut runtime = BuiltinRuntime::new().unwrap();
        let mut config = BTreeMap::new();
        config.insert("secret".into(), Value::String(secret.into()));
        config.insert("issuer".into(), Value::String(issuer.into()));
        let authorize = |runtime: &mut BuiltinRuntime, token: &str| {
            runtime.invoke(ProviderCall {
                provider: "DemoJwt",
                capacity: "HttpAuth",
                implementation: "rust::axl::auth::jwt",
                operation: "authorize",
                config: config.clone(),
                input: Value::String(token.into()),
            })
        };

        assert_eq!(authorize(&mut runtime, &good).unwrap(), Value::Bool(true));
        assert_eq!(
            authorize(&mut runtime, &missing_sub).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            authorize(&mut runtime, &wrong_iss).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            authorize(&mut runtime, "not.a.jwt").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            authorize(&mut runtime, "a.b.c").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn sqlite_transaction_commit_survives_restart_and_rollback_hides_writes() {
        let database = std::env::temp_dir().join(format!(
            "axl-tx-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableTx
entity Record
  id: uuid required
  value: text required
entity RecordPair
  first: Record required
  second: Record required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
capacity TransactionManager
  op begin text -> Result<text>
  op commit text -> Result<unit>
  op rollback text -> Result<unit>
skill DurableRecords provides RecordStore
  native rust axl::store::sqlite
  config path: text = {path}
  effect db.read
  effect db.write
skill DurableTx provides TransactionManager
  native rust axl::tx::sqlite
  config path: text = {path}
  effect db.write
flow CommitTwo RecordPair -> Result<Record>
  in tx: TransactionManager = DurableTx
  in store: RecordStore = DurableRecords
  call tid = tx.begin("commit-two")?
  call first = store.save(input.first)?
  call second = store.save(input.second)?
  call done = tx.commit(tid)?
  return second
flow RollbackTwo RecordPair -> Result<unit>
  in tx: TransactionManager = DurableTx
  in store: RecordStore = DurableRecords
  call tid = tx.begin("rollback-two")?
  call first = store.save(input.first)?
  call second = store.save(input.second)?
  call done = tx.rollback(tid)?
  return done
flow Find uuid -> Result<Record>
  in store: RecordStore = DurableRecords
  call found = store.find(input)?
  return found
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let commit_pair = json!({
            "first": {"id": "tx-c1", "value": "one"},
            "second": {"id": "tx-c2", "value": "two"}
        });
        {
            let mut first = BuiltinRuntime::new().unwrap();
            let saved =
                evaluate_flow_with_runtime(&graph, "CommitTwo", commit_pair, &mut first).unwrap();
            assert_eq!(saved["ok"]["id"], "tx-c2");
        }
        {
            let mut second = BuiltinRuntime::new().unwrap();
            let found =
                evaluate_flow_with_runtime(&graph, "Find", json!("tx-c1"), &mut second).unwrap();
            assert_eq!(found["ok"]["value"], "one");
            let found =
                evaluate_flow_with_runtime(&graph, "Find", json!("tx-c2"), &mut second).unwrap();
            assert_eq!(found["ok"]["value"], "two");
        }

        let rollback_pair = json!({
            "first": {"id": "tx-r1", "value": "gone"},
            "second": {"id": "tx-r2", "value": "also-gone"}
        });
        {
            let mut runtime = BuiltinRuntime::new().unwrap();
            let rolled =
                evaluate_flow_with_runtime(&graph, "RollbackTwo", rollback_pair, &mut runtime)
                    .unwrap();
            assert_eq!(rolled, json!({"ok": null}));
        }
        {
            let mut runtime = BuiltinRuntime::new().unwrap();
            let missing =
                evaluate_flow_with_runtime(&graph, "Find", json!("tx-r1"), &mut runtime).unwrap();
            assert_eq!(missing["error"], "not_found");
            let missing =
                evaluate_flow_with_runtime(&graph, "Find", json!("tx-r2"), &mut runtime).unwrap();
            assert_eq!(missing["error"], "not_found");
        }
        drop(database.exists().then(|| std::fs::remove_file(&database)));
    }

    #[test]
    fn memory_transaction_rollback_restores_snapshot() {
        const SOURCE: &str = r#"axl 4
app MemoryTx
entity Record
  id: uuid required
  value: text required
entity RecordPair
  first: Record required
  second: Record required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
capacity TransactionManager
  op begin text -> Result<text>
  op commit text -> Result<unit>
  op rollback text -> Result<unit>
skill MemoryRecords provides RecordStore
  native rust axl::store::memory
  effect db.read
  effect db.write
skill MemoryTx provides TransactionManager
  native rust axl::tx::memory
  effect db.write
flow RollbackTwo RecordPair -> Result<unit>
  in tx: TransactionManager = MemoryTx
  in store: RecordStore = MemoryRecords
  call tid = tx.begin("rollback-two")?
  call first = store.save(input.first)?
  call second = store.save(input.second)?
  call done = tx.rollback(tid)?
  return done
flow Find uuid -> Result<Record>
  in store: RecordStore = MemoryRecords
  call found = store.find(input)?
  return found
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let pair = json!({
            "first": {"id": "m1", "value": "a"},
            "second": {"id": "m2", "value": "b"}
        });
        let rolled = evaluate_flow_with_runtime(&graph, "RollbackTwo", pair, &mut runtime).unwrap();
        assert_eq!(rolled, json!({"ok": null}));
        let missing =
            evaluate_flow_with_runtime(&graph, "Find", json!("m1"), &mut runtime).unwrap();
        assert_eq!(missing["error"], "not_found");
        let missing =
            evaluate_flow_with_runtime(&graph, "Find", json!("m2"), &mut runtime).unwrap();
        assert_eq!(missing["error"], "not_found");
    }

    #[test]
    fn sqlite_migration_up_survives_restart_and_down_rolls_back() {
        let database = std::env::temp_dir().join(format!(
            "axl-migrate-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableMigrate
capacity MigrationRunner
  op up text -> Result<text>
  op down text -> Result<text>
  op status unit -> Result<text>
skill DurableMigrations provides MigrationRunner
  native rust axl::migrate::sqlite
  config path: text = {path}
  effect db.write
flow Apply text -> Result<text>
  in migrations: MigrationRunner = DurableMigrations
  call version = migrations.up(input)?
  return version
flow Rollback text -> Result<text>
  in migrations: MigrationRunner = DurableMigrations
  call version = migrations.down(input)?
  return version
flow Status unit -> Result<text>
  in migrations: MigrationRunner = DurableMigrations
  call version = migrations.status(input)?
  return version
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        {
            let mut first = BuiltinRuntime::new().unwrap();
            let applied =
                evaluate_flow_with_runtime(&graph, "Apply", json!("v1"), &mut first).unwrap();
            assert_eq!(applied, json!({"ok": "v1"}));
            let applied =
                evaluate_flow_with_runtime(&graph, "Apply", json!("v2"), &mut first).unwrap();
            assert_eq!(applied, json!({"ok": "v2"}));
            let status =
                evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut first).unwrap();
            assert_eq!(status, json!({"ok": "v2"}));
        }
        {
            let mut second = BuiltinRuntime::new().unwrap();
            let status =
                evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut second).unwrap();
            assert_eq!(status, json!({"ok": "v2"}));
            let rolled =
                evaluate_flow_with_runtime(&graph, "Rollback", json!("v2"), &mut second).unwrap();
            assert_eq!(rolled, json!({"ok": "v2"}));
            let status =
                evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut second).unwrap();
            assert_eq!(status, json!({"ok": "v1"}));
        }
        {
            let mut third = BuiltinRuntime::new().unwrap();
            let status =
                evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut third).unwrap();
            assert_eq!(status, json!({"ok": "v1"}));
        }
        drop(database.exists().then(|| std::fs::remove_file(&database)));
    }

    #[test]
    fn memory_migration_up_down_and_status() {
        const SOURCE: &str = r#"axl 4
app MemoryMigrate
capacity MigrationRunner
  op up text -> Result<text>
  op down text -> Result<text>
  op status unit -> Result<text>
skill MemoryMigrations provides MigrationRunner
  native rust axl::migrate::memory
  effect db.write
flow Apply text -> Result<text>
  in migrations: MigrationRunner = MemoryMigrations
  call version = migrations.up(input)?
  return version
flow Rollback text -> Result<text>
  in migrations: MigrationRunner = MemoryMigrations
  call version = migrations.down(input)?
  return version
flow Status unit -> Result<text>
  in migrations: MigrationRunner = MemoryMigrations
  call version = migrations.status(input)?
  return version
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        let status =
            evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut runtime).unwrap();
        assert_eq!(status, json!({"ok": "0"}));
        let applied =
            evaluate_flow_with_runtime(&graph, "Apply", json!("v1"), &mut runtime).unwrap();
        assert_eq!(applied, json!({"ok": "v1"}));
        let applied =
            evaluate_flow_with_runtime(&graph, "Apply", json!("v2"), &mut runtime).unwrap();
        assert_eq!(applied, json!({"ok": "v2"}));
        let status =
            evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut runtime).unwrap();
        assert_eq!(status, json!({"ok": "v2"}));
        let rolled =
            evaluate_flow_with_runtime(&graph, "Rollback", json!("v2"), &mut runtime).unwrap();
        assert_eq!(rolled, json!({"ok": "v2"}));
        let status =
            evaluate_flow_with_runtime(&graph, "Status", Value::Null, &mut runtime).unwrap();
        assert_eq!(status, json!({"ok": "v1"}));
    }

    #[test]
    fn sqlite_store_query_filters_orders_pages_and_survives_restart() {
        let database = std::env::temp_dir().join(format!(
            "axl-query-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableQuery
entity Record
  id: uuid required
  kind: text required
  account: text required
  occurred_at: text required
entity RecordQuery
  filter: Map<text,text> optional
  order_by: text optional
  direction: text optional
  limit: int optional
  offset: int optional
entity RecordPage
  items: List<Record> required
  total: int required
  limit: int required
  offset: int required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
  op query RecordQuery -> Result<RecordPage> idempotent
skill DurableRecords provides RecordStore
  native rust axl::store::sqlite
  config path: text = {path}
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = DurableRecords
  call saved = store.save(input)?
  return saved
flow Query RecordQuery -> Result<RecordPage>
  in store: RecordStore = DurableRecords
  call page = store.query(input)?
  return page
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let records = [
            json!({"id": "q1", "kind": "income", "account": "a1", "occurred_at": "2026-08-27T09:00:00Z"}),
            json!({"id": "q2", "kind": "expense", "account": "a1", "occurred_at": "2026-08-27T10:00:00Z"}),
            json!({"id": "q3", "kind": "income", "account": "a1", "occurred_at": "2026-08-27T11:00:00Z"}),
        ];
        {
            let mut first = BuiltinRuntime::new().unwrap();
            for record in &records {
                let saved =
                    evaluate_flow_with_runtime(&graph, "Save", record.clone(), &mut first).unwrap();
                assert_eq!(saved["ok"]["id"], record["id"]);
            }
            let page = evaluate_flow_with_runtime(
                &graph,
                "Query",
                json!({
                    "filter": {"kind": "income", "account": "a1"},
                    "order_by": "occurred_at",
                    "direction": "desc",
                    "limit": 1,
                    "offset": 0
                }),
                &mut first,
            )
            .unwrap();
            assert_eq!(page["ok"]["total"], 2);
            assert_eq!(page["ok"]["limit"], 1);
            assert_eq!(page["ok"]["offset"], 0);
            assert_eq!(page["ok"]["items"][0]["id"], "q3");
        }
        {
            let mut second = BuiltinRuntime::new().unwrap();
            let page = evaluate_flow_with_runtime(
                &graph,
                "Query",
                json!({
                    "filter": {"kind": "income", "account": "a1"},
                    "order_by": "occurred_at",
                    "direction": "desc",
                    "limit": 1,
                    "offset": 0
                }),
                &mut second,
            )
            .unwrap();
            assert_eq!(page["ok"]["total"], 2);
            assert_eq!(page["ok"]["items"][0]["id"], "q3");
        }
        drop(database.exists().then(|| std::fs::remove_file(&database)));
    }

    #[test]
    fn memory_store_query_filters_orders_and_pages() {
        const SOURCE: &str = r#"axl 4
app MemoryQuery
entity Record
  id: uuid required
  kind: text required
  account: text required
  occurred_at: text required
entity RecordQuery
  filter: Map<text,text> optional
  order_by: text optional
  direction: text optional
  limit: int optional
  offset: int optional
entity RecordPage
  items: List<Record> required
  total: int required
  limit: int required
  offset: int required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
  op query RecordQuery -> Result<RecordPage> idempotent
skill MemoryRecords provides RecordStore
  native rust axl::store::memory
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = MemoryRecords
  call saved = store.save(input)?
  return saved
flow Query RecordQuery -> Result<RecordPage>
  in store: RecordStore = MemoryRecords
  call page = store.query(input)?
  return page
"#;
        let graph = compile_source(SOURCE).unwrap().graph;
        let mut runtime = BuiltinRuntime::new().unwrap();
        for record in [
            json!({"id": "m1", "kind": "income", "account": "cash", "occurred_at": "t1"}),
            json!({"id": "m2", "kind": "expense", "account": "cash", "occurred_at": "t2"}),
            json!({"id": "m3", "kind": "income", "account": "cash", "occurred_at": "t3"}),
        ] {
            evaluate_flow_with_runtime(&graph, "Save", record, &mut runtime).unwrap();
        }
        let page = evaluate_flow_with_runtime(
            &graph,
            "Query",
            json!({
                "filter": {"kind": "income"},
                "order_by": "occurred_at",
                "direction": "asc",
                "limit": 10,
                "offset": 1
            }),
            &mut runtime,
        )
        .unwrap();
        assert_eq!(page["ok"]["total"], 2);
        assert_eq!(page["ok"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(page["ok"]["items"][0]["id"], "m3");
    }

    #[test]
    fn document_store_persists_across_independent_runtimes() {
        let database = std::env::temp_dir().join(format!(
            "axl-document-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableDocumentStore
entity Record
  id: uuid required
  value: text required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
skill DurableRecords provides RecordStore
  native rust axl::store::document
  config path: text = {path}
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = DurableRecords
  call saved = store.save(input)?
  return saved
flow Find uuid -> Result<Record>
  in store: RecordStore = DurableRecords
  call found = store.find(input)?
  return found
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let record = json!({"id": "doc-1", "value": "survives restart"});
        {
            let mut first_runtime = BuiltinRuntime::new().unwrap();
            let saved =
                evaluate_flow_with_runtime(&graph, "Save", record.clone(), &mut first_runtime)
                    .unwrap();
            assert_eq!(saved, json!({"ok": record}));
        }
        {
            let mut second_runtime = BuiltinRuntime::new().unwrap();
            let found =
                evaluate_flow_with_runtime(&graph, "Find", json!("doc-1"), &mut second_runtime)
                    .unwrap();
            assert_eq!(found["ok"]["value"], "survives restart");
        }
        drop(database.exists().then(|| std::fs::remove_file(&database)));
    }

    #[test]
    fn document_store_query_filters_orders_pages_and_survives_restart() {
        let database = std::env::temp_dir().join(format!(
            "axl-document-query-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = serde_json::to_string(database.to_str().unwrap()).unwrap();
        let source = format!(
            r#"axl 4
app DurableDocumentQuery
entity Record
  id: uuid required
  kind: text required
  account: text required
  occurred_at: text required
entity RecordQuery
  filter: Map<text,text> optional
  order_by: text optional
  direction: text optional
  limit: int optional
  offset: int optional
entity RecordPage
  items: List<Record> required
  total: int required
  limit: int required
  offset: int required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
  op query RecordQuery -> Result<RecordPage> idempotent
skill DurableRecords provides RecordStore
  native rust axl::store::document
  config path: text = {path}
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = DurableRecords
  call saved = store.save(input)?
  return saved
flow Query RecordQuery -> Result<RecordPage>
  in store: RecordStore = DurableRecords
  call page = store.query(input)?
  return page
"#
        );
        let graph = compile_source(&source).unwrap().graph;
        let records = [
            json!({"id": "q1", "kind": "income", "account": "a1", "occurred_at": "2026-08-27T09:00:00Z"}),
            json!({"id": "q2", "kind": "expense", "account": "a1", "occurred_at": "2026-08-27T10:00:00Z"}),
            json!({"id": "q3", "kind": "income", "account": "a1", "occurred_at": "2026-08-27T11:00:00Z"}),
        ];
        {
            let mut first = BuiltinRuntime::new().unwrap();
            for record in &records {
                let saved =
                    evaluate_flow_with_runtime(&graph, "Save", record.clone(), &mut first).unwrap();
                assert_eq!(saved["ok"]["id"], record["id"]);
            }
            let page = evaluate_flow_with_runtime(
                &graph,
                "Query",
                json!({
                    "filter": {"kind": "income", "account": "a1"},
                    "order_by": "occurred_at",
                    "direction": "desc",
                    "limit": 1,
                    "offset": 0
                }),
                &mut first,
            )
            .unwrap();
            assert_eq!(page["ok"]["total"], 2);
            assert_eq!(page["ok"]["items"][0]["id"], "q3");
        }
        {
            let mut second = BuiltinRuntime::new().unwrap();
            let page = evaluate_flow_with_runtime(
                &graph,
                "Query",
                json!({
                    "filter": {"kind": "income", "account": "a1"},
                    "order_by": "occurred_at",
                    "direction": "desc",
                    "limit": 1,
                    "offset": 0
                }),
                &mut second,
            )
            .unwrap();
            assert_eq!(page["ok"]["total"], 2);
            assert_eq!(page["ok"]["items"][0]["id"], "q3");
        }
        drop(database.exists().then(|| std::fs::remove_file(&database)));
    }

    #[test]
    fn three_store_families_share_save_find_query_contract() {
        for native in [
            "axl::store::memory",
            "axl::store::sqlite",
            "axl::store::document",
        ] {
            let source = format!(
                r#"axl 4
app StoreFamily
entity Record
  id: uuid required
  kind: text required
  account: text required
  occurred_at: text required
entity RecordQuery
  filter: Map<text,text> optional
  order_by: text optional
  direction: text optional
  limit: int optional
  offset: int optional
entity RecordPage
  items: List<Record> required
  total: int required
  limit: int required
  offset: int required
capacity RecordStore
  op save Record -> Result<Record>
  op find uuid -> Result<Record> idempotent
  op query RecordQuery -> Result<RecordPage> idempotent
skill FamilyRecords provides RecordStore
  native rust {native}
  effect db.read
  effect db.write
flow Save Record -> Result<Record>
  in store: RecordStore = FamilyRecords
  call saved = store.save(input)?
  return saved
flow Find uuid -> Result<Record>
  in store: RecordStore = FamilyRecords
  call found = store.find(input)?
  return found
flow Query RecordQuery -> Result<RecordPage>
  in store: RecordStore = FamilyRecords
  call page = store.query(input)?
  return page
"#
            );
            let graph = compile_source(&source).unwrap().graph;
            let mut runtime = BuiltinRuntime::new().unwrap();
            let record = json!({
                "id": "shared-1",
                "kind": "income",
                "account": "a1",
                "occurred_at": "2026-08-28T01:00:00Z"
            });
            let saved =
                evaluate_flow_with_runtime(&graph, "Save", record.clone(), &mut runtime).unwrap();
            assert_eq!(saved["ok"]["id"], "shared-1", "native={native}");
            let found = evaluate_flow_with_runtime(&graph, "Find", json!("shared-1"), &mut runtime)
                .unwrap();
            assert_eq!(found["ok"], record, "native={native}");
            let page = evaluate_flow_with_runtime(
                &graph,
                "Query",
                json!({
                    "filter": {"kind": "income"},
                    "order_by": "occurred_at",
                    "direction": "asc",
                    "limit": 5,
                    "offset": 0
                }),
                &mut runtime,
            )
            .unwrap();
            assert_eq!(page["ok"]["total"], 1, "native={native}");
            assert_eq!(page["ok"]["items"][0]["id"], "shared-1", "native={native}");
        }
    }
}
