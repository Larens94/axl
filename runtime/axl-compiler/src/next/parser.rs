use super::ast::*;
use super::diagnostic::{Diagnostic, FixSafety, Repair, SourceSpan};

#[derive(Debug, Clone)]
struct SourceLine {
    number: usize,
    indent: usize,
    text: String,
    raw: String,
}

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let lines = source_lines(source, &mut diagnostics);
    let mut version = None;
    let mut app_name = None;
    let mut declarations = Vec::new();
    let mut cursor = 0;

    while cursor < lines.len() {
        let line = &lines[cursor];
        if line.indent != 0 {
            diagnostics.push(Diagnostic::error(
                "AXL-P004",
                "parse",
                "a declaration must begin at column 1",
                span(line),
            ));
            cursor += 1;
            continue;
        }

        let end = block_end(&lines, cursor);
        let body = &lines[cursor + 1..end];
        if let Some(value) = line.text.strip_prefix("axl ") {
            match value.trim().parse::<u16>() {
                Ok(4) => version = Some(4),
                Ok(found) => diagnostics.push(
                    Diagnostic::error(
                        "AXL-P001",
                        "parse",
                        "this compiler experiment accepts AXL version 4",
                        span(line),
                    )
                    .expected("4", found.to_string()),
                ),
                Err(_) => diagnostics.push(
                    Diagnostic::error(
                        "AXL-P001",
                        "parse",
                        "the language header is 'axl 4'",
                        span(line),
                    )
                    .expected("integer version", value.trim()),
                ),
            }
            if !body.is_empty() {
                diagnostics.push(unexpected_body(line, "axl header"));
            }
        } else if let Some(value) = line.text.strip_prefix("app ") {
            if value.trim().is_empty() {
                diagnostics.push(missing_name(line, "application"));
            } else {
                app_name = Some(value.trim().to_string());
            }
            if !body.is_empty() {
                diagnostics.push(unexpected_body(line, "app header"));
            }
        } else if line.text.starts_with("enum ") {
            parse_enum(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("entity ") {
            parse_entity(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("capacity ") {
            parse_capacity(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("skill ") {
            parse_skill(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("blueprint ") {
            parse_blueprint(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("instance ") {
            parse_instance(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("flow ") {
            parse_flow(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("api ") {
            parse_api(line, body, &mut declarations, &mut diagnostics);
        } else if line.text.starts_with("agent ") {
            parse_agent(line, body, &mut declarations, &mut diagnostics);
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P003",
                "parse",
                format!("unknown top-level declaration '{}'", line.text),
                span(line),
            ));
        }
        cursor = end;
    }

    if version.is_none() {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P001",
                "parse",
                "missing language header",
                SourceSpan::line(1, source.lines().next().unwrap_or("")),
            )
            .expected("axl 4", "end of file")
            .repair(
                FixSafety::Safe,
                Repair {
                    kind: "insert".into(),
                    target: "line 1".into(),
                    replacement: Some("axl 4".into()),
                    candidates: Vec::new(),
                },
            ),
        );
    }
    if app_name.is_none() {
        diagnostics.push(Diagnostic::error(
            "AXL-P002",
            "parse",
            "missing application declaration",
            SourceSpan::line(1, source.lines().next().unwrap_or("")),
        ));
    }

    if diagnostics.is_empty() {
        Ok(Program {
            version: version.unwrap_or(4),
            name: app_name.unwrap_or_default(),
            declarations,
        })
    } else {
        Err(diagnostics)
    }
}

fn parse_enum(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["enum ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "enum"));
        return;
    }
    let variants = body
        .iter()
        .filter_map(|line| {
            if line.text.split_whitespace().count() == 1 {
                Some(EnumVariant {
                    name: line.text.clone(),
                    span: span(line),
                })
            } else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P710",
                        "parse",
                        "an enum variant is a single name",
                        span(line),
                    )
                    .expected("variant_name", &line.text),
                );
                None
            }
        })
        .collect();
    declarations.push(Declaration::Enum(Enum {
        name: name.to_string(),
        variants,
        span: span(header),
    }));
}

fn source_lines(source: &str, diagnostics: &mut Vec<Diagnostic>) -> Vec<SourceLine> {
    let mut lines = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let number = index + 1;
        let without_comment = strip_comment(raw);
        if without_comment.trim().is_empty() {
            continue;
        }
        if without_comment.contains('\t') {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P005",
                    "parse",
                    "tabs are not canonical AXL indentation",
                    SourceSpan::line(number, raw),
                )
                .repair(
                    FixSafety::Safe,
                    Repair {
                        kind: "replace".into(),
                        target: format!("line {number}"),
                        replacement: Some(without_comment.replace('\t', "  ")),
                        candidates: Vec::new(),
                    },
                ),
            );
        }
        let indent = without_comment
            .chars()
            .take_while(|character| character.is_whitespace())
            .count();
        lines.push(SourceLine {
            number,
            indent,
            text: without_comment.trim().to_string(),
            raw: raw.to_string(),
        });
    }
    lines
}

fn strip_comment(source: &str) -> String {
    let mut quoted = false;
    let mut escaped = false;
    let chars: Vec<char> = source.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        if escaped {
            escaped = false;
        } else if quoted && character == '\\' {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if !quoted && character == '/' && chars.get(index + 1).copied() == Some('/') {
            return chars[..index].iter().collect();
        }
        index += 1;
    }
    source.to_string()
}

fn block_end(lines: &[SourceLine], start: usize) -> usize {
    let mut end = start + 1;
    while end < lines.len() && lines[end].indent > lines[start].indent {
        end += 1;
    }
    end
}

fn parse_entity(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["entity ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "entity"));
        return;
    }
    let mut fields = Vec::new();
    for line in body {
        let Some((field_name, remainder)) = line.text.split_once(':') else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P110",
                    "parse",
                    "an entity field uses 'name: type qualifiers'",
                    span(line),
                )
                .expected("name: type", &line.text),
            );
            continue;
        };
        let mut parts = remainder.split_whitespace();
        let Some(type_name) = parts.next() else {
            diagnostics.push(Diagnostic::error(
                "AXL-P111",
                "parse",
                "an entity field requires a type",
                span(line),
            ));
            continue;
        };
        fields.push(EntityField {
            name: field_name.trim().to_string(),
            type_name: type_name.to_string(),
            qualifiers: parts.map(str::to_string).collect(),
            span: span(line),
        });
    }
    declarations.push(Declaration::Entity(Entity {
        name: name.to_string(),
        fields,
        span: span(header),
    }));
}

