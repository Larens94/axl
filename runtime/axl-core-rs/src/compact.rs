use crate::ir::*;
use crate::type_names::MAX_TYPE_DEPTH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactParseError(pub String);

impl std::fmt::Display for CompactParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompactParseError {}

const MAX_CALL_ARITY: usize = 65_535;
const MAX_SOURCE_BYTES: usize = 1_000_000;

const OPERATORS: &[&str] = &["+", "-", "*", "/", "=", "!", ">", "<", "G", "L"];
const TYPE_NAMES: &[(&str, &str)] = &[("i", "int"), ("s", "string"), ("b", "bool")];
const TYPE_CODES: &[(&str, &str)] = &[("int", "i"), ("string", "s"), ("bool", "b")];
const OPERATOR_NAMES: &[(&str, &str)] = &[("=", "=="), ("!", "!="), ("G", ">="), ("L", "<=")];

fn operator_display(op: &str) -> &str {
    for (code, name) in OPERATOR_NAMES {
        if *code == op { return name; }
    }
    op
}

fn operator_code(op: &str) -> &str {
    for (code, name) in OPERATOR_NAMES {
        if *name == op { return code; }
    }
    op
}

fn type_code(type_name: &str) -> Result<String, CompactParseError> {
    if type_name.starts_with("list<") && type_name.ends_with('>') {
        return Ok(format!("l{}", type_code(&type_name[5..type_name.len()-1])?));
    }
    if type_name.starts_with("map<") && type_name.ends_with('>') {
        let inner = &type_name[4..type_name.len()-1];
        let (kt, vt) = split_map_type_inner(inner)?;
        return Ok(format!("m{}{}", type_code(kt)?, type_code(vt)?));
    }
    for (name, code) in TYPE_CODES {
        if *name == type_name { return Ok(code.to_string()); }
    }
    Err(CompactParseError(format!("cannot encode type '{type_name}'")))
}

fn type_name_from_code(code: &str) -> Result<String, CompactParseError> {
    let (name, pos) = type_name_prefix(code, 0, 0)?;
    if pos != code.len() {
        return Err(CompactParseError(format!("invalid type '{code}'")));
    }
    Ok(name)
}

fn type_name_prefix(code: &str, position: usize, depth: usize) -> Result<(String, usize), CompactParseError> {
    if position >= code.len() {
        return Err(CompactParseError("incomplete type".into()));
    }
    let bytes = code.as_bytes();
    match bytes[position] {
        b'l' => {
            if depth >= MAX_TYPE_DEPTH {
                return Err(CompactParseError(format!("type nesting is too deep ({MAX_TYPE_DEPTH})")));
            }
            let (item_type, pos) = type_name_prefix(code, position + 1, depth + 1)?;
            Ok((format!("list<{item_type}>"), pos))
        }
        b'm' => {
            if depth >= MAX_TYPE_DEPTH {
                return Err(CompactParseError(format!("type nesting is too deep ({MAX_TYPE_DEPTH})")));
            }
            let (key_type, pos) = type_name_prefix(code, position + 1, depth + 1)?;
            let (value_type, pos) = type_name_prefix(code, pos, depth + 1)?;
            Ok((format!("map<{key_type},{value_type}>"), pos))
        }
        _ => {
            for (tcode, tname) in TYPE_NAMES {
                if code[position..].starts_with(tcode) {
                    return Ok((tname.to_string(), position + tcode.len()));
                }
            }
            Err(CompactParseError(format!("invalid type '{}'", &code[position..])))
        }
    }
}

fn split_map_type_inner(source: &str) -> Result<(&str, &str), CompactParseError> {
    let mut depth = 0;
    for (i, ch) in source.chars().enumerate() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Ok((&source[..i], &source[i+1..])),
            _ => {}
        }
    }
    Err(CompactParseError("invalid map type".into()))
}

pub fn is_compact_source(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with('2') || trimmed.starts_with('3')
}

pub fn program_to_compact(program: &Program) -> Result<String, CompactParseError> {
    let version = if has_v3_features(program) { "3" } else { "2" };
    let mut frames = vec![version.to_string()];
    for instruction in &program.instructions {
        instruction_frames(instruction, &mut frames)?;
    }
    let source = frames.join(";");
    if source.len() > MAX_SOURCE_BYTES {
        return Err(CompactParseError(format!("source exceeds {MAX_SOURCE_BYTES} bytes")));
    }
    Ok(source)
}

