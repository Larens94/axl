use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ir::*;
use crate::memory::MemoryStore;
use crate::policy::{ApprovalRequest, AuditEvent, Tool};
use crate::primitives;
use crate::validation;

#[derive(Debug, Clone)]
pub struct RuntimeError(pub String);

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuntimeError {}

pub struct ExecutionResult {
    pub output: Vec<Value>,
    pub memory: HashMap<String, Value>,
    pub audit: Vec<AuditEvent>,
}

pub struct InterpreterConfig {
    pub max_steps: usize,
    pub max_output_bytes: usize,
    pub max_value_bytes: usize,
    pub max_value_nodes: usize,
    pub max_value_depth: usize,
    pub max_tool_calls: usize,
    pub max_memory_ops: usize,
    pub max_function_depth: usize,
    pub scope: String,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            max_steps: 10_000, max_output_bytes: 1_000_000,
            max_value_bytes: 1_000_000, max_value_nodes: 100_000, max_value_depth: 256,
            max_tool_calls: 100, max_memory_ops: 1_000, max_function_depth: 256,
            scope: "session:default".into(),
        }
    }
}

enum Runnable {
    Agent(Vec<String>, Vec<Instruction>), // name+tools, body
    Workflow(Vec<Instruction>),
}

struct InterpreterState {
    tools: HashMap<String, Tool>,
    config: InterpreterConfig,
    approve_fn: Option<Box<dyn Fn(&ApprovalRequest) -> bool + Send + Sync>>,
    variables: HashMap<String, Value>,
    output: Vec<Value>,
    output_bytes: usize,
    steps: usize,
    tool_calls: usize,
    memory_ops: usize,
    function_depth: usize,
    functions: HashMap<String, Function>,
    runnables: HashMap<String, Runnable>,
    current_agent: Option<String>,
    audit: Vec<AuditEvent>,
}

pub fn render_value(value: &Value) -> Result<String, RuntimeError> {
    match value {
        Value::Bool(b) => Ok(format!("{b}")),
        Value::Int(n) => Ok(n.to_string()),
        Value::String(s) => Ok(s.clone()),
        Value::List(_) | Value::Map(_) => {
            serde_json::to_string(&value.to_json_value())
                .map_err(|_| RuntimeError("output value cannot be rendered".into()))
        }
        _ => Ok(String::new()),
    }
}

pub fn run_program(
    program: &Program,
    tools: Vec<Tool>,
    memory: Arc<Mutex<dyn MemoryStore>>,
    config: InterpreterConfig,
    approve_fn: Option<Box<dyn Fn(&ApprovalRequest) -> bool + Send + Sync>>,
) -> Result<ExecutionResult, RuntimeError> {
    validation::validate(program).map_err(|e| RuntimeError(e.to_string()))?;

    let mut tool_map = HashMap::new();
    for t in tools {
        let name = t.name.clone();
        tool_map.insert(name, t);
    }

    let mut runnables = HashMap::new();
    let mut functions = HashMap::new();
    for instruction in &program.instructions {
        match instruction {
            Instruction::Agent(a) => { runnables.insert(a.name.clone(), Runnable::Agent(a.tools.clone(), a.body.clone())); }
            Instruction::Workflow(w) => { runnables.insert(w.name.clone(), Runnable::Workflow(w.body.clone())); }
            Instruction::Function(f) => { functions.insert(f.name.clone(), f.clone()); }
            _ => {}
        }
    }

    let mut state = InterpreterState {
        tools: tool_map, config, approve_fn,
        variables: HashMap::new(), output: Vec::new(), output_bytes: 0, steps: 0,
        tool_calls: 0, memory_ops: 0, function_depth: 0,
        functions, runnables, current_agent: None, audit: Vec::new(),
    };

    execute_block(&mut state, &program.instructions, memory.clone())?;

    let snapshot = memory.lock().unwrap().snapshot(&state.config.scope).map_err(RuntimeError)?;

    Ok(ExecutionResult {
        output: state.output,
        memory: snapshot,
        audit: state.audit,
    })
}

