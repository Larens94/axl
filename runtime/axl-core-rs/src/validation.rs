use std::collections::{HashMap, HashSet};

use crate::ir::*;
use crate::type_names;
use crate::ui_registry;

const MAX_NESTING_DEPTH: usize = 256;
const MAX_CALL_DEPTH: usize = 256;

const RESERVED: &[&str] = &[
    "agent", "call", "else", "emit", "end", "false", "forget", "fn", "if",
    "import", "let", "memory", "meta", "recall", "return", "run", "true",
    "uses", "while", "workflow",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

fn check_identifier(value: &str, label: &str) -> Result<(), ValidationError> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || !value.as_bytes()[0].is_ascii_alphabetic() && value.as_bytes()[0] != b'_'
    {
        return Err(ValidationError(format!("invalid {label} identifier")));
    }
    if RESERVED.contains(&value) {
        return Err(ValidationError(format!("reserved {label} identifier '{value}'")));
    }
    Ok(())
}

fn check_qualified_identifier(value: &str, label: &str) -> Result<(), ValidationError> {
    for part in value.split('.') {
        check_identifier(part, label)?;
    }
    Ok(())
}

fn check_type_name(value: &str) -> Result<(), ValidationError> {
    type_names::validate_type_name(value).map_err(|e| ValidationError(e.to_string()))
}

pub fn validate(program: &Program) -> Result<(), ValidationError> {
    validate_nesting(program)?;
    let mut names = HashSet::new();
    let mut view_ids = HashSet::new();
    let mut runnables: HashMap<String, usize> = HashMap::new();

    for (i, instruction) in program.instructions.iter().enumerate() {
        match instruction {
            Instruction::Agent(agent) => {
                check_identifier(&agent.name, "runnable")?;
                if !names.insert(agent.name.clone()) {
                    return Err(ValidationError(format!("duplicate runnable '{}'", agent.name)));
                }
                runnables.insert(agent.name.clone(), i);
            }
            Instruction::Workflow(wf) => {
                check_identifier(&wf.name, "runnable")?;
                if !names.insert(wf.name.clone()) {
                    return Err(ValidationError(format!("duplicate runnable '{}'", wf.name)));
                }
                runnables.insert(wf.name.clone(), i);
            }
            Instruction::Function(func) => {
                check_qualified_identifier(&func.name, "function")?;
            }
            Instruction::UiView(view) => {
                if view.view_id < 1 {
                    return Err(ValidationError("UI view id must be a positive integer".into()));
                }
                if !view_ids.insert(view.view_id) {
                    return Err(ValidationError(format!("duplicate UI view id '{}'", view.view_id)));
                }
            }
            _ => {}
        }
    }

    for instruction in &program.instructions {
        validate_instruction(instruction, &names, true)?;
    }
    validate_call_graph(program, &runnables)?;
    Ok(())
}

