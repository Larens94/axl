use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};
use serde_json::{Map, Value, json};

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
    memory: Arc<Mutex<BTreeMap<String, BTreeMap<String, Value>>>>,
    sqlite: Arc<Mutex<BTreeMap<String, Arc<Mutex<Connection>>>>>,
}

impl BuiltinRuntime {
    pub fn new() -> Result<Self, RuntimeError> {
        Ok(Self {
            memory: Arc::new(Mutex::new(BTreeMap::new())),
            sqlite: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }

    fn sqlite_connection(&self, call: &ProviderCall<'_>) -> Result<Arc<Mutex<Connection>>, String> {
        let configured_path = call.config.get("path").and_then(Value::as_str);
        let key = configured_path
            .map(str::to_string)
            .unwrap_or_else(|| format!(":memory:{}", call.provider));
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
            "rust::axl::auth::bearer" => bearer_auth_call(call),
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

fn memory_store_call(
    stores: &mut BTreeMap<String, BTreeMap<String, Value>>,
    call: ProviderCall<'_>,
) -> Result<Value, String> {
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
        operation => Err(format!(
            "SQLite store does not implement operation '{operation}' for {}",
            call.capacity
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

fn initialize_sqlite(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS axl_records (\
             provider TEXT NOT NULL, \
             record_id TEXT NOT NULL, \
             payload TEXT NOT NULL, \
             PRIMARY KEY (provider, record_id));",
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
}