fn step(state: &mut InterpreterState) -> Result<(), RuntimeError> {
    state.steps += 1;
    if state.steps > state.config.max_steps {
        return Err(RuntimeError(format!("execution budget exceeded ({})", state.config.max_steps)));
    }
    Ok(())
}

fn memory_op_check(state: &mut InterpreterState) -> Result<(), RuntimeError> {
    state.memory_ops += 1;
    if state.memory_ops > state.config.max_memory_ops {
        return Err(RuntimeError(format!("memory operation budget exceeded ({})", state.config.max_memory_ops)));
    }
    Ok(())
}

fn execute_block(state: &mut InterpreterState, instructions: &[Instruction], memory: Arc<Mutex<dyn MemoryStore>>) -> Result<(), RuntimeError> {
    for instruction in instructions {
        step(state)?;
        match instruction {
            Instruction::MemoryWrite(mw) => {
                memory_op_check(state)?;
                let val = evaluate(state, &mw.value, memory.clone())?;
                memory.lock().unwrap().set(&mw.key, val, &state.config.scope, mw.confidence, mw.ttl_seconds, &mw.source).map_err(RuntimeError)?;
            }
            Instruction::Forget(key) => {
                memory_op_check(state)?;
                memory.lock().unwrap().delete(key, &state.config.scope).map_err(RuntimeError)?;
            }
            Instruction::Let { target, value, .. } => {
                let val = evaluate(state, value, memory.clone())?;
                state.variables.insert(target.clone(), val);
            }
            Instruction::Return(expr) => {
                let _val = evaluate(state, expr, memory.clone())?;
                return Err(RuntimeError("__return".into()));
            }
            Instruction::Emit(expr) => {
                let val = evaluate(state, expr, memory.clone())?;
                let size = render_value(&val)?.len();
                if state.output_bytes + size > state.config.max_output_bytes {
                    return Err(RuntimeError(format!("output budget exceeded ({})", state.config.max_output_bytes)));
                }
                state.output_bytes += size;
                state.output.push(val);
            }
            Instruction::If(if_inst) => {
                let cond = evaluate(state, &if_inst.condition, memory.clone())?;
                match cond {
                    Value::Bool(true) => execute_block(state, &if_inst.body, memory.clone())?,
                    Value::Bool(false) => execute_block(state, &if_inst.else_body, memory.clone())?,
                    _ => return Err(RuntimeError("if condition must be boolean".into())),
                }
            }
            Instruction::While(while_inst) => {
                loop {
                    let cond = evaluate(state, &while_inst.condition, memory.clone())?;
                    match cond {
                        Value::Bool(true) => {}
                        Value::Bool(false) => break,
                        _ => return Err(RuntimeError("while condition must be boolean".into())),
                    }
                    step(state)?;
                    execute_block(state, &while_inst.body, memory.clone())?;
                }
            }
            Instruction::Run(name) => {
                let (agent_tools, agent_body) = match state.runnables.get(name) {
                    Some(Runnable::Agent(tools, body)) => (tools.clone(), body.clone()),
                    Some(Runnable::Workflow(body)) => (vec![], body.clone()),
                    None => return Err(RuntimeError(format!("unknown agent or workflow '{name}'"))),
                };
                let prev_agent = state.current_agent.clone();
                let prev_vars = state.variables.clone();
                if !agent_tools.is_empty() {
                    state.current_agent = Some(name.clone());
                    state.variables = HashMap::new();
                }
                let result = execute_block(state, &agent_body, memory.clone());
                state.current_agent = prev_agent;
                state.variables = prev_vars;
                result?;
            }
            Instruction::Agent(_) | Instruction::Workflow(_) | Instruction::Function(_) | Instruction::UiView(_) | Instruction::Annotation(_) => {}
            _ => {}
        }
    }
    Ok(())
}