fn parse_capacity(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["capacity ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "capacity"));
        return;
    }
    let mut operations = Vec::new();
    for line in body {
        let Some(value) = line.text.strip_prefix("op ") else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P210",
                    "parse",
                    "a capacity contains operation declarations",
                    span(line),
                )
                .expected("op name input -> output", &line.text),
            );
            continue;
        };
        let Some((left, output)) = value.split_once("->") else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P211",
                    "parse",
                    "an operation requires an output type",
                    span(line),
                )
                .expected("op name input -> output", &line.text),
            );
            continue;
        };
        let mut left = left.split_whitespace();
        let (Some(operation_name), Some(input)) = (left.next(), left.next()) else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P212",
                    "parse",
                    "an operation requires a name and one input type",
                    span(line),
                )
                .expected("op name input -> output", &line.text),
            );
            continue;
        };
        let mut output = output.split_whitespace();
        let output_type = output.next().unwrap_or_default();
        let qualifiers = output.collect::<Vec<_>>();
        for qualifier in &qualifiers {
            if *qualifier != "idempotent" {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P213",
                        "parse",
                        format!("unknown operation qualifier '{qualifier}'"),
                        span(line),
                    )
                    .expected("idempotent", *qualifier),
                );
            }
        }
        operations.push(Operation {
            name: operation_name.to_string(),
            input: input.to_string(),
            output: output_type.to_string(),
            idempotent: qualifiers.contains(&"idempotent"),
            span: span(line),
        });
    }
    declarations.push(Declaration::Capacity(Capacity {
        name: name.to_string(),
        operations,
        span: span(header),
    }));
}

fn parse_skill(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let value = header.text["skill ".len()..].trim();
    let Some((name, provides)) = value.split_once(" provides ") else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P310",
                "parse",
                "a skill must declare the capacity it provides",
                span(header),
            )
            .expected("skill Name provides Capacity", &header.text),
        );
        return;
    };
    let mut skill = Skill {
        name: name.trim().to_string(),
        provides: provides.trim().to_string(),
        native: None,
        configs: Vec::new(),
        effects: Vec::new(),
        capabilities: Vec::new(),
        span: span(header),
    };
    for line in body {
        if let Some(value) = line.text.strip_prefix("native ") {
            let mut parts = value.split_whitespace();
            let (Some(target), Some(symbol)) = (parts.next(), parts.next()) else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P311",
                        "parse",
                        "a native binding needs a target and symbol",
                        span(line),
                    )
                    .expected("native rust crate::symbol", &line.text),
                );
                continue;
            };
            skill.native = Some(NativeBinding {
                target: target.to_string(),
                symbol: symbol.to_string(),
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("config ") {
            let Some((declaration, config_value)) = value.split_once(" = ") else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P313",
                        "parse",
                        "a skill config needs a typed name and value",
                        span(line),
                    )
                    .expected("config name: type = value", &line.text),
                );
                continue;
            };
            let Some((name, type_name)) = declaration.split_once(':') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P314",
                        "parse",
                        "a skill config needs an explicit type",
                        span(line),
                    )
                    .expected("config name: type = value", &line.text),
                );
                continue;
            };
            skill.configs.push(SkillConfig {
                name: name.trim().to_string(),
                type_name: type_name.trim().to_string(),
                value: config_value.trim().to_string(),
                span: span(line),
            });
        } else if let Some(effect) = line.text.strip_prefix("effect ") {
            skill.effects.push(effect.trim().to_string());
        } else if let Some(capability) = line.text.strip_prefix("capability ") {
            skill.capabilities.push(capability.trim().to_string());
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P312",
                "parse",
                format!("unknown skill declaration '{}'", line.text),
                span(line),
            ));
        }
    }
    declarations.push(Declaration::Skill(skill));
}

