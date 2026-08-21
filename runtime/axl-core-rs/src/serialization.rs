use serde_json::{json, Map, Value as Json};

use crate::ir::*;
use crate::validation;

pub const IR_VERSION: &str = "1.2";
pub const MAX_IR_BYTES: usize = 2_000_000;

#[derive(Debug)]
pub struct SerializationError(pub String);

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SerializationError {}

pub fn program_to_json(program: &Program) -> Result<String, SerializationError> {
    validation::validate(program).map_err(|e| SerializationError(e.to_string()))?;
    let doc = program_to_document_inner(program)?;
    serde_json::to_string_pretty(&doc).map_err(|e| SerializationError(e.to_string()))
}

pub fn program_to_document(program: &Program) -> Result<Json, SerializationError> {
    validation::validate(program).map_err(|e| SerializationError(e.to_string()))?;
    program_to_document_inner(program)
}

fn program_to_document_inner(program: &Program) -> Result<Json, SerializationError> {
    for inst in &program.instructions {
        if matches!(inst, Instruction::Annotation(_) | Instruction::UiView(_)) {
            return Err(SerializationError("AX-UI experimental nodes are not available in AX-IR 1.2".into()));
        }
    }
    let encoded = encode_program(program);
    Ok(json!({"ir_version": IR_VERSION, "program": encoded}))
}

pub fn program_from_json(payload: &str) -> Result<Program, SerializationError> {
    if payload.len() > MAX_IR_BYTES {
        return Err(SerializationError(format!("IR payload exceeds {MAX_IR_BYTES} bytes")));
    }
    let doc: Json = serde_json::from_str(payload)
        .map_err(|e| SerializationError(format!("invalid JSON: {e}")))?;
    let obj = doc.as_object().ok_or_else(|| SerializationError("IR document must be an object".into()))?;
    if !obj.contains_key("ir_version") || !obj.contains_key("program") {
        return Err(SerializationError("invalid IR envelope fields".into()));
    }
    let version = obj["ir_version"].as_str().ok_or_else(|| SerializationError("IR version must be a string".into()))?;
    if version != "1.0" && version != "1.1" && version != "1.2" {
        return Err(SerializationError(format!("unsupported IR version '{version}'")));
    }
    let mut payload_val = obj["program"].clone();
    if version == "1.0" { upgrade_1_0(&mut payload_val); }
    let program = decode_program(&payload_val).map_err(|e| SerializationError(e.to_string()))?;
    validation::validate(&program).map_err(|e| SerializationError(e.to_string()))?;
    Ok(program)
}

fn upgrade_1_0(val: &mut Json) {
    if let Json::Array(arr) = val {
        for item in arr.iter_mut() { upgrade_1_0(item); }
    } else if let Json::Object(obj) = val {
        for (_, v) in obj.iter_mut() { upgrade_1_0(v); }
        if obj.get("type").and_then(|v| v.as_str()) == Some("Let") && !obj.contains_key("type_name") {
            obj.insert("type_name".into(), Json::Null);
        }
    }
}

fn encode_program(program: &Program) -> Json {
    Json::Object({
        let mut m = Map::new();
        m.insert("type".into(), json!("Program"));
        m.insert("instructions".into(), Json::Array(program.instructions.iter().map(encode_instruction).collect()));
        m
    })
}

fn encode_instruction(inst: &Instruction) -> Json {
    match inst {
        Instruction::Let { target, value, type_name } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Let"));
            m.insert("target".into(), json!(target));
            m.insert("value".into(), encode_expression(value));
            m.insert("type_name".into(), match type_name {
                Some(t) => json!(t),
                None => Json::Null,
            });
            Json::Object(m)
        }
        Instruction::Return(expr) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Return"));
            m.insert("value".into(), encode_expression(expr));
            Json::Object(m)
        }
        Instruction::Emit(expr) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Emit"));
            m.insert("value".into(), encode_expression(expr));
            Json::Object(m)
        }
        Instruction::MemoryWrite(mw) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("MemoryWrite"));
            m.insert("key".into(), json!(mw.key));
            m.insert("value".into(), encode_expression(&mw.value));
            m.insert("confidence".into(), json!(mw.confidence));
            m.insert("ttl_seconds".into(), match mw.ttl_seconds { Some(v) => json!(v), None => Json::Null });
            m.insert("source".into(), json!(mw.source));
            Json::Object(m)
        }
        Instruction::Forget(key) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Forget"));
            m.insert("key".into(), json!(key));
            Json::Object(m)
        }
        Instruction::If(if_inst) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("If"));
            m.insert("condition".into(), encode_expression(&if_inst.condition));
            m.insert("body".into(), Json::Array(if_inst.body.iter().map(encode_instruction).collect()));
            m.insert("else_body".into(), Json::Array(if_inst.else_body.iter().map(encode_instruction).collect()));
            Json::Object(m)
        }
        Instruction::While(w) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("While"));
            m.insert("condition".into(), encode_expression(&w.condition));
            m.insert("body".into(), Json::Array(w.body.iter().map(encode_instruction).collect()));
            Json::Object(m)
        }
        Instruction::Agent(a) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Agent"));
            m.insert("name".into(), json!(a.name));
            m.insert("tools".into(), Json::Array(a.tools.iter().map(|t| json!(t)).collect()));
            m.insert("body".into(), Json::Array(a.body.iter().map(encode_instruction).collect()));
            Json::Object(m)
        }
        Instruction::Workflow(w) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Workflow"));
            m.insert("name".into(), json!(w.name));
            m.insert("body".into(), Json::Array(w.body.iter().map(encode_instruction).collect()));
            Json::Object(m)
        }
        Instruction::Run(name) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Run"));
            m.insert("name".into(), json!(name));
            Json::Object(m)
        }
        Instruction::Function(f) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Function"));
            m.insert("name".into(), json!(f.name));
            m.insert("parameters".into(), Json::Array(f.parameters.iter().map(|p| {
                let mut pm = Map::new();
                pm.insert("type".into(), json!("Parameter"));
                pm.insert("name".into(), json!(p.name));
                pm.insert("type_name".into(), json!(p.type_name));
                Json::Object(pm)
            }).collect()));
            m.insert("return_type".into(), json!(f.return_type));
            m.insert("body".into(), Json::Array(f.body.iter().map(encode_instruction).collect()));
            Json::Object(m)
        }
        Instruction::Annotation(_) | Instruction::UiView(_) => Json::Null,
        _ => Json::Null,
    }
}