fn evaluate(state: &mut InterpreterState, expr: &Expression, memory: Arc<Mutex<dyn MemoryStore>>) -> Result<Value, RuntimeError> {
    step(state)?;
    match expr {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::ListExpression(items) => {
            let vals: Vec<Value> = items.iter().map(|i| evaluate(state, i, memory.clone())).collect::<Result<_, _>>()?;
            Ok(Value::List(vals))
        }
        Expression::MapExpression(entries) => {
            let vals: Vec<(Value, Value)> = entries.iter()
                .map(|(k, v)| Ok((evaluate(state, k, memory.clone())?, evaluate(state, v, memory.clone())?)))
                .collect::<Result<_, RuntimeError>>()?;
            Ok(Value::Map(vals))
        }
        Expression::Variable(name) => {
            state.variables.get(name).cloned()
                .ok_or_else(|| RuntimeError(format!("unknown variable '{name}'")))
        }
        Expression::Recall(key) => {
            memory_op_check(state)?;
            match memory.lock().unwrap().get(key, &state.config.scope).map_err(RuntimeError)? {
                Some(v) => Ok(v),
                None => Err(RuntimeError(format!("unknown memory '{key}'"))),
            }
        }
        Expression::ToolCall { name, arguments } => {
            let _ = std::fs::write("/tmp/axl_toolcall.txt", format!("name={name} args={}\n", arguments.len()));
            state.tool_calls += 1;
            if state.tool_calls > state.config.max_tool_calls {
                return Err(RuntimeError(format!("tool call budget exceeded ({})", state.config.max_tool_calls)));
            }

            // Check tool exists — check the list first to avoid accidentally executing side-effecting primitives
            let is_native_primitive = primitives::available_primitives().contains(&name.as_str()) || primitives::call_primitive(name, &[]).is_ok();
            let tool_effect = if is_native_primitive {
                Some("native".to_string())
            } else {
                state.tools.get(name).map(|t| t.effect.clone())
            };
            let tool_approval = if is_native_primitive { Some(false) } else { state.tools.get(name).map(|t| t.approval) };
            let tool_exists = is_native_primitive || state.tools.contains_key(name);

            if !tool_exists {
                let request = ApprovalRequest { tool: name.clone(), arguments: vec![], effect: "unknown".into() };
                state.audit.push(AuditEvent::create(&request, "denied"));
                return Err(RuntimeError(format!("tool '{name}' is not allowed")));
            }

            // Check agent grants
            if let Some(ref agent_name) = state.current_agent {
                let agent_has_tool = match state.runnables.get(agent_name) {
                    Some(Runnable::Agent(tools, _)) => tools.contains(name),
                    _ => false,
                };
                if !agent_has_tool {
                    let request = ApprovalRequest { tool: name.clone(), arguments: vec![], effect: tool_effect.unwrap_or_default() };
                    state.audit.push(AuditEvent::create(&request, "denied"));
                    return Err(RuntimeError(format!("tool '{name}' not granted to agent '{agent_name}'")));
                }
            }

            let args: Vec<Value> = arguments.iter().map(|a| evaluate(state, a, memory.clone())).collect::<Result<_, _>>()?;
            if name.contains("server") {
                let _ = std::fs::write("/tmp/axl_interp_debug.txt", format!("name={name} raw={} evaluated={}\n", arguments.len(), args.len()));
            }
            let request = ApprovalRequest {
                tool: name.clone(), arguments: args.clone(),
                effect: tool_effect.clone().unwrap_or_default(),
            };

            if tool_approval == Some(true) {
                match &state.approve_fn {
                    None => {
                        state.audit.push(AuditEvent::create(&request, "approval_required"));
                        return Err(RuntimeError(format!("tool '{name}' requires approval")));
                    }
                    Some(f) => {
                        if !f(&request) {
                            state.audit.push(AuditEvent::create(&request, "denied"));
                            return Err(RuntimeError(format!("tool '{name}' denied")));
                        }
                        state.audit.push(AuditEvent::create(&request, "approved"));
                    }
                }
            }

            // Execute: native primitive or user tool
            let result = if is_native_primitive {
                primitives::call_primitive(name, &args).map_err(|e| {
                    let req = ApprovalRequest { tool: name.clone(), arguments: args.clone(), effect: tool_effect.unwrap_or_default() };
                    state.audit.push(AuditEvent::create(&req, "failed"));
                    RuntimeError(format!("primitive '{name}' failed: {e}"))
                })?
            } else {
                let handler = state.tools.get(name).map(|t| &t.handler);
                handler.unwrap()(&args).map_err(|e| {
                    let req = ApprovalRequest { tool: name.clone(), arguments: args.clone(), effect: tool_effect.unwrap_or_default() };
                    state.audit.push(AuditEvent::create(&req, "failed"));
                    RuntimeError(format!("tool '{name}' failed: {e}"))
                })?
            };
            Ok(result)
        }
        Expression::FunctionCall { name, arguments } => {
            let func = state.functions.get(name).cloned()
                .ok_or_else(|| RuntimeError(format!("unknown function '{name}'")))?;

            if state.function_depth >= state.config.max_function_depth {
                return Err(RuntimeError(format!("function call depth exceeded ({})", state.config.max_function_depth)));
            }

            let args: Vec<Value> = arguments.iter().map(|a| evaluate(state, a, memory.clone())).collect::<Result<_, _>>()?;
            let prev_vars = state.variables.clone();
            state.variables = func.parameters.iter().zip(args.iter())
                .map(|(p, v)| (p.name.clone(), v.clone())).collect();
            state.function_depth += 1;

            let result = execute_block_returning(state, &func.body, memory.clone());
            state.function_depth -= 1;
            state.variables = prev_vars;

            match result {
                Ok(Some(v)) => Ok(v),
                Ok(None) => Err(RuntimeError(format!("function '{name}' completed without return"))),
                Err(e) if e.0 == "__return" => {
                    // Return value was captured by the last emit or we need another approach
                    Err(RuntimeError(format!("function '{name}' completed without return")))
                }
                Err(e) => Err(e),
            }
        }
        Expression::Binary { left, operator, right } => {
            let lv = evaluate(state, left, memory.clone())?;
            let rv = evaluate(state, right, memory.clone())?;
            binary_op(&lv, operator, &rv)
        }
        _ => Err(RuntimeError("unsupported expression".into())),
    }
}