fn parse_blueprint(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["blueprint ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "blueprint"));
        return;
    }
    let mut blueprint = Blueprint {
        name: name.to_string(),
        ports: Vec::new(),
        bindings: Vec::new(),
        contracts: Vec::new(),
        effects: Vec::new(),
        capabilities: Vec::new(),
        span: span(header),
    };
    for line in body {
        if let Some(value) = line.text.strip_prefix("in ") {
            parse_port(value, PortKind::Input, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("out ") {
            parse_port(value, PortKind::Output, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("slot ") {
            parse_port(value, PortKind::Slot, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("hook ") {
            parse_port(value, PortKind::Hook, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("param ") {
            parse_port(
                value,
                PortKind::Parameter,
                line,
                &mut blueprint,
                diagnostics,
            );
        } else if let Some(value) = line.text.strip_prefix("state ") {
            parse_port(value, PortKind::State, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("event ") {
            parse_port(value, PortKind::Event, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("action ") {
            parse_port(value, PortKind::Action, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("error ") {
            parse_port(value, PortKind::Error, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("policy ") {
            parse_port(value, PortKind::Policy, line, &mut blueprint, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("use ") {
            let Some((port, provider)) = value.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P411",
                        "parse",
                        "a binding connects a port to a provider",
                        span(line),
                    )
                    .expected("use port = Provider", &line.text),
                );
                continue;
            };
            blueprint.bindings.push(Binding {
                port: port.trim().to_string(),
                provider: provider.trim().to_string(),
                span: span(line),
            });
        } else if let Some(expression) = line.text.strip_prefix("requires ") {
            blueprint
                .contracts
                .push(contract(ContractKind::Requires, expression, line));
        } else if let Some(expression) = line.text.strip_prefix("ensures ") {
            blueprint
                .contracts
                .push(contract(ContractKind::Ensures, expression, line));
        } else if let Some(expression) = line.text.strip_prefix("invariant ") {
            blueprint
                .contracts
                .push(contract(ContractKind::Invariant, expression, line));
        } else if let Some(effect) = line.text.strip_prefix("effect ") {
            blueprint.effects.push(effect.trim().to_string());
        } else if let Some(capability) = line.text.strip_prefix("capability ") {
            blueprint.capabilities.push(capability.trim().to_string());
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P410",
                "parse",
                format!("unknown blueprint declaration '{}'", line.text),
                span(line),
            ));
        }
    }
    declarations.push(Declaration::Blueprint(blueprint));
}

fn parse_port(
    value: &str,
    kind: PortKind,
    line: &SourceLine,
    blueprint: &mut Blueprint,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some((name, type_and_default)) = value.split_once(':') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P412",
                "parse",
                "a blueprint port requires a name and type",
                span(line),
            )
            .expected("name: Type", value),
        );
        return;
    };
    let (type_name, default) = match type_and_default.split_once('=') {
        Some((type_name, default)) => (
            type_name.trim().to_string(),
            Some(default.trim().to_string()),
        ),
        None => (type_and_default.trim().to_string(), None),
    };
    blueprint.ports.push(Port {
        kind,
        name: name.trim().to_string(),
        type_name,
        default,
        span: span(line),
    });
}

fn contract(kind: ContractKind, expression: &str, line: &SourceLine) -> Contract {
    Contract {
        kind,
        expression: expression.trim().to_string(),
        span: span(line),
    }
}

fn parse_instance(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let value = header.text["instance ".len()..].trim();
    let Some((name, blueprint)) = value.split_once(" of ") else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P610",
                "parse",
                "an instance must name its blueprint",
                span(header),
            )
            .expected("instance Name of Blueprint", &header.text),
        );
        return;
    };
    let mut instance = Instance {
        name: name.trim().to_string(),
        blueprint: blueprint.trim().to_string(),
        settings: Vec::new(),
        bindings: Vec::new(),
        span: span(header),
    };
    for line in body {
        if let Some(value) = line.text.strip_prefix("set ") {
            let Some((parameter, value)) = value.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P611",
                        "parse",
                        "a setting assigns a value to a parameter",
                        span(line),
                    )
                    .expected("set parameter = value", &line.text),
                );
                continue;
            };
            instance.settings.push(Setting {
                parameter: parameter.trim().to_string(),
                value: value.trim().to_string(),
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("use ") {
            let Some((port, provider)) = value.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P612",
                        "parse",
                        "an instance override connects a surface to a provider",
                        span(line),
                    )
                    .expected("use surface = Provider", &line.text),
                );
                continue;
            };
            instance.bindings.push(Binding {
                port: port.trim().to_string(),
                provider: provider.trim().to_string(),
                span: span(line),
            });
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P613",
                "parse",
                format!("unknown instance declaration '{}'", line.text),
                span(line),
            ));
        }
    }
    declarations.push(Declaration::Instance(instance));
}

fn parse_flow(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let value = header.text["flow ".len()..].trim();
    let Some((left, output)) = value.split_once("->") else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P810",
                "parse",
                "a flow requires an input and output type",
                span(header),
            )
            .expected("flow Name Input -> Output", &header.text),
        );
        return;
    };
    let mut left = left.split_whitespace();
    let (Some(name), Some(input), None) = (left.next(), left.next(), left.next()) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P811",
                "parse",
                "a flow header requires a name and one input type",
                span(header),
            )
            .expected("flow Name Input -> Output", &header.text),
        );
        return;
    };
    let mut dependencies = Vec::new();
    let mut bindings = Vec::new();
    let mut statements = Vec::new();
    let statement_indent = body.iter().map(|line| line.indent).min().unwrap_or(0);
    for (line_index, line) in body.iter().enumerate() {
        if line.indent > statement_indent {
            continue;
        }
        if line.text == "]" {
            continue;
        }
        if let Some(value) = line.text.strip_prefix("in ") {
            let Some((name, capacity_and_default)) = value.split_once(':') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P816",
                        "parse",
                        "a flow dependency requires a name and capacity",
                        span(line),
                    )
                    .expected("in name: Capacity = Provider", &line.text),
                );
                continue;
            };
            let (capacity, default) = match capacity_and_default.split_once('=') {
                Some((capacity, provider)) => (
                    capacity.trim().to_string(),
                    Some(provider.trim().to_string()),
                ),
                None => (capacity_and_default.trim().to_string(), None),
            };
            dependencies.push(FlowDependency {
                name: name.trim().to_string(),
                capacity,
                default,
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("use ") {
            let Some((dependency, provider)) = value.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P817",
                        "parse",
                        "a flow binding connects a dependency to a provider",
                        span(line),
                    )
                    .expected("use dependency = Provider", &line.text),
                );
                continue;
            };
            bindings.push(Binding {
                port: dependency.trim().to_string(),
                provider: provider.trim().to_string(),
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("let ") {
            let Some((name, expression)) = value.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P812",
                        "parse",
                        "a let statement binds an expression",
                        span(line),
                    )
                    .expected("let name = expression", &line.text),
                );
                continue;
            };
            let mut expression = expression.trim().to_string();
            if expression.starts_with('[') && !expression.ends_with(']') {
                for continuation in body.iter().skip(line_index + 1) {
                    expression.push(' ');
                    expression.push_str(continuation.text.trim());
                    if continuation.text == "]" {
                        break;
                    }
                }
            } else {
                for continuation in body
                    .iter()
                    .skip(line_index + 1)
                    .take_while(|candidate| candidate.indent > line.indent)
                {
                    expression.push(' ');
                    expression.push_str(continuation.text.trim());
                }
            }
            statements.push(FlowStatement::Let {
                name: name.trim().to_string(),
                expression,
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("require ") {
            let Some((expression, message)) = value.rsplit_once(" else ") else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P813",
                        "parse",
                        "a require statement needs an error message",
                        span(line),
                    )
                    .expected("require expression else \"message\"", &line.text),
                );
                continue;
            };
            match serde_json::from_str::<String>(message.trim()) {
                Ok(message) => statements.push(FlowStatement::Require {
                    expression: expression.trim().to_string(),
                    message,
                    span: span(line),
                }),
                Err(_) => diagnostics.push(
                    Diagnostic::error(
                        "AXL-P814",
                        "parse",
                        "a require error message must be a JSON string",
                        span(line),
                    )
                    .expected("\"message\"", message.trim()),
                ),
            }
        } else if let Some(value) = line.text.strip_prefix("call ") {
            parse_flow_call(value, line, &mut statements, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("attempt ") {
            parse_attempt(value, line, body, line_index, &mut statements, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("run ") {
            parse_flow_run(value, line, &mut statements, diagnostics);
        } else if let Some(value) = line.text.strip_prefix("make ") {
            let Some((name, type_name)) = value.split_once(':') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P823",
                        "parse",
                        "a record constructor requires a variable and entity type",
                        span(line),
                    )
                    .expected("make name: Entity", &line.text),
                );
                continue;
            };
            let mut fields = Vec::new();
            for field_line in body
                .iter()
                .skip(line_index + 1)
                .take_while(|candidate| candidate.indent > line.indent)
            {
                let Some((field, expression)) = field_line.text.split_once('=') else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-P824",
                            "parse",
                            "a constructed field binds an expression",
                            span(field_line),
                        )
                        .expected("field = expression", &field_line.text),
                    );
                    continue;
                };
                fields.push(RecordFieldValue {
                    name: field.trim().to_string(),
                    expression: expression.trim().to_string(),
                    span: span(field_line),
                });
            }
            statements.push(FlowStatement::Make {
                name: name.trim().to_string(),
                type_name: type_name.trim().to_string(),
                fields,
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("fold ") {
            let parsed = value
                .split_once('=')
                .and_then(|(name_and_type, remainder)| {
                    let (name, type_name) = name_and_type.split_once(':')?;
                    let (collection, initial_and_item) = remainder.split_once(" from ")?;
                    let (initial, item) = initial_and_item.rsplit_once(" as ")?;
                    Some((name, type_name, collection, initial, item))
                });
            let Some((name, type_name, collection, initial, item)) = parsed else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P825",
                        "parse",
                        "a fold requires a result type, collection, initial value and item",
                        span(line),
                    )
                    .expected(
                        "fold name: Type = collection from initial as item",
                        &line.text,
                    ),
                );
                continue;
            };
            let nested = body
                .iter()
                .skip(line_index + 1)
                .take_while(|candidate| candidate.indent > line.indent)
                .collect::<Vec<_>>();
            let Some(next_line) = nested.first() else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P826",
                        "parse",
                        "a fold requires a next expression",
                        span(line),
                    )
                    .expected("next = expression", "missing"),
                );
                continue;
            };
            let Some((keyword, expression)) = next_line.text.split_once('=') else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P826",
                        "parse",
                        "a fold requires a next expression",
                        span(next_line),
                    )
                    .expected("next = expression", &next_line.text),
                );
                continue;
            };
            if keyword.trim() != "next" {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P826",
                        "parse",
                        "a fold body begins with 'next ='",
                        span(next_line),
                    )
                    .expected("next = expression", &next_line.text),
                );
                continue;
            }
            let mut update = expression.trim().to_string();
            for continuation in nested.iter().skip(1) {
                update.push(' ');
                update.push_str(continuation.text.trim());
            }
            statements.push(FlowStatement::Fold {
                name: name.trim().to_string(),
                type_name: type_name.trim().to_string(),
                collection: collection.trim().to_string(),
                initial: initial.trim().to_string(),
                item: item.trim().to_string(),
                update,
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("match ") {
            let parsed = value.split_once('=').and_then(|(name_and_type, subject)| {
                let (name, type_name) = name_and_type.split_once(':')?;
                Some((name.trim(), type_name.trim(), subject.trim()))
            });
            let Some((name, type_name, subject)) = parsed else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P828",
                        "parse",
                        "a match requires a result type and subject",
                        span(line),
                    )
                    .expected("match name: Type = expression", &line.text),
                );
                continue;
            };
            let mut cases = Vec::new();
            for case_line in body
                .iter()
                .skip(line_index + 1)
                .take_while(|candidate| candidate.indent > line.indent)
            {
                let Some((variant, expression)) = case_line.text.split_once("=>") else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-P829",
                            "parse",
                            "a match case maps a variant to an expression",
                            span(case_line),
                        )
                        .expected("variant => expression", &case_line.text),
                    );
                    continue;
                };
                cases.push(MatchCase {
                    variant: variant.trim().to_string(),
                    expression: expression.trim().to_string(),
                    span: span(case_line),
                });
            }
            statements.push(FlowStatement::Match {
                name: name.into(),
                type_name: type_name.into(),
                subject: subject.into(),
                cases,
                span: span(line),
            });
        } else if let Some(value) = line.text.strip_prefix("map ") {
            if let Some((name, type_name, collection, item, expression)) =
                parse_transform(value, "value", line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Map {
                    name,
                    type_name,
                    collection,
                    item,
                    expression,
                    span: span(line),
                });
            }
        } else if let Some(value) = line.text.strip_prefix("filter ") {
            if let Some((name, type_name, collection, item, predicate)) =
                parse_transform(value, "where", line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Filter {
                    name,
                    type_name,
                    collection,
                    item,
                    predicate,
                    span: span(line),
                });
            }
        } else if let Some(value) = line.text.strip_prefix("sort ") {
            if let Some((name, type_name, collection, item, key, direction)) =
                parse_sort(value, line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Sort {
                    name,
                    type_name,
                    collection,
                    item,
                    key,
                    direction,
                    span: span(line),
                });
            }
        } else if let Some(value) = line.text.strip_prefix("group ") {
            if let Some((name, type_name, collection, item, key)) =
                parse_transform(value, "by", line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Group {
                    name,
                    type_name,
                    collection,
                    item,
                    key,
                    span: span(line),
                });
            }
        } else if let Some(value) = line.text.strip_prefix("parallel ") {
            if let Some((name, type_name, collection, item, flow, argument, propagate)) =
                parse_parallel(value, line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Parallel {
                    name,
                    type_name,
                    collection,
                    item,
                    flow,
                    argument,
                    propagate,
                    span: span(line),
                });
            }
        } else if let Some(value) = line.text.strip_prefix("race ") {
            if let Some((name, type_name, collection, item, flow, argument, propagate)) =
                parse_race(value, line, body, line_index, diagnostics)
            {
                statements.push(FlowStatement::Race {
                    name,
                    type_name,
                    collection,
                    item,
                    flow,
                    argument,
                    propagate,
                    span: span(line),
                });
            }
        } else if let Some(expression) = line.text.strip_prefix("return ") {
            statements.push(FlowStatement::Return {
                expression: expression.trim().to_string(),
                span: span(line),
            });
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P815",
                "parse",
                format!("unknown flow statement '{}'", line.text),
                span(line),
            ));
        }
    }
    declarations.push(Declaration::Flow(Flow {
        name: name.to_string(),
        input: input.to_string(),
        output: output.trim().to_string(),
        dependencies,
        bindings,
        statements,
        span: span(header),
    }));
}

