use super::ast::*;

pub fn format(program: &Program) -> String {
    let mut output = vec![
        format!("axl {}", program.version),
        format!("app {}", program.name),
    ];
    for declaration in &program.declarations {
        output.push(String::new());
        match declaration {
            Declaration::Enum(value) => {
                output.push(format!("enum {}", value.name));
                for variant in &value.variants {
                    output.push(format!("  {}", variant.name));
                }
            }
            Declaration::Entity(entity) => {
                output.push(format!("entity {}", entity.name));
                for field in &entity.fields {
                    let qualifiers = if field.qualifiers.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", field.qualifiers.join(" "))
                    };
                    output.push(format!(
                        "  {}: {}{}",
                        field.name, field.type_name, qualifiers
                    ));
                }
            }
            Declaration::Capacity(capacity) => {
                output.push(format!("capacity {}", capacity.name));
                for operation in &capacity.operations {
                    output.push(format!(
                        "  op {} {} -> {}",
                        operation.name, operation.input, operation.output
                    ));
                }
            }
            Declaration::Skill(skill) => {
                output.push(format!("skill {} provides {}", skill.name, skill.provides));
                if let Some(native) = &skill.native {
                    output.push(format!("  native {} {}", native.target, native.symbol));
                }
                append_values(&mut output, "effect", &skill.effects);
                append_values(&mut output, "capability", &skill.capabilities);
            }
            Declaration::Blueprint(blueprint) => {
                output.push(format!("blueprint {}", blueprint.name));
                for port in &blueprint.ports {
                    let keyword = port.kind.keyword();
                    let default = port
                        .default
                        .as_ref()
                        .map(|value| format!(" = {value}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  {keyword} {}: {}{default}",
                        port.name, port.type_name
                    ));
                }
                for binding in &blueprint.bindings {
                    output.push(format!("  use {} = {}", binding.port, binding.provider));
                }
                for contract in &blueprint.contracts {
                    let keyword = match contract.kind {
                        ContractKind::Requires => "requires",
                        ContractKind::Ensures => "ensures",
                        ContractKind::Invariant => "invariant",
                    };
                    output.push(format!("  {keyword} {}", contract.expression));
                }
                append_values(&mut output, "effect", &blueprint.effects);
                append_values(&mut output, "capability", &blueprint.capabilities);
            }
            Declaration::Instance(instance) => {
                output.push(format!(
                    "instance {} of {}",
                    instance.name, instance.blueprint
                ));
                for setting in &instance.settings {
                    output.push(format!("  set {} = {}", setting.parameter, setting.value));
                }
                for binding in &instance.bindings {
                    output.push(format!("  use {} = {}", binding.port, binding.provider));
                }
            }
            Declaration::Flow(flow) => {
                output.push(format!(
                    "flow {} {} -> {}",
                    flow.name, flow.input, flow.output
                ));
                for dependency in &flow.dependencies {
                    let default = dependency
                        .default
                        .as_ref()
                        .map(|provider| format!(" = {provider}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  in {}: {}{}",
                        dependency.name, dependency.capacity, default
                    ));
                }
                for binding in &flow.bindings {
                    output.push(format!("  use {} = {}", binding.port, binding.provider));
                }
                for statement in &flow.statements {
                    match statement {
                        FlowStatement::Let {
                            name, expression, ..
                        } => output.push(format!("  let {name} = {expression}")),
                        FlowStatement::Require {
                            expression,
                            message,
                            ..
                        } => output.push(format!(
                            "  require {} else {}",
                            expression,
                            serde_json::to_string(message)
                                .expect("flow messages are JSON encodable")
                        )),
                        FlowStatement::Call {
                            name,
                            dependency,
                            operation,
                            argument,
                            propagate,
                            ..
                        } => output.push(format!(
                            "  call {name} = {dependency}.{operation}({argument}){}",
                            if *propagate { "?" } else { "" }
                        )),
                        FlowStatement::Make {
                            name,
                            type_name,
                            fields,
                            ..
                        } => {
                            output.push(format!("  make {name}: {type_name}"));
                            for field in fields {
                                output.push(format!("    {} = {}", field.name, field.expression));
                            }
                        }
                        FlowStatement::Fold {
                            name,
                            type_name,
                            collection,
                            initial,
                            item,
                            update,
                            ..
                        } => {
                            output.push(format!(
                                "  fold {name}: {type_name} = {collection} from {initial} as {item}"
                            ));
                            if let Some((condition, when_true, when_false)) =
                                conditional_parts(update)
                            {
                                output.push(format!("    next = if {condition}"));
                                output.push(format!("      then {when_true}"));
                                output.push(format!("      else {when_false}"));
                            } else {
                                output.push(format!("    next = {update}"));
                            }
                        }
                        FlowStatement::Run {
                            name,
                            flow,
                            argument,
                            propagate,
                            ..
                        } => output.push(format!(
                            "  run {name} = {flow}({argument}){}",
                            if *propagate { "?" } else { "" }
                        )),
                        FlowStatement::Match {
                            name,
                            type_name,
                            subject,
                            cases,
                            ..
                        } => {
                            output.push(format!("  match {name}: {type_name} = {subject}"));
                            for case in cases {
                                output.push(format!("    {} => {}", case.variant, case.expression));
                            }
                        }
                        FlowStatement::Map {
                            name,
                            type_name,
                            collection,
                            item,
                            expression,
                            ..
                        } => {
                            output.push(format!(
                                "  map {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!("    value = {expression}"));
                        }
                        FlowStatement::Filter {
                            name,
                            type_name,
                            collection,
                            item,
                            predicate,
                            ..
                        } => {
                            output.push(format!(
                                "  filter {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!("    where = {predicate}"));
                        }
                        FlowStatement::Return { expression, .. } => {
                            output.push(format!("  return {expression}"));
                        }
                    }
                }
            }
            Declaration::Api(api) => {
                output.push(format!("api {}", api.name));
                for route in &api.routes {
                    output.push(format!(
                        "  {} {} {} -> {} = {}",
                        route.method, route.path, route.input, route.output, route.flow
                    ));
                }
            }
            Declaration::Agent(agent) => {
                output.push(format!("agent {}", agent.name));
                append_values(&mut output, "believe", &agent.beliefs);
                append_values(&mut output, "goal", &agent.goals);
                append_values(&mut output, "plan", &agent.plans);
                append_values(&mut output, "effect", &agent.effects);
                append_values(&mut output, "capability", &agent.capabilities);
            }
        }
    }
    output.push(String::new());
    output.join("\n")
}

fn append_values(output: &mut Vec<String>, keyword: &str, values: &[String]) {
    for value in values {
        output.push(format!("  {keyword} {value}"));
    }
}

fn conditional_parts(value: &str) -> Option<(&str, &str, &str)> {
    let value = value.strip_prefix("if ")?;
    let (condition, values) = value.split_once(" then ")?;
    let (when_true, when_false) = values.rsplit_once(" else ")?;
    Some((condition, when_true, when_false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::parser;

    #[test]
    fn canonical_source_round_trips() {
        let source = "axl 4\napp Demo\nentity Customer\n    id: uuid key\n";
        let first = parser::parse(source).unwrap();
        let formatted = format(&first);
        let second = parser::parse(&formatted).unwrap();
        assert_eq!(format(&second), formatted);
        assert!(formatted.contains("  id: uuid key"));
    }
}