fn has_v3_features(program: &Program) -> bool {
    program.instructions.iter().any(|i| matches!(i, Instruction::Annotation(_) | Instruction::UiView(_)))
}

fn instruction_frames(instruction: &Instruction, frames: &mut Vec<String>) -> Result<(), CompactParseError> {
    match instruction {
        Instruction::Let { target, value, type_name } => {
            let mut frame = format!("10|{target}|{}", expression_source(value)?);
            if let Some(tn) = type_name {
                frame.push_str(&format!("|{}", type_code(tn)?));
            }
            frames.push(frame);
        }
        Instruction::Return(expr) => frames.push(format!("11|{}", expression_source(expr)?)),
        Instruction::Emit(expr) => frames.push(format!("12|{}", expression_source(expr)?)),
        Instruction::MemoryWrite(mw) => {
            let mut frame = format!("20|{}|{}", mw.key, expression_source(&mw.value)?);
            if (mw.confidence, mw.ttl_seconds, mw.source.as_str()) != (100, None, "program") {
                let ttl = match mw.ttl_seconds { None => "-".into(), Some(v) => v.to_string() };
                frame.push_str(&format!("|{}|{}|{}", mw.confidence, ttl, mw.source));
            }
            frames.push(frame);
        }
        Instruction::Forget(key) => frames.push(format!("21|{key}")),
        Instruction::If(if_inst) => {
            frames.push(format!("30|{}", expression_source(&if_inst.condition)?));
            for child in &if_inst.body { instruction_frames(child, frames)?; }
            if !if_inst.else_body.is_empty() {
                frames.push("31".into());
                for child in &if_inst.else_body { instruction_frames(child, frames)?; }
            }
            frames.push("99".into());
        }
        Instruction::While(while_inst) => {
            frames.push(format!("32|{}", expression_source(&while_inst.condition)?));
            for child in &while_inst.body { instruction_frames(child, frames)?; }
            frames.push("99".into());
        }
        Instruction::Function(func) => {
            let params: Vec<String> = func.parameters.iter()
                .map(|p| format!("{}:{}", p.name, type_code(&p.type_name).unwrap_or_default()))
                .collect();
            frames.push(format!("40|{}|{}|{}", func.name, params.join(","), type_code(&func.return_type)?));
            for child in &func.body { instruction_frames(child, frames)?; }
            frames.push("99".into());
        }
        Instruction::Agent(agent) => {
            let mut frame = format!("50|{}", agent.name);
            if !agent.tools.is_empty() {
                frame.push_str(&format!("|{}", agent.tools.join(",")));
            }
            frames.push(frame);
            for child in &agent.body { instruction_frames(child, frames)?; }
            frames.push("99".into());
        }
        Instruction::Workflow(wf) => {
            frames.push(format!("51|{}", wf.name));
            for child in &wf.body { instruction_frames(child, frames)?; }
            frames.push("99".into());
        }
        Instruction::Run(name) => frames.push(format!("52|{name}")),
        Instruction::Annotation(ann) => {
            let value = serde_json::to_string(&ann.value).unwrap_or_default();
            frames.push(format!("80|{}|{}|{}", ann.kind, ann.target, value));
        }
        Instruction::UiView(view) => {
            frames.push(format!("60|{}", view.view_id));
            ui_node_frames(&view.root, frames);
        }
        // New 3.0 instructions — not yet supported in compact format
        _ => {}
    }
    Ok(())
}

fn ui_node_frames(node: &UiNode, frames: &mut Vec<String>) {
    frames.push(format!("61|{}|{}", node.node_id, node.component_id));
    for prop in &node.properties {
        frames.push(format!("62|{}|{}", prop.property_id, expression_source(&prop.value).unwrap_or_default()));
    }
    for event in &node.events {
        frames.push(format!("63|{}|{}", event.event_id, event.action_id));
    }
    for child in &node.children {
        ui_node_frames(child, frames);
    }
    frames.push("99".into());
}