fn parse_flow_call(
    value: &str,
    line: &SourceLine,
    statements: &mut Vec<FlowStatement>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some((name, invocation)) = value.split_once('=') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P818",
                "parse",
                "a call binds a provider operation result",
                span(line),
            )
            .expected("call name = dependency.operation(argument)?", &line.text),
        );
        return;
    };
    let invocation = invocation.trim();
    let (invocation, propagate) = match invocation.strip_suffix('?') {
        Some(value) => (value.trim(), true),
        None => (invocation, false),
    };
    let Some((target, argument)) = invocation.split_once('(') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P819",
                "parse",
                "a call requires a target and parenthesized argument",
                span(line),
            )
            .expected("dependency.operation(argument)", invocation),
        );
        return;
    };
    let Some(argument) = argument.strip_suffix(')') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P819",
                "parse",
                "a call requires a closing ')'",
                span(line),
            )
            .expected("dependency.operation(argument)", invocation),
        );
        return;
    };
    let Some((dependency, operation)) = target.trim().split_once('.') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P819",
                "parse",
                "a call target names a dependency and operation",
                span(line),
            )
            .expected("dependency.operation(argument)", invocation),
        );
        return;
    };
    if name.trim().is_empty()
        || dependency.trim().is_empty()
        || operation.trim().is_empty()
        || argument.trim().is_empty()
    {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P819",
                "parse",
                "a call requires a result, dependency, operation and argument",
                span(line),
            )
            .expected("call name = dependency.operation(argument)?", &line.text),
        );
        return;
    }
    statements.push(FlowStatement::Call {
        name: name.trim().to_string(),
        dependency: dependency.trim().to_string(),
        operation: operation.trim().to_string(),
        argument: argument.trim().to_string(),
        propagate,
        span: span(line),
    });
}

