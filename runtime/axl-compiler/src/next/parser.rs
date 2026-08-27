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
        operations.push(Operation {
            name: operation_name.to_string(),
            input: input.to_string(),
            output: output.trim().to_string(),
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
