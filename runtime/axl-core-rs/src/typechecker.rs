use std::collections::HashMap;

use crate::ir::*;
use crate::type_names;

const ANY: &str = "any";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCheckError(pub String);

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TypeCheckError {}

struct FunctionSignature {
    parameters: Vec<String>,
    return_type: String,
}

pub fn typecheck(program: &Program) -> Result<(), TypeCheckError> {
    let mut functions: HashMap<String, &Function> = HashMap::new();
    let mut signatures: HashMap<String, FunctionSignature> = HashMap::new();

    for instruction in &program.instructions {
        if let Instruction::Function(func) = instruction {
            if functions.contains_key(&func.name) {
                return Err(TypeCheckError(format!("duplicate function '{}'", func.name)));
            }
            for param in &func.parameters {
                require_known_type(&param.type_name)?;
            }
            require_known_type(&func.return_type)?;
            let param_names: Vec<String> = func.parameters.iter().map(|p| p.name.clone()).collect();
            if param_names.len() != param_names.iter().collect::<std::collections::HashSet<_>>().len() {
                return Err(TypeCheckError(format!("duplicate parameter in function '{}'", func.name)));
            }
            functions.insert(func.name.clone(), func);
            signatures.insert(func.name.clone(), FunctionSignature {
                parameters: func.parameters.iter().map(|p| p.type_name.clone()).collect(),
                return_type: func.return_type.clone(),
            });
        }
    }

    for func in functions.values() {
        let mut env: HashMap<String, String> = func.parameters.iter()
            .map(|p| (p.name.clone(), p.type_name.clone())).collect();
        let mut return_types = Vec::new();
        check_block(&func.body, &mut env, &signatures, &mut return_types, Some(&func.name))?;
        for rt in &return_types {
            if !compatible(&func.return_type, rt) {
                return Err(TypeCheckError(format!(
                    "function '{}' must return {}, got {}", func.name, func.return_type, rt
                )));
            }
        }
        if !always_returns(&func.body) {
            return Err(TypeCheckError(format!("function '{}' may complete without returning", func.name)));
        }
    }

    let mut env = HashMap::new();
    check_block(&program.instructions, &mut env, &signatures, &mut Vec::new(), None)?;
    Ok(())
}