fn parse_flow_run(
    value: &str,
    line: &SourceLine,
    statements: &mut Vec<FlowStatement>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let parsed = value.split_once('=').and_then(|(name, invocation)| {
        let invocation = invocation.trim();
        let (invocation, propagate) = match invocation.strip_suffix('?') {
            Some(value) => (value.trim(), true),
            None => (invocation, false),
        };
        let (flow, argument) = invocation.split_once('(')?;
        let argument = argument.strip_suffix(')')?;
        Some((name.trim(), flow.trim(), argument.trim(), propagate))
    });
    let Some((name, flow, argument, propagate)) = parsed else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P827",
                "parse",
                "a run binds another flow result",
                span(line),
            )
            .expected("run name = Flow(argument)?", &line.text),
        );
        return;
    };
    if name.is_empty() || flow.is_empty() || argument.is_empty() {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P827",
                "parse",
                "a run requires a result, flow and argument",
                span(line),
            )
            .expected("run name = Flow(argument)?", &line.text),
        );
        return;
    }
    statements.push(FlowStatement::Run {
        name: name.into(),
        flow: flow.into(),
        argument: argument.into(),
        propagate,
        span: span(line),
    });
}

fn parse_attempt(
    value: &str,
    line: &SourceLine,
    body: &[SourceLine],
    line_index: usize,
    statements: &mut Vec<FlowStatement>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let invocation = value.split_once('=').and_then(|(name, invocation)| {
        let invocation = invocation.trim();
        let (invocation, propagate) = invocation
            .strip_suffix('?')
            .map_or((invocation, false), |value| (value.trim(), true));
        let (target, argument) = invocation.split_once('(')?;
        let argument = argument.strip_suffix(')')?;
        let (dependency, operation) = target.trim().split_once('.')?;
        Some((
            name.trim(),
            dependency.trim(),
            operation.trim(),
            argument.trim(),
            propagate,
        ))
    });
    let Some((name, dependency, operation, argument, propagate)) = invocation else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P836",
                "parse",
                "attempt requires a provider operation invocation",
                span(line),
            )
            .expected("attempt name = dependency.operation(argument)?", &line.text),
        );
        return;
    };
    let nested = body
        .iter()
        .skip(line_index + 1)
        .take_while(|candidate| candidate.indent > line.indent)
        .collect::<Vec<_>>();
    let value = |keyword: &str| {
        nested.iter().find_map(|nested_line| {
            let (found, value) = nested_line.text.split_once('=')?;
            (found.trim() == keyword).then_some(value.trim())
        })
    };
    let retry = value("retry").and_then(|value| value.parse::<u32>().ok());
    let timeout_ms = value("timeout_ms").and_then(|value| value.parse::<u64>().ok());
    let (Some(retry), Some(timeout_ms)) = (retry, timeout_ms) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P837",
                "parse",
                "attempt requires numeric retry and timeout_ms clauses",
                span(line),
            )
            .expected(
                "retry = count\n  timeout_ms = milliseconds",
                "missing or invalid",
            ),
        );
        return;
    };
    statements.push(FlowStatement::Attempt {
        name: name.into(),
        dependency: dependency.into(),
        operation: operation.into(),
        argument: argument.into(),
        propagate,
        retry,
        timeout_ms,
        span: span(line),
    });
}