fn expression_source(expr: &Expression) -> Result<String, CompactParseError> {
    match expr {
        Expression::Literal(Value::Bool(b)) => Ok(if *b { "?1".into() } else { "?0".into() }),
        Expression::Literal(Value::Int(n)) => Ok(format!("#{n}")),
        Expression::Literal(Value::String(s)) => {
            require_unicode_scalar(s)?;
            Ok(serde_json::to_string(s).unwrap_or_default())
        }
        Expression::Literal(_) => Err(CompactParseError("cannot encode literal".into())),
        Expression::Variable(name) => Ok(format!("${name}")),
        Expression::Recall(key) => Ok(format!("@{key}")),
        Expression::Binary { left, operator, right } => {
            let op = operator_code(operator);
            Ok(format!("{},{},{}", expression_source(left)?, expression_source(right)?, op))
        }
        Expression::ToolCall { name, arguments } => {
            require_call_arity(arguments.len())?;
            let mut parts: Vec<String> = arguments.iter().map(|a| expression_source(a)).collect::<Result<_, _>>()?;
            parts.push(format!("!{name}/{}", arguments.len()));
            Ok(parts.join(","))
        }
        Expression::FunctionCall { name, arguments } => {
            require_call_arity(arguments.len())?;
            let mut parts: Vec<String> = arguments.iter().map(|a| expression_source(a)).collect::<Result<_, _>>()?;
            parts.push(format!("^{name}/{}", arguments.len()));
            Ok(parts.join(","))
        }
        Expression::ListExpression(items) => {
            require_call_arity(items.len())?;
            let mut parts: Vec<String> = items.iter().map(|a| expression_source(a)).collect::<Result<_, _>>()?;
            parts.push(format!("~{}", items.len()));
            Ok(parts.join(","))
        }
        Expression::MapExpression(entries) => {
            require_call_arity(entries.len())?;
            let mut parts = Vec::new();
            for (k, v) in entries {
                parts.push(expression_source(k)?);
                parts.push(expression_source(v)?);
            }
            parts.push(format!("%{}", entries.len()));
            Ok(parts.join(","))
        }
        // New 3.0 expressions — not yet supported in compact format
        _ => Err(CompactParseError("cannot encode agent-native expression in compact format".into())),
    }
}

fn require_call_arity(arity: usize) -> Result<(), CompactParseError> {
    if arity > MAX_CALL_ARITY { Err(CompactParseError("invalid call arity".into())) } else { Ok(()) }
}

fn require_unicode_scalar(_value: &str) -> Result<(), CompactParseError> {
    Ok(()) // valid UTF-8 is guaranteed by Rust strings
}

pub fn parse_compact(source: &str) -> Result<Program, CompactParseError> {
    let frames = split_compact_frames(source)?;
    if frames.is_empty() || (frames[0] != "2" && frames[0] != "3") {
        return Err(CompactParseError("compact source requires version header '2' or '3'".into()));
    }
    let version = frames[0].clone();
    let (instructions, position, terminator) = parse_block(&frames, 1, false, &version)?;
    if terminator.is_some() {
        return Err(CompactParseError(format!("frame {}: unexpected opcode", position)));
    }
    Ok(Program { instructions })
}

pub fn split_compact_frames(source: &str) -> Result<Vec<String>, CompactParseError> {
    let cleaned = remove_unquoted_whitespace(source)?;
    split_quoted(&cleaned, ';')
}