fn encode_expression(expr: &Expression) -> Json {
    match expr {
        Expression::Literal(v) => encode_value(v),
        Expression::Variable(name) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Variable"));
            m.insert("name".into(), json!(name));
            Json::Object(m)
        }
        Expression::Recall(key) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Recall"));
            m.insert("key".into(), json!(key));
            Json::Object(m)
        }
        Expression::ToolCall { name, arguments } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("ToolCall"));
            m.insert("name".into(), json!(name));
            m.insert("arguments".into(), Json::Array(arguments.iter().map(encode_expression).collect()));
            Json::Object(m)
        }
        Expression::FunctionCall { name, arguments } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("FunctionCall"));
            m.insert("name".into(), json!(name));
            m.insert("arguments".into(), Json::Array(arguments.iter().map(encode_expression).collect()));
            Json::Object(m)
        }
        Expression::ListExpression(items) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("ListExpression"));
            m.insert("items".into(), Json::Array(items.iter().map(encode_expression).collect()));
            Json::Object(m)
        }
        Expression::MapExpression(entries) => {
            let mut m = Map::new();
            m.insert("type".into(), json!("MapExpression"));
            m.insert("entries".into(), Json::Array(entries.iter().map(|(k, v)| {
                Json::Array(vec![encode_expression(k), encode_expression(v)])
            }).collect()));
            Json::Object(m)
        }
        Expression::Binary { left, operator, right } => {
            let mut m = Map::new();
            m.insert("type".into(), json!("Binary"));
            m.insert("left".into(), encode_expression(left));
            m.insert("operator".into(), json!(operator));
            m.insert("right".into(), encode_expression(right));
            Json::Object(m)
        }
        _ => Json::Null,
    }
}

fn encode_value(val: &Value) -> Json {
    match val {
        Value::String(s) => json!(s),
        Value::Int(n) => json!(n),
        Value::Bool(b) => json!(b),
        Value::List(items) => Json::Array(items.iter().map(encode_value).collect()),
        Value::Map(entries) => Json::Array(entries.iter().map(|(k, v)| Json::Array(vec![encode_value(k), encode_value(v)])).collect()),
        _ => Json::Null,
    }
}

fn decode_program(val: &Json) -> Result<Program, String> {
    let obj = val.as_object().ok_or("Program must be an object")?;
    let instructions = obj.get("instructions").and_then(|v| v.as_array()).ok_or("missing instructions")?;
    Ok(Program {
        instructions: instructions.iter().map(|v| decode_instruction(v)).collect::<Result<_, _>>()?,
    })
}