fn parse_transform(
    value: &str,
    body_keyword: &str,
    line: &SourceLine,
    body: &[SourceLine],
    line_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, String, String, String, String)> {
    let parsed = value.split_once('=').and_then(|(name_and_type, source)| {
        let (name, type_name) = name_and_type.split_once(':')?;
        let (collection, item) = source.rsplit_once(" as ")?;
        Some((
            name.trim(),
            type_name.trim(),
            collection.trim(),
            item.trim(),
        ))
    });
    let Some((name, type_name, collection, item)) = parsed else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P830",
                "parse",
                "a collection transform requires a result type, source and item",
                span(line),
            )
            .expected(
                format!("name: List<T> = collection as item\n  {body_keyword} = expression"),
                &line.text,
            ),
        );
        return None;
    };
    let nested = body
        .iter()
        .skip(line_index + 1)
        .take_while(|candidate| candidate.indent > line.indent)
        .collect::<Vec<_>>();
    let Some(expression_line) = nested.first() else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P831",
                "parse",
                "a collection transform requires a body expression",
                span(line),
            )
            .expected(format!("{body_keyword} = expression"), "missing"),
        );
        return None;
    };
    let Some((keyword, expression)) = expression_line.text.split_once('=') else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P831",
                "parse",
                "a collection transform body binds an expression",
                span(expression_line),
            )
            .expected(
                format!("{body_keyword} = expression"),
                &expression_line.text,
            ),
        );
        return None;
    };
    if keyword.trim() != body_keyword {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P831",
                "parse",
                format!("a collection transform body begins with '{body_keyword} ='"),
                span(expression_line),
            )
            .expected(
                format!("{body_keyword} = expression"),
                &expression_line.text,
            ),
        );
        return None;
    }
    let mut expression = expression.trim().to_string();
    for continuation in nested.iter().skip(1) {
        expression.push(' ');
        expression.push_str(continuation.text.trim());
    }
    Some((
        name.into(),
        type_name.into(),
        collection.into(),
        item.into(),
        expression,
    ))
}

fn parse_sort(
    value: &str,
    line: &SourceLine,
    body: &[SourceLine],
    line_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, String, String, String, String, String)> {
    let parsed = value.split_once('=').and_then(|(name_and_type, source)| {
        let (name, type_name) = name_and_type.split_once(':')?;
        let (collection, item) = source.rsplit_once(" as ")?;
        Some((
            name.trim(),
            type_name.trim(),
            collection.trim(),
            item.trim(),
        ))
    });
    let Some((name, type_name, collection, item)) = parsed else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P832",
                "parse",
                "a sort requires a result type, source and item",
                span(line),
            )
            .expected("sort name: List<T> = collection as item", &line.text),
        );
        return None;
    };
    let nested = body
        .iter()
        .skip(line_index + 1)
        .take_while(|candidate| candidate.indent > line.indent)
        .collect::<Vec<_>>();
    let key = nested.iter().find_map(|nested_line| {
        nested_line
            .text
            .split_once('=')
            .filter(|(keyword, _)| keyword.trim() == "by")
            .map(|(_, value)| value.trim().to_string())
    });
    let direction = nested.iter().find_map(|nested_line| {
        nested_line
            .text
            .split_once('=')
            .filter(|(keyword, _)| keyword.trim() == "direction")
            .map(|(_, value)| value.trim().to_string())
    });
    let (Some(key), Some(direction)) = (key, direction) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P833",
                "parse",
                "a sort requires 'by' and 'direction' clauses",
                span(line),
            )
            .expected("by = expression\n  direction = asc|desc", "missing clause"),
        );
        return None;
    };
    Some((
        name.into(),
        type_name.into(),
        collection.into(),
        item.into(),
        key,
        direction,
    ))
}

#[allow(clippy::type_complexity)]
fn parse_parallel(
    value: &str,
    line: &SourceLine,
    body: &[SourceLine],
    line_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, String, String, String, String, String, bool)> {
    let header = value.split_once('=').and_then(|(name_and_type, source)| {
        let (name, type_name) = name_and_type.split_once(':')?;
        let (collection, item) = source.rsplit_once(" as ")?;
        Some((
            name.trim(),
            type_name.trim(),
            collection.trim(),
            item.trim(),
        ))
    });
    let Some((name, type_name, collection, item)) = header else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P834",
                "parse",
                "parallel requires a result type, collection and item",
                span(line),
            )
            .expected("parallel name: List<T> = collection as item", &line.text),
        );
        return None;
    };
    let run_line = body
        .iter()
        .skip(line_index + 1)
        .take_while(|candidate| candidate.indent > line.indent)
        .next();
    let invocation = run_line.and_then(|run_line| {
        let (keyword, invocation) = run_line.text.split_once('=')?;
        (keyword.trim() == "run").then_some(invocation.trim())
    });
    let parsed = invocation.and_then(|invocation| {
        let (invocation, propagate) = invocation
            .strip_suffix('?')
            .map_or((invocation, false), |value| (value.trim(), true));
        let (flow, argument) = invocation.split_once('(')?;
        let argument = argument.strip_suffix(')')?;
        Some((flow.trim(), argument.trim(), propagate))
    });
    let Some((flow, argument, propagate)) = parsed else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P835",
                "parse",
                "parallel requires a flow invocation",
                span(line),
            )
            .expected(
                "run = Flow(item)?",
                run_line.map_or("missing", |line| &line.text),
            ),
        );
        return None;
    };
    Some((
        name.into(),
        type_name.into(),
        collection.into(),
        item.into(),
        flow.into(),
        argument.into(),
        propagate,
    ))
}