fn execute_block_returning(state: &mut InterpreterState, instructions: &[Instruction], memory: Arc<Mutex<dyn MemoryStore>>) -> Result<Option<Value>, RuntimeError> {
    for instruction in instructions {
        step(state)?;
        match instruction {
            Instruction::MemoryWrite(mw) => {
                memory_op_check(state)?;
                let val = evaluate(state, &mw.value, memory.clone())?;
                memory.lock().unwrap().set(&mw.key, val, &state.config.scope, mw.confidence, mw.ttl_seconds, &mw.source).map_err(RuntimeError)?;
            }
            Instruction::Forget(key) => {
                memory_op_check(state)?;
                memory.lock().unwrap().delete(key, &state.config.scope).map_err(RuntimeError)?;
            }
            Instruction::Let { target, value, .. } => {
                let val = evaluate(state, value, memory.clone())?;
                state.variables.insert(target.clone(), val);
            }
            Instruction::Return(expr) => {
                return Ok(Some(evaluate(state, expr, memory.clone())?));
            }
            Instruction::Emit(expr) => {
                let val = evaluate(state, expr, memory.clone())?;
                let size = render_value(&val)?.len();
                if state.output_bytes + size > state.config.max_output_bytes {
                    return Err(RuntimeError(format!("output budget exceeded ({})", state.config.max_output_bytes)));
                }
                state.output_bytes += size;
                state.output.push(val);
            }
            Instruction::If(if_inst) => {
                let cond = evaluate(state, &if_inst.condition, memory.clone())?;
                match cond {
                    Value::Bool(true) => {
                        if let Some(v) = execute_block_returning(state, &if_inst.body, memory.clone())? {
                            return Ok(Some(v));
                        }
                    }
                    Value::Bool(false) => {
                        if let Some(v) = execute_block_returning(state, &if_inst.else_body, memory.clone())? {
                            return Ok(Some(v));
                        }
                    }
                    _ => return Err(RuntimeError("if condition must be boolean".into())),
                }
            }
            Instruction::While(while_inst) => {
                loop {
                    let cond = evaluate(state, &while_inst.condition, memory.clone())?;
                    match cond {
                        Value::Bool(true) => {}
                        Value::Bool(false) => break,
                        _ => return Err(RuntimeError("while condition must be boolean".into())),
                    }
                    step(state)?;
                    if let Some(v) = execute_block_returning(state, &while_inst.body, memory.clone())? {
                        return Ok(Some(v));
                    }
                }
            }
            Instruction::Run(name) => {
                let (agent_tools, agent_body) = match state.runnables.get(name) {
                    Some(Runnable::Agent(tools, body)) => (tools.clone(), body.clone()),
                    Some(Runnable::Workflow(body)) => (vec![], body.clone()),
                    None => return Err(RuntimeError(format!("unknown agent or workflow '{name}'"))),
                };
                let prev_agent = state.current_agent.clone();
                let prev_vars = state.variables.clone();
                if !agent_tools.is_empty() {
                    state.current_agent = Some(name.clone());
                    state.variables = HashMap::new();
                }
                let result = execute_block_returning(state, &agent_body, memory.clone());
                state.current_agent = prev_agent;
                state.variables = prev_vars;
                if let Some(v) = result? {
                    return Ok(Some(v));
                }
            }
            _ => {}
        }
    }
    Ok(None)
}