fn parse_block(
    frames: &[String], mut position: usize, allow_else: bool, version: &str,
) -> Result<(Vec<Instruction>, usize, Option<String>), CompactParseError> {
    let mut instructions = Vec::new();
    while position < frames.len() {
        let index = position;
        let frame = &frames[position];
        position += 1;
        if frame.is_empty() { continue; }
        let fields = split_quoted(frame, '|')?;
        let opcode = fields[0].as_str();
        match opcode {
            "99" => return Ok((instructions, position, Some("99".into()))),
            "31" if allow_else => return Ok((instructions, position, Some("31".into()))),
            "31" => return Err(CompactParseError(format!("frame {index}: unexpected else opcode"))),
            "10" if fields.len() == 3 || fields.len() == 4 => {
                let type_name = if fields.len() == 4 {
                    Some(type_name_from_code(&fields[3])
                        .map_err(|e| CompactParseError(format!("frame {index}: invalid binding type '{}': {}", fields[3], e)))?)
                } else { None };
                instructions.push(Instruction::Let {
                    target: fields[1].clone(),
                    value: parse_expression(&fields[2], index)?,
                    type_name,
                });
            }
            "12" if fields.len() == 2 => {
                instructions.push(Instruction::Emit(parse_expression(&fields[1], index)?));
            }
            "20" if fields.len() == 3 || fields.len() == 6 => {
                let mut confidence = 100i32;
                let mut ttl_seconds = None;
                let mut source = "program".to_string();
                if fields.len() == 6 {
                    confidence = fields[3].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid memory metadata")))?;
                    ttl_seconds = if fields[4] == "-" { None } else { Some(fields[4].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid memory metadata")))?) };
                    source = fields[5].clone();
                }
                instructions.push(Instruction::MemoryWrite(MemoryWrite {
                    key: fields[1].clone(),
                    value: parse_expression(&fields[2], index)?,
                    confidence, ttl_seconds, source, tags: vec![],
                }));
            }
            "21" if fields.len() == 2 => {
                instructions.push(Instruction::Forget(fields[1].clone()));
            }
            "30" if fields.len() == 2 => {
                let (body, mut pos, term) = parse_block(frames, position, true, version)?;
                if term.is_none() {
                    return Err(CompactParseError(format!("frame {index}: if missing end opcode 99")));
                }
                let mut else_body = Vec::new();
                if term.as_deref() == Some("31") {
                    let (eb, pos2, term2) = parse_block(frames, pos, false, version)?;
                    if term2.as_deref() != Some("99") {
                        return Err(CompactParseError(format!("frame {index}: if missing end opcode 99")));
                    }
                    else_body = eb;
                    pos = pos2;
                }
                position = pos;
                instructions.push(Instruction::If(If {
                    condition: parse_expression(&fields[1], index)?,
                    body, else_body,
                }));
            }
            "32" if fields.len() == 2 => {
                let (body, pos, term) = parse_block(frames, position, false, version)?;
                if term.as_deref() != Some("99") {
                    return Err(CompactParseError(format!("frame {index}: while missing end opcode 99")));
                }
                position = pos;
                instructions.push(Instruction::While(While {
                    condition: parse_expression(&fields[1], index)?,
                    body,
                }));
            }
            "40" if fields.len() == 4 => {
                let mut parameters = Vec::new();
                if !fields[2].is_empty() {
                    for raw in fields[2].split(',') {
                        let (name, rest) = raw.split_once(':')
                            .ok_or_else(|| CompactParseError(format!("frame {index}: invalid function parameter '{raw}'")))?;
                        let ptype = type_name_from_code(rest)
                            .map_err(|e| CompactParseError(format!("frame {index}: invalid function parameter '{raw}': {e}")))?;
                        parameters.push(Parameter { name: name.to_string(), type_name: ptype });
                    }
                }
                let return_type = type_name_from_code(&fields[3])
                    .map_err(|e| CompactParseError(format!("frame {index}: invalid return type '{}': {}", fields[3], e)))?;
                let (body, pos, term) = parse_block(frames, position, false, version)?;
                if term.as_deref() != Some("99") {
                    return Err(CompactParseError(format!("frame {index}: function missing end opcode 99")));
                }
                position = pos;
                instructions.push(Instruction::Function(Function {
                    name: fields[1].clone(), parameters, return_type, body,
                }));
            }
            "11" if fields.len() == 2 => {
                instructions.push(Instruction::Return(parse_expression(&fields[1], index)?));
            }
            "50" if fields.len() == 2 || fields.len() == 3 => {
                let tools = if fields.len() == 3 {
                    fields[2].split(',').filter(|s| !s.is_empty()).map(String::from).collect()
                } else { Vec::new() };
                let (body, pos, term) = parse_block(frames, position, false, version)?;
                if term.as_deref() != Some("99") {
                    return Err(CompactParseError(format!("frame {index}: agent missing end opcode 99")));
                }
                position = pos;
                instructions.push(Instruction::Agent(Agent {
                    name: fields[1].clone(), tools, body,
                    goal: None, tool_defs: vec![], memory_defs: vec![], handlers: vec![],
                }));
            }
            "51" if fields.len() == 2 => {
                let (body, pos, term) = parse_block(frames, position, false, version)?;
                if term.as_deref() != Some("99") {
                    return Err(CompactParseError(format!("frame {index}: workflow missing end opcode 99")));
                }
                position = pos;
                instructions.push(Instruction::Workflow(Workflow { name: fields[1].clone(), body, handlers: vec![] }));
            }
            "52" if fields.len() == 2 => {
                instructions.push(Instruction::Run(fields[1].clone()));
            }
            "80" if version == "3" && fields.len() == 4 => {
                let kind: i32 = fields[1].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid annotation")))?;
                let target: i32 = fields[2].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid annotation")))?;
                let value: String = serde_json::from_str(&fields[3]).map_err(|_| CompactParseError(format!("frame {index}: invalid annotation")))?;
                instructions.push(Instruction::Annotation(Annotation { kind, target, value }));
            }
            "60" if version == "3" && fields.len() == 2 => {
                let view_id: i32 = fields[1].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid view id")))?;
                let (root, pos) = parse_ui_node(frames, position)?;
                position = pos;
                instructions.push(Instruction::UiView(UiView { view_id, root }));
            }
            _ => return Err(CompactParseError(format!("frame {index}: invalid opcode or arity '{opcode}'"))),
        }
    }
    Ok((instructions, position, None))
}

fn parse_ui_node(frames: &[String], position: usize) -> Result<(UiNode, usize), CompactParseError> {
    if position >= frames.len() {
        return Err(CompactParseError("UI view missing root node".into()));
    }
    let index = position;
    let fields = split_quoted(&frames[position], '|')?;
    let mut position = position + 1;
    if fields.len() != 3 || fields[0] != "61" {
        return Err(CompactParseError(format!("frame {index}: UI view requires node opcode 61")));
    }
    let node_id: i32 = fields[1].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid UI node")))?;
    let component_id: i32 = fields[2].parse().map_err(|_| CompactParseError(format!("frame {index}: invalid UI node")))?;
    let mut properties = Vec::new();
    let mut events = Vec::new();
    let mut children = Vec::new();
    while position < frames.len() {
        let child_index = position;
        let child_fields = split_quoted(&frames[position], '|')?;
        match child_fields[0].as_str() {
            "99" => return Ok((UiNode { node_id, component_id, properties, events, children }, position + 1)),
            "62" if child_fields.len() == 3 => {
                let pid: i32 = child_fields[1].parse().map_err(|_| CompactParseError(format!("frame {child_index}: invalid UI property")))?;
                properties.push(UiProperty { property_id: pid, value: parse_expression(&child_fields[2], child_index)? });
                position += 1;
            }
            "63" if child_fields.len() == 3 => {
                let eid: i32 = child_fields[1].parse().map_err(|_| CompactParseError(format!("frame {child_index}: invalid UI event")))?;
                let aid: i32 = child_fields[2].parse().map_err(|_| CompactParseError(format!("frame {child_index}: invalid UI event")))?;
                events.push(UiEvent { event_id: eid, action_id: aid });
                position += 1;
            }
            "61" => {
                let (child, pos) = parse_ui_node(frames, position)?;
                children.push(child);
                position = pos;
            }
            other => return Err(CompactParseError(format!("frame {child_index}: invalid UI opcode or arity '{other}'"))),
        }
    }
    Err(CompactParseError(format!("frame {index}: UI node missing end opcode 99")))
}

fn parse_expression(source: &str, frame: usize) -> Result<Expression, CompactParseError> {
    let tokens = split_quoted(source, ',')?;
    let mut stack: Vec<Expression> = Vec::new();
    for token in &tokens {
        if let Some(cap) = regex_captures(r"^%(\d+)$", token) {
            let arity: usize = cap.parse().map_err(|_| CompactParseError(format!("frame {frame}: invalid map arity")))?;
            let width = arity * 2;
            if arity > MAX_CALL_ARITY || stack.len() < width {
                return Err(CompactParseError(format!("frame {frame}: invalid map arity")));
            }
            let entries: Vec<_> = stack.drain(stack.len()-width..).collect();
            let pairs: Vec<_> = entries.chunks(2).map(|c| (c[0].clone(), c[1].clone())).collect();
            stack.push(Expression::MapExpression(pairs));
        } else if let Some(cap) = regex_captures(r"^~(\d+)$", token) {
            let arity: usize = cap.parse().map_err(|_| CompactParseError(format!("frame {frame}: invalid list arity")))?;
            if arity > MAX_CALL_ARITY || stack.len() < arity {
                return Err(CompactParseError(format!("frame {frame}: invalid list arity")));
            }
            let items: Vec<_> = stack.drain(stack.len()-arity..).collect();
            stack.push(Expression::ListExpression(items));
        } else if let Some((kind, name, arity_str)) = parse_call_token(token) {
            let arity: usize = arity_str.parse().map_err(|_| CompactParseError(format!("frame {frame}: invalid call arity")))?;
            if arity > MAX_CALL_ARITY || stack.len() < arity {
                return Err(CompactParseError(format!("frame {frame}: call '{token}' needs {arity} values")));
            }
            let arguments: Vec<_> = stack.drain(stack.len()-arity..).collect();
            let expr = if kind == '!' {
                Expression::ToolCall { name: name.to_string(), arguments }
            } else {
                Expression::FunctionCall { name: name.to_string(), arguments }
            };
            stack.push(expr);
        } else if OPERATORS.iter().any(|op| *op == token.as_str()) {
            if stack.len() < 2 {
                return Err(CompactParseError(format!("frame {frame}: operator '{token}' needs two values")));
            }
            let right = stack.pop().unwrap();
            let left = stack.pop().unwrap();
            stack.push(Expression::Binary {
                left: Box::new(left),
                operator: operator_display(token).to_string(),
                right: Box::new(right),
            });
        } else if let Some(n) = token.strip_prefix('#') {
            let val: i64 = n.parse().map_err(|_| CompactParseError(format!("frame {frame}: invalid integer '{token}'")))?;
            stack.push(Expression::Literal(Value::Int(val)));
        } else if let Some(name) = token.strip_prefix('$') {
            if name.is_empty() { return Err(CompactParseError(format!("frame {frame}: invalid variable"))); }
            stack.push(Expression::Variable(name.to_string()));
        } else if let Some(key) = token.strip_prefix('@') {
            if key.is_empty() { return Err(CompactParseError(format!("frame {frame}: invalid recall"))); }
            stack.push(Expression::Recall(key.to_string()));
        } else if token.starts_with('"') {
            let val: String = serde_json::from_str(token)
                .map_err(|_| CompactParseError(format!("frame {frame}: invalid string")))?;
            stack.push(Expression::Literal(Value::String(val)));
        } else if token == "?1" {
            stack.push(Expression::Literal(Value::Bool(true)));
        } else if token == "?0" {
            stack.push(Expression::Literal(Value::Bool(false)));
        } else {
            return Err(CompactParseError(format!("frame {frame}: invalid expression token '{token}'")));
        }
    }
    if stack.len() != 1 {
        return Err(CompactParseError(format!("frame {frame}: expression leaves {} values", stack.len())));
    }
    Ok(stack.pop().unwrap())
}

fn parse_call_token(token: &str) -> Option<(char, &str, &str)> {
    let bytes = token.as_bytes();
    if bytes.len() < 3 || (bytes[0] != b'!' && bytes[0] != b'^') { return None; }
    let kind = bytes[0] as char;
    let rest = &token[1..];
    let slash = rest.rfind('/')?;
    if slash == 0 || slash >= rest.len() - 1 { return None; }
    let name = &rest[..slash];
    let arity = &rest[slash+1..];
    if !arity.bytes().all(|b| b.is_ascii_digit()) { return None; }
    if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.') { return None; }
    Some((kind, name, arity))
}

fn regex_captures(pattern: &str, text: &str) -> Option<String> {
    // Simple pattern: ^X(\d+)$
    if pattern.starts_with('^') && pattern.ends_with('$') {
        let inner = &pattern[1..pattern.len()-1];
        if let Some(paren_start) = inner.find('(') {
            let prefix = &inner[..paren_start];
            let suffix_start = inner.rfind(')').map(|p| p + 1)?;
            let suffix = &inner[suffix_start..];
            if text.starts_with(prefix) && text.ends_with(suffix) {
                let cap = &text[prefix.len()..text.len()-suffix.len()];
                if !cap.is_empty() { return Some(cap.to_string()); }
            }
        }
    }
    None
}

fn split_quoted(source: &str, delimiter: char) -> Result<Vec<String>, CompactParseError> {
    let mut values = Vec::new();
    let mut value = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for ch in source.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            value.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
            value.push(ch);
            continue;
        }
        if ch == delimiter && !quoted {
            values.push(value);
            value = String::new();
        } else {
            value.push(ch);
        }
    }
    if quoted || escaped {
        return Err(CompactParseError("unterminated string".into()));
    }
    values.push(value);
    Ok(values)
}

fn remove_unquoted_whitespace(source: &str) -> Result<String, CompactParseError> {
    let mut result = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for ch in source.chars() {
        if escaped {
            result.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            result.push(ch);
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
            result.push(ch);
        } else if quoted || !ch.is_whitespace() {
            result.push(ch);
        }
    }
    if quoted || escaped {
        return Err(CompactParseError("unterminated string".into()));
    }
    Ok(result)
}