#[allow(clippy::type_complexity)]
fn parse_race(
    value: &str,
    line: &SourceLine,
    body: &[SourceLine],
    line_index: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, String, String, String, String, String, bool)> {
    let header = value.split_once('=').and_then(|(name_and_type, source)| {
        let (name, type_name) = name_and_type.split_once(':')?;
        let (collection, item) = source.rsplit_once(" as ")?;
        Some((
            name.trim(),
            type_name.trim(),
            collection.trim(),
            item.trim(),
        ))
    });
    let Some((name, type_name, collection, item)) = header else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P838",
                "parse",
                "race requires a result type, collection and item",
                span(line),
            )
            .expected("race name: T = collection as item", &line.text),
        );
        return None;
    };
    let run_line = body
        .iter()
        .skip(line_index + 1)
        .take_while(|candidate| candidate.indent > line.indent)
        .next();
    let invocation = run_line.and_then(|run_line| {
        let (keyword, invocation) = run_line.text.split_once('=')?;
        (keyword.trim() == "run").then_some(invocation.trim())
    });
    let parsed = invocation.and_then(|invocation| {
        let (invocation, propagate) = invocation
            .strip_suffix('?')
            .map_or((invocation, false), |value| (value.trim(), true));
        let (flow, argument) = invocation.split_once('(')?;
        let argument = argument.strip_suffix(')')?;
        Some((flow.trim(), argument.trim(), propagate))
    });
    let Some((flow, argument, propagate)) = parsed else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P839",
                "parse",
                "race requires a flow invocation",
                span(line),
            )
            .expected(
                "run = Flow(item)?",
                run_line.map_or("missing", |line| &line.text),
            ),
        );
        return None;
    };
    Some((
        name.into(),
        type_name.into(),
        collection.into(),
        item.into(),
        flow.into(),
        argument.into(),
        propagate,
    ))
}

fn parse_agent(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["agent ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "agent"));
        return;
    }
    let mut agent = Agent {
        name: name.to_string(),
        beliefs: Vec::new(),
        goals: Vec::new(),
        plans: Vec::new(),
        effects: Vec::new(),
        capabilities: Vec::new(),
        span: span(header),
    };
    for line in body {
        if let Some(value) = line.text.strip_prefix("believe ") {
            agent.beliefs.push(value.trim().to_string());
        } else if let Some(value) = line.text.strip_prefix("goal ") {
            agent.goals.push(value.trim().to_string());
        } else if let Some(value) = line.text.strip_prefix("plan ") {
            agent.plans.push(value.trim().to_string());
        } else if let Some(value) = line.text.strip_prefix("effect ") {
            agent.effects.push(value.trim().to_string());
        } else if let Some(value) = line.text.strip_prefix("capability ") {
            agent.capabilities.push(value.trim().to_string());
        } else {
            diagnostics.push(Diagnostic::error(
                "AXL-P510",
                "parse",
                format!("unknown agent declaration '{}'", line.text),
                span(line),
            ));
        }
    }
    declarations.push(Declaration::Agent(agent));
}

