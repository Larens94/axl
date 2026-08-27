use super::ast::*;

pub fn format(program: &Program) -> String {
    let mut output = vec![
        format!("axl {}", program.version),
        format!("app {}", program.name),
    ];
    for declaration in &program.declarations {
        output.push(String::new());
        match declaration {
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
                    output.push(format!(
                        "  set {} = {}",
                        setting.parameter, setting.value
                    ));
                }
                for binding in &instance.bindings {
                    output.push(format!("  use {} = {}", binding.port, binding.provider));
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