fn decode_instruction(val: &Json) -> Result<Instruction, String> {
    let obj = val.as_object().ok_or("instruction must be an object")?;
    let typ = obj.get("type").and_then(|v| v.as_str()).ok_or("missing type")?;
    match typ {
        "Let" => Ok(Instruction::Let {
            target: obj.get("target").and_then(|v| v.as_str()).unwrap_or("").into(),
            value: decode_expression(obj.get("value"))?,
            type_name: obj.get("type_name").and_then(|v| v.as_str()).map(|s| s.into()),
        }),
        "Return" => Ok(Instruction::Return(decode_expression(obj.get("value"))?)),
        "Emit" => Ok(Instruction::Emit(decode_expression(obj.get("value"))?)),
        "MemoryWrite" => Ok(Instruction::MemoryWrite(MemoryWrite {
            key: obj.get("key").and_then(|v| v.as_str()).unwrap_or("").into(),
            value: decode_expression(obj.get("value"))?,
            confidence: obj.get("confidence").and_then(|v| v.as_i64()).unwrap_or(100) as i32,
            ttl_seconds: obj.get("ttl_seconds").and_then(|v| v.as_i64()),
            source: obj.get("source").and_then(|v| v.as_str()).unwrap_or("program").into(),
            tags: vec![],
        })),
        "Forget" => Ok(Instruction::Forget(obj.get("key").and_then(|v| v.as_str()).unwrap_or("").into())),
        "If" => Ok(Instruction::If(If {
            condition: decode_expression(obj.get("condition"))?,
            body: decode_instructions_array(obj.get("body"))?,
            else_body: decode_instructions_array(obj.get("else_body"))?,
        })),
        "While" => Ok(Instruction::While(While {
            condition: decode_expression(obj.get("condition"))?,
            body: decode_instructions_array(obj.get("body"))?,
        })),
        "Agent" => Ok(Instruction::Agent(Agent {
            name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
            tools: obj.get("tools").and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            body: decode_instructions_array(obj.get("body"))?,
            goal: obj.get("goal").and_then(|v| v.as_str()).map(|s| s.into()),
            tool_defs: vec![], memory_defs: vec![], handlers: vec![],
        })),
        "Workflow" => Ok(Instruction::Workflow(Workflow {
            name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
            body: decode_instructions_array(obj.get("body"))?,
            handlers: vec![],
        })),
        "Run" => Ok(Instruction::Run(obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into())),
        "Function" => {
            let params = obj.get("parameters").and_then(|v| v.as_array())
                .map(|a| a.iter().map(|p| {
                    let po = p.as_object().ok_or("parameter must be object")?;
                    Ok(Parameter {
                        name: po.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                        type_name: po.get("type_name").and_then(|v| v.as_str()).unwrap_or("").into(),
                    })
                }).collect::<Result<Vec<_>, String>>())
                .unwrap_or(Ok(vec![]))?;
            Ok(Instruction::Function(Function {
                name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                parameters: params,
                return_type: obj.get("return_type").and_then(|v| v.as_str()).unwrap_or("int").into(),
                body: decode_instructions_array(obj.get("body"))?,
            }))
        }
        _ => Err(format!("unknown instruction type '{typ}'")),
    }
}

fn decode_instructions_array(val: Option<&Json>) -> Result<Vec<Instruction>, String> {
    match val {
        Some(Json::Array(arr)) => arr.iter().map(|v| decode_instruction(v)).collect(),
        _ => Ok(vec![]),
    }
}

fn decode_expression(val: Option<&Json>) -> Result<Expression, String> {
    let val = val.ok_or("missing expression")?;
    match val {
        Json::String(s) => Ok(Expression::Literal(Value::String(s.clone()))),
        Json::Number(n) => Ok(Expression::Literal(Value::Int(n.as_i64().unwrap_or(0)))),
        Json::Bool(b) => Ok(Expression::Literal(Value::Bool(*b))),
        Json::Object(obj) => {
            let typ = obj.get("type").and_then(|v| v.as_str()).ok_or("missing expression type")?;
            match typ {
                "Variable" => Ok(Expression::Variable(obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into())),
                "Recall" => Ok(Expression::Recall(obj.get("key").and_then(|v| v.as_str()).unwrap_or("").into())),
                "ToolCall" => Ok(Expression::ToolCall {
                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                    arguments: decode_expression_array(obj.get("arguments"))?,
                }),
                "FunctionCall" => Ok(Expression::FunctionCall {
                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                    arguments: decode_expression_array(obj.get("arguments"))?,
                }),
                "ListExpression" => Ok(Expression::ListExpression(
                    decode_expression_array(obj.get("items"))?,
                )),
                "MapExpression" => {
                    let entries = obj.get("entries").and_then(|v| v.as_array())
                        .map(|a| a.iter().map(|pair| {
                            let arr = pair.as_array().ok_or("map entry must be array")?;
                            if arr.len() != 2 { return Err("map entry must have 2 elements".into()); }
                            Ok((decode_expression(Some(&arr[0]))?, decode_expression(Some(&arr[1]))?))
                        }).collect::<Result<Vec<_>, String>>())
                        .unwrap_or(Ok(vec![]))?;
                    Ok(Expression::MapExpression(entries))
                }
                "Binary" => Ok(Expression::Binary {
                    left: Box::new(decode_expression(obj.get("left"))?),
                    operator: obj.get("operator").and_then(|v| v.as_str()).unwrap_or("").into(),
                    right: Box::new(decode_expression(obj.get("right"))?),
                }),
                _ => Err(format!("unknown expression type '{typ}'")),
            }
        }
        Json::Null => Err("unexpected null expression".into()),
        Json::Array(_) => Err("unexpected array expression".into()),
    }
}

fn decode_expression_array(val: Option<&Json>) -> Result<Vec<Expression>, String> {
    match val {
        Some(Json::Array(arr)) => arr.iter().map(|v| decode_expression(Some(v))).collect(),
        _ => Ok(vec![]),
    }
}