fn binary_op(left: &Value, op: &str, right: &Value) -> Result<Value, RuntimeError> {
    match op {
        "==" | "!=" => {
            let eq = left == right;
            Ok(Value::Bool(if op == "==" { eq } else { !eq }))
        }
        "+" => match (left, right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            _ => Err(RuntimeError("invalid operands for '+'".into())),
        }
        "-" if matches!((left, right), (Value::Int(_), Value::Int(_))) => {
            if let (Value::Int(a), Value::Int(b)) = (left, right) { Ok(Value::Int(a - b)) } else { unreachable!() }
        }
        "*" if matches!((left, right), (Value::Int(_), Value::Int(_))) => {
            if let (Value::Int(a), Value::Int(b)) = (left, right) { Ok(Value::Int(a * b)) } else { unreachable!() }
        }
        "/" if matches!((left, right), (Value::Int(_), Value::Int(_))) => {
            if let (Value::Int(a), Value::Int(b)) = (left, right) {
                if *b == 0 { return Err(RuntimeError("division by zero".into())); }
                if a % b != 0 { return Err(RuntimeError("non-integer division is not supported".into())); }
                Ok(Value::Int(a / b))
            } else { unreachable!() }
        }
        ">" | "<" | ">=" | "<=" => {
            if let (Value::Int(a), Value::Int(b)) = (left, right) {
                Ok(Value::Bool(match op {
                    ">" => a > b, "<" => a < b, ">=" => a >= b, "<=" => a <= b,
                    _ => unreachable!(),
                }))
            } else {
                Err(RuntimeError(format!("invalid operands for '{op}'")))
            }
        }
        _ => Err(RuntimeError(format!("unknown operator '{op}'"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compact::parse_compact;

    #[test]
    fn basic_emit() {
        let program = parse_compact("2;12|\"hello\"").unwrap();
        let memory: Arc<Mutex<dyn MemoryStore>> = Arc::new(Mutex::new(crate::memory::InMemoryStore::new()));
        let result = run_program(&program, vec![], memory, InterpreterConfig::default(), None).unwrap();
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0], Value::String("hello".into()));
    }

    #[test]
    fn arithmetic() {
        let program = parse_compact("2;10|x|#2,#3,#4,*,+|i;12|$x").unwrap();
        let memory: Arc<Mutex<dyn MemoryStore>> = Arc::new(Mutex::new(crate::memory::InMemoryStore::new()));
        let result = run_program(&program, vec![], memory, InterpreterConfig::default(), None).unwrap();
        assert_eq!(result.output[0], Value::Int(14));
    }
}