fn validate_nesting(program: &Program) -> Result<(), ValidationError> {
    let mut stack: Vec<(&Instruction, usize)> = program.instructions.iter().map(|i| (i, 1)).collect();
    while let Some((node, depth)) = stack.pop() {
        if depth > MAX_NESTING_DEPTH {
            return Err(ValidationError(format!("program nesting depth exceeds {MAX_NESTING_DEPTH}")));
        }
        match node {
            Instruction::Agent(a) => stack.extend(a.body.iter().map(|i| (i, depth + 1))),
            Instruction::Workflow(w) => stack.extend(w.body.iter().map(|i| (i, depth + 1))),
            Instruction::Function(f) => stack.extend(f.body.iter().map(|i| (i, depth + 1))),
            Instruction::While(w) => {
                stack.extend(w.body.iter().map(|i| (i, depth + 1)));
            }
            Instruction::If(if_inst) => {
                stack.extend(if_inst.body.iter().map(|i| (i, depth + 1)));
                stack.extend(if_inst.else_body.iter().map(|i| (i, depth + 1)));
            }
            Instruction::UiView(v) => {
                // validate UI tree nesting
                let mut ui_stack = vec![(&v.root, depth + 1)];
                while let Some((node, d)) = ui_stack.pop() {
                    if d > MAX_NESTING_DEPTH {
                        return Err(ValidationError(format!("program nesting depth exceeds {MAX_NESTING_DEPTH}")));
                    }
                    ui_stack.extend(node.children.iter().map(|c| (c, d + 1)));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_instruction(
    instruction: &Instruction, names: &HashSet<String>, top_level: bool,
) -> Result<(), ValidationError> {
    match instruction {
        Instruction::Agent(agent) => {
            if !top_level {
                return Err(ValidationError("agent and workflow declarations must be top-level".into()));
            }
            for tool in &agent.tools {
                check_identifier(tool, "tool")?;
            }
            if agent.tools.len() != agent.tools.iter().collect::<HashSet<_>>().len() {
                return Err(ValidationError(format!("duplicate tool grant in agent '{}'", agent.name)));
            }
            validate_block(&agent.body, names)?;
        }
        Instruction::Workflow(wf) => {
            if !top_level {
                return Err(ValidationError("agent and workflow declarations must be top-level".into()));
            }
            validate_block(&wf.body, names)?;
        }
        Instruction::Function(func) => {
            if !top_level {
                return Err(ValidationError("agent and workflow declarations must be top-level".into()));
            }
            check_type_name(&func.return_type)?;
            for param in &func.parameters {
                check_identifier(&param.name, "parameter")?;
                check_type_name(&param.type_name)?;
            }
            validate_block(&func.body, names)?;
        }
        Instruction::Run(name) => {
            check_identifier(name, "runnable")?;
            if !names.contains(name) {
                return Err(ValidationError(format!("unknown runnable '{name}'")));
            }
        }
        Instruction::If(if_inst) => {
            validate_expression(&if_inst.condition)?;
            validate_block(&if_inst.body, names)?;
            validate_block(&if_inst.else_body, names)?;
        }
        Instruction::While(w) => {
            validate_expression(&w.condition)?;
            validate_block(&w.body, names)?;
        }
        Instruction::MemoryWrite(mw) => {
            check_identifier(&mw.key, "memory")?;
            check_identifier(&mw.source, "source")?;
            validate_expression(&mw.value)?;
            if mw.confidence < 0 || mw.confidence > 100 {
                return Err(ValidationError("memory confidence must be an integer from 0 to 100".into()));
            }
            if let Some(ttl) = mw.ttl_seconds {
                if ttl < 1 {
                    return Err(ValidationError("memory ttl must be a positive integer".into()));
                }
            }
        }
        Instruction::Let { target, value, type_name } => {
            check_identifier(target, "variable")?;
            if let Some(tn) = type_name { check_type_name(tn)?; }
            validate_expression(value)?;
        }
        Instruction::Return(expr) | Instruction::Emit(expr) => validate_expression(expr)?,
        Instruction::Forget(key) => check_identifier(key, "memory")?,
        Instruction::Annotation(ann) => {
            if !top_level {
                return Err(ValidationError("annotations must be top-level".into()));
            }
            if !ui_registry::annotation_kind_valid(ann.kind) {
                return Err(ValidationError(format!("unknown annotation kind '{}'", ann.kind)));
            }
            if ann.target < 1 {
                return Err(ValidationError("annotation target must be a positive integer".into()));
            }
            if ann.value.is_empty() {
                return Err(ValidationError("annotation value must be a non-empty string".into()));
            }
        }
        Instruction::UiView(view) => {
            if !top_level {
                return Err(ValidationError("UI views must be top-level".into()));
            }
            validate_ui_tree(&view.root)?;
        }
        _ => {}
    }
    Ok(())
}

fn validate_block(body: &[Instruction], names: &HashSet<String>) -> Result<(), ValidationError> {
    for child in body {
        validate_instruction(child, names, false)?;
    }
    Ok(())
}

fn validate_expression(expr: &Expression) -> Result<(), ValidationError> {
    match expr {
        Expression::Literal(Value::String(s)) => {
            // validate UTF-8
            let _ = s.as_bytes();
        }
        Expression::Literal(_) => {}
        Expression::Binary { left, operator, right } => {
            const OPERATORS: &[&str] = &["+", "-", "*", "/", "==", "!=", ">", "<", ">=", "<="];
            if !OPERATORS.contains(&operator.as_str()) {
                return Err(ValidationError(format!("unknown operator '{operator}'")));
            }
            validate_expression(left)?;
            validate_expression(right)?;
        }
        Expression::ToolCall { name, arguments } => {
            check_identifier(name, "tool")?;
            for arg in arguments { validate_expression(arg)?; }
        }
        Expression::FunctionCall { name, arguments } => {
            check_qualified_identifier(name, "function")?;
            for arg in arguments { validate_expression(arg)?; }
        }
        Expression::ListExpression(items) => {
            for item in items { validate_expression(item)?; }
        }
        Expression::MapExpression(entries) => {
            for (k, v) in entries {
                validate_expression(k)?;
                validate_expression(v)?;
            }
        }
        Expression::Variable(name) => check_identifier(name, "variable")?,
        Expression::Recall(key) => check_identifier(key, "memory")?,
        _ => {}
    }
    Ok(())
}

fn validate_ui_tree(root: &UiNode) -> Result<(), ValidationError> {
    let mut ids = HashSet::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.node_id < 1 || !ids.insert(node.node_id) {
            return Err(ValidationError(format!("invalid or duplicate UI node id '{}'", node.node_id)));
        }
        let contract = ui_registry::component(node.component_id)
            .ok_or_else(|| ValidationError(format!("unknown UI component '{}'", node.component_id)))?;
        let mut prop_ids = HashSet::new();
        for prop in &node.properties {
            if !prop_ids.insert(prop.property_id) {
                return Err(ValidationError(format!("duplicate UI property on node '{}'", node.node_id)));
            }
            let expected = contract.properties.iter().find(|p| p.id == prop.property_id)
                .ok_or_else(|| ValidationError(format!("property '{}' is not valid for component '{}'", prop.property_id, node.component_id)))?;
            match &prop.value {
                Expression::Literal(val) => {
                    let actual = match val {
                        Value::String(_) => "string",
                        Value::Int(_) => "int",
                        Value::Bool(_) => "bool",
                        _ => return Err(ValidationError("UI property has unsupported literal type".into())),
                    };
                    if actual != expected.type_name {
                        return Err(ValidationError(format!("property '{}' requires {}, got {}", prop.property_id, expected.type_name, actual)));
                    }
                }
                Expression::Variable(_) | Expression::ToolCall { .. } | Expression::FunctionCall { .. } => {
                    // Dynamic properties are allowed — resolved at runtime
                }
                _ => {
                    return Err(ValidationError("UI property has unsupported expression type".into()));
                }
            }
        }
        let mut event_ids = HashSet::new();
        for event in &node.events {
            if !contract.events.contains(&event.event_id) || event.action_id < 1 || !event_ids.insert(event.event_id) {
                return Err(ValidationError(format!("invalid event '{}' on node '{}'", event.event_id, node.node_id)));
            }
        }
        if !node.children.is_empty() && !contract.children {
            return Err(ValidationError(format!("component '{}' cannot have children", node.component_id)));
        }
        stack.extend(node.children.iter());
    }
    Ok(())
}

fn validate_call_graph(program: &Program, _runnables: &HashMap<String, usize>) -> Result<(), ValidationError> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    for instruction in &program.instructions {
        match instruction {
            Instruction::Agent(agent) => { graph.insert(agent.name.clone(), runs(&agent.body)); }
            Instruction::Workflow(wf) => { graph.insert(wf.name.clone(), runs(&wf.body)); }
            _ => continue,
        };
    }
    // DFS cycle detection
    let mut state: HashMap<String, u8> = graph.keys().map(|k| (k.clone(), 0u8)).collect();
    for root in graph.keys() {
        if state[root] != 0 { continue; }
        let mut dfs_stack: Vec<(String, Vec<String>)> = vec![(root.clone(), graph[root].iter().cloned().collect())];
        state.insert(root.clone(), 1);
        while let Some((name, deps)) = dfs_stack.last_mut() {
            if deps.is_empty() {
                state.insert(name.clone(), 2);
                dfs_stack.pop();
                continue;
            }
            let dep = deps.pop().unwrap();
            if state.get(&dep) == Some(&1) {
                return Err(ValidationError(format!("workflow cycle detected at '{dep}'")));
            }
            if state.get(&dep) == Some(&0) {
                if dfs_stack.len() >= MAX_CALL_DEPTH {
                    return Err(ValidationError(format!("workflow call depth exceeds {MAX_CALL_DEPTH}")));
                }
                state.insert(dep.clone(), 1);
                let next_deps = graph.get(&dep).cloned().unwrap_or_default().into_iter().collect();
                dfs_stack.push((dep, next_deps));
            }
        }
    }
    Ok(())
}

fn runs(body: &[Instruction]) -> HashSet<String> {
    let mut result = HashSet::new();
    for instruction in body {
        match instruction {
            Instruction::Run(name) => { result.insert(name.clone()); }
            Instruction::If(if_inst) => {
                result.extend(runs(&if_inst.body));
                result.extend(runs(&if_inst.else_body));
            }
            Instruction::While(w) => { result.extend(runs(&w.body)); }
            _ => {}
        }
    }
    result
}