fn parse_api(
    header: &SourceLine,
    body: &[SourceLine],
    declarations: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = header.text["api ".len()..].trim();
    if name.is_empty() {
        diagnostics.push(missing_name(header, "api"));
        return;
    }
    let mut routes = Vec::new();
    let mut auth = None;
    let mut cursor = 0;
    while cursor < body.len() {
        let line = &body[cursor];
        if let Some(value) = line.text.strip_prefix("auth ") {
            let parsed = value.split_once('=').and_then(|(surface, provider)| {
                let (scheme, capacity) = surface.split_once(':')?;
                Some((scheme.trim(), capacity.trim(), provider.trim()))
            });
            let Some((scheme, capacity, provider)) = parsed else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P913",
                        "parse",
                        "API auth requires a scheme, capacity and provider",
                        span(line),
                    )
                    .expected("auth bearer: HttpAuth = AuthProvider", &line.text),
                );
                cursor += 1;
                continue;
            };
            if auth.is_some() {
                diagnostics.push(Diagnostic::error(
                    "AXL-P914",
                    "parse",
                    "an API can declare auth only once",
                    span(line),
                ));
                cursor += 1;
                continue;
            }
            auth = Some(ApiAuth {
                scheme: scheme.into(),
                capacity: capacity.into(),
                provider: provider.into(),
                span: span(line),
            });
            cursor += 1;
            continue;
        }
        if line.text.starts_with("bind ") {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P916",
                    "parse",
                    "a request binding must be nested under a route",
                    span(line),
                )
                .expected("route\n  bind field = source", &line.text),
            );
            cursor += 1;
            continue;
        }
        let Some((method, remainder)) = line.text.split_once(' ') else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P910",
                    "parse",
                    "an API route requires method, path, signature and flow",
                    span(line),
                )
                .expected("post /path Input -> Output = Flow", &line.text),
            );
            cursor += 1;
            continue;
        };
        let Some((signature, flow)) = remainder.rsplit_once('=') else {
            diagnostics.push(
                Diagnostic::error("AXL-P910", "parse", "an API route binds a flow", span(line))
                    .expected("post /path Input -> Output = Flow", &line.text),
            );
            cursor += 1;
            continue;
        };
        let Some((request, output)) = signature.split_once("->") else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P911",
                    "parse",
                    "an API route requires an output type",
                    span(line),
                )
                .expected("post /path Input -> Output = Flow", &line.text),
            );
            cursor += 1;
            continue;
        };
        let mut request = request.split_whitespace();
        let (Some(path), Some(input), None) = (request.next(), request.next(), request.next())
        else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P912",
                    "parse",
                    "an API route requires one path and input type",
                    span(line),
                )
                .expected("post /path Input -> Output = Flow", &line.text),
            );
            cursor += 1;
            continue;
        };
        let flow = flow.trim();
        let (flow, input_source, input_name, mut bindings) =
            if let Some((flow, binding)) = flow.split_once(" from ") {
                let Some((source, name)) = parse_request_source(binding) else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-P915",
                            "parse",
                            "a request binding needs a source and name",
                            span(line),
                        )
                        .expected("Flow from path.id|query.name", binding),
                    );
                    cursor += 1;
                    continue;
                };
                (
                    flow.trim(),
                    source.clone(),
                    name.clone(),
                    vec![HttpRequestBinding {
                        target: None,
                        source,
                        name,
                        span: span(line),
                    }],
                )
            } else {
                (
                    flow,
                    "body".to_string(),
                    None,
                    vec![HttpRequestBinding {
                        target: None,
                        source: "body".into(),
                        name: None,
                        span: span(line),
                    }],
                )
            };
        cursor += 1;
        let mut found_nested = false;
        while cursor < body.len() && body[cursor].indent > line.indent {
            let binding_line = &body[cursor];
            let Some(value) = binding_line.text.strip_prefix("bind ") else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P916",
                        "parse",
                        "unknown nested API route declaration",
                        span(binding_line),
                    )
                    .expected(
                        "bind field = body|body.field|path.name|query.name",
                        &binding_line.text,
                    ),
                );
                cursor += 1;
                continue;
            };
            let parsed = value.split_once('=').and_then(|(target, source)| {
                let target = target.trim();
                let (source, name) = parse_request_source(source.trim())?;
                (!target.is_empty()).then(|| (target.to_string(), source, name))
            });
            let Some((target, source, name)) = parsed else {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-P916",
                        "parse",
                        "invalid composite request binding",
                        span(binding_line),
                    )
                    .expected(
                        "bind field = body|body.field|path.name|query.name",
                        &binding_line.text,
                    ),
                );
                cursor += 1;
                continue;
            };
            if !found_nested {
                if input_source != "body" || input_name.is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-P917",
                        "parse",
                        "inline and nested request bindings cannot be combined",
                        span(binding_line),
                    ));
                }
                bindings.clear();
                found_nested = true;
            }
            bindings.push(HttpRequestBinding {
                target: Some(target),
                source,
                name,
                span: span(binding_line),
            });
            cursor += 1;
        }
        let (input_source, input_name) = if found_nested {
            ("composite".into(), None)
        } else {
            (input_source, input_name)
        };
        routes.push(ApiRoute {
            method: method.to_ascii_lowercase(),
            path: path.into(),
            input: input.into(),
            output: output.trim().into(),
            flow: flow.into(),
            input_source,
            input_name,
            bindings,
            span: span(line),
        });
    }
    declarations.push(Declaration::Api(Api {
        name: name.into(),
        auth,
        routes,
        span: span(header),
    }));
}

fn parse_request_source(value: &str) -> Option<(String, Option<String>)> {
    if value == "body" {
        return Some(("body".into(), None));
    }
    let (source, name) = value.split_once('.')?;
    (matches!(source, "body" | "path" | "query") && !name.is_empty())
        .then(|| (source.into(), Some(name.into())))
}

fn missing_name(line: &SourceLine, kind: &str) -> Diagnostic {
    Diagnostic::error(
        "AXL-P010",
        "parse",
        format!("{kind} name is required"),
        span(line),
    )
}

fn unexpected_body(line: &SourceLine, kind: &str) -> Diagnostic {
    Diagnostic::error(
        "AXL-P006",
        "parse",
        format!("{kind} cannot contain an indented body"),
        span(line),
    )
}

fn span(line: &SourceLine) -> SourceSpan {
    SourceSpan::line(line.number, &line.raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_semantic_blueprints() {
        let source = r#"axl 4
app Demo

entity Customer
  id: uuid key
  email: email unique

capacity CustomerStore
  op save Customer -> Result<Customer>

skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
  effect db.write

blueprint CRM
  param page_size: int = 25
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text
  in store: CustomerStore
  use store = SqliteCustomers
  invariant Customer.email unique
"#;
        let program = parse(source).unwrap();
        assert_eq!(program.name, "Demo");
        assert_eq!(program.declarations.len(), 4);
        let Declaration::Blueprint(blueprint) = &program.declarations[3] else {
            panic!("expected blueprint")
        };
        assert_eq!(blueprint.ports[0].name, "page_size");
        assert_eq!(blueprint.ports[0].kind, PortKind::Parameter);
        assert_eq!(blueprint.ports[0].default.as_deref(), Some("25"));
        assert_eq!(blueprint.ports[1].kind, PortKind::State);
        assert_eq!(blueprint.ports[2].kind, PortKind::Event);
        assert_eq!(blueprint.ports[3].kind, PortKind::Error);
        assert_eq!(blueprint.ports[4].name, "store");
        assert_eq!(blueprint.bindings[0].provider, "SqliteCustomers");
    }

    #[test]
    fn emits_repair_for_missing_version() {
        let errors = parse("app Demo\nentity Customer\n  id: uuid").unwrap_err();
        let error = errors
            .iter()
            .find(|error| error.code == "AXL-P001")
            .unwrap();
        assert_eq!(error.fix_safety, FixSafety::Safe);
        assert_eq!(error.repairs[0].replacement.as_deref(), Some("axl 4"));
    }
}