fn check_block(
    instructions: &[Instruction],
    env: &mut HashMap<String, String>,
    signatures: &HashMap<String, FunctionSignature>,
    return_types: &mut Vec<String>,
    function_name: Option<&str>,
) -> Result<(), TypeCheckError> {
    for instruction in instructions {
        match instruction {
            Instruction::Function(_) | Instruction::UiView(_) => {}
            Instruction::Let { target, value, type_name } => {
                if let Some(tn) = type_name { require_known_type(tn)?; }
                let value_type = expression_type(value, env, signatures)?;
                if let Some(tn) = type_name {
                    if !compatible(tn, &value_type) {
                        return Err(TypeCheckError(format!("variable '{target}' must be {tn}, got {value_type}")));
                    }
                }
                env.insert(target.clone(), type_name.clone().unwrap_or(value_type));
            }
            Instruction::Return(expr) => {
                let fname = function_name.ok_or_else(|| TypeCheckError("return is only valid inside a function".into()))?;
                let _ = fname;
                return_types.push(expression_type(expr, env, signatures)?);
            }
            Instruction::Emit(expr) | Instruction::MemoryWrite(MemoryWrite { value: expr, .. }) => {
                expression_type(expr, env, signatures)?;
            }
            Instruction::If(if_inst) => {
                let ct = expression_type(&if_inst.condition, env, signatures)?;
                if ct != "bool" && ct != ANY {
                    return Err(TypeCheckError("if condition must be bool".into()));
                }
                let mut body_env = env.clone();
                check_block(&if_inst.body, &mut body_env, signatures, return_types, function_name)?;
                let mut else_env = env.clone();
                check_block(&if_inst.else_body, &mut else_env, signatures, return_types, function_name)?;
            }
            Instruction::While(w) => {
                let ct = expression_type(&w.condition, env, signatures)?;
                if ct != "bool" && ct != ANY {
                    return Err(TypeCheckError("while condition must be bool".into()));
                }
                let mut body_env = env.clone();
                check_block(&w.body, &mut body_env, signatures, return_types, function_name)?;
            }
            Instruction::Agent(agent) => {
                let mut agent_env = HashMap::new();
                check_block(&agent.body, &mut agent_env, signatures, return_types, function_name)?;
            }
            Instruction::Workflow(wf) => {
                let mut wf_env = HashMap::new();
                check_block(&wf.body, &mut wf_env, signatures, return_types, function_name)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn expression_type(
    expr: &Expression,
    env: &HashMap<String, String>,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<String, TypeCheckError> {
    match expr {
        Expression::Literal(val) => Ok(match val {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            _ => ANY,
        }.to_string()),
        Expression::ListExpression(items) => {
            let item_types: Vec<String> = items.iter().map(|i| expression_type(i, env, signatures)).collect::<Result<_, _>>()?;
            let item_type = if item_types.is_empty() { ANY.to_string() } else { unify_types(&item_types, "list items")? };
            Ok(format!("list<{item_type}>"))
        }
        Expression::MapExpression(entries) => {
            let key_types: Vec<String> = entries.iter().map(|(k, _)| expression_type(k, env, signatures)).collect::<Result<_, _>>()?;
            let val_types: Vec<String> = entries.iter().map(|(_, v)| expression_type(v, env, signatures)).collect::<Result<_, _>>()?;
            let key_type = if key_types.is_empty() { ANY.to_string() } else { unify_types(&key_types, "map keys")? };
            let val_type = if val_types.is_empty() { ANY.to_string() } else { unify_types(&val_types, "map values")? };
            if !["int", "string", "bool", ANY].contains(&key_type.as_str()) {
                return Err(TypeCheckError("map keys must be scalar".into()));
            }
            Ok(format!("map<{key_type},{val_type}>"))
        }
        Expression::Variable(name) => {
            env.get(name).cloned().ok_or_else(|| TypeCheckError(format!("unknown variable '{name}'")))
        }
        Expression::Recall(_) | Expression::ToolCall { .. } => Ok(ANY.to_string()),
        Expression::FunctionCall { name, arguments } => {
            let sig = signatures.get(name).ok_or_else(|| TypeCheckError(format!("unknown function '{name}'")))?;
            if arguments.len() != sig.parameters.len() {
                return Err(TypeCheckError(format!(
                    "function '{name}' expects {} arguments, got {}", sig.parameters.len(), arguments.len()
                )));
            }
            for (i, (arg, expected)) in arguments.iter().zip(sig.parameters.iter()).enumerate() {
                let actual = expression_type(arg, env, signatures)?;
                if !compatible(expected, &actual) {
                    return Err(TypeCheckError(format!("argument {} of '{name}' must be {expected}, got {actual}", i + 1)));
                }
            }
            Ok(sig.return_type.clone())
        }
        Expression::Binary { left, operator, right } => {
            let lt = expression_type(left, env, signatures)?;
            let rt = expression_type(right, env, signatures)?;
            if lt == ANY || rt == ANY { return Ok(ANY.to_string()); }
            match operator.as_str() {
                "==" | "!=" => {
                    if lt != rt { return Err(TypeCheckError(format!("operator '{operator}' requires matching types"))); }
                    Ok("bool".to_string())
                }
                ">" | "<" | ">=" | "<=" => {
                    if lt != "int" || rt != "int" { return Err(TypeCheckError(format!("operator '{operator}' requires int operands"))); }
                    Ok("bool".to_string())
                }
                "+" if lt == "string" && rt == "string" => Ok("string".to_string()),
                _ => {
                    if lt != "int" || rt != "int" { return Err(TypeCheckError(format!("operator '{operator}' requires int operands"))); }
                    Ok("int".to_string())
                }
            }
        }
    }
}

fn unify_types(types: &[String], context: &str) -> Result<String, TypeCheckError> {
    let mut result = ANY.to_string();
    for t in types {
        result = unify_pair(&result, t, context)?;
    }
    Ok(result)
}

fn unify_pair(left: &str, right: &str, context: &str) -> Result<String, TypeCheckError> {
    if left == ANY { return Ok(right.to_string()); }
    if right == ANY || left == right { return Ok(left.to_string()); }
    if let (Some(ll), Some(rl)) = (type_names::split_list_type(left), type_names::split_list_type(right)) {
        return Ok(format!("list<{}>", unify_pair(ll, rl, context)?));
    }
    if let (Some((lk, lv)), Some((rk, rv))) = (type_names::split_map_type(left), type_names::split_map_type(right)) {
        let k = unify_pair(lk, rk, context)?;
        let v = unify_pair(lv, rv, context)?;
        return Ok(format!("map<{k},{v}>"));
    }
    Err(TypeCheckError(format!("{context} must have one type")))
}

fn compatible(expected: &str, actual: &str) -> bool {
    if expected == actual || expected == ANY || actual == ANY { return true; }
    if let (Some((ek, ev)), Some((ak, av))) = (type_names::split_map_type(expected), type_names::split_map_type(actual)) {
        return compatible(ek, ak) && compatible(ev, av);
    }
    if let (Some(ei), Some(ai)) = (type_names::split_list_type(expected), type_names::split_list_type(actual)) {
        return compatible(ei, ai);
    }
    false
}

fn require_known_type(type_name: &str) -> Result<(), TypeCheckError> {
    if !type_names::is_known_type_name(type_name) {
        return Err(TypeCheckError(format!("unknown type '{type_name}'")));
    }
    Ok(())
}

fn always_returns(instructions: &[Instruction]) -> bool {
    for instruction in instructions {
        match instruction {
            Instruction::Return(_) => return true,
            Instruction::If(if_inst) if !if_inst.else_body.is_empty()
                && always_returns(&if_inst.body) && always_returns(&if_inst.else_body) => return true,
            _ => {}
        }
    }
    false
}
