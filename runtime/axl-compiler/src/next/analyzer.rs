use std::collections::{BTreeMap, BTreeSet};

use super::ast::*;
use super::diagnostic::{Diagnostic, FixSafety, Repair, SourceSpan};
use super::ir::{GraphContract, GraphEdge, GraphGrant, GraphIr, GraphNode};

pub fn analyze(program: &Program) -> Result<GraphIr, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut declarations = BTreeMap::new();

    for declaration in &program.declarations {
        if !valid_name(declaration.name(), false) {
            diagnostics.push(Diagnostic::error(
                "AXL-N001",
                "names",
                format!("invalid declaration name '{}'", declaration.name()),
                declaration.span().clone(),
            ));
        }
        if let Some(previous) = declarations.insert(declaration.name(), declaration) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-N002",
                    "names",
                    format!("duplicate declaration '{}'", declaration.name()),
                    declaration.span().clone(),
                )
                .expected(
                    format!("a name different from '{}''", previous.name()),
                    declaration.name(),
                ),
            );
        }
    }

    for declaration in &program.declarations {
        match declaration {
            Declaration::Entity(entity) => check_entity(entity, &declarations, &mut diagnostics),
            Declaration::Capacity(capacity) => {
                check_capacity(capacity, &declarations, &mut diagnostics)
            }
            Declaration::Skill(skill) => check_skill(skill, &declarations, &mut diagnostics),
            Declaration::Blueprint(blueprint) => {
                check_blueprint(blueprint, &declarations, &mut diagnostics)
            }
            Declaration::Instance(instance) => {
                check_instance(instance, &declarations, &mut diagnostics)
            }
            Declaration::Agent(agent) => check_agent(agent, &mut diagnostics),
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut graph = lower(program);
    graph.canonicalize();
    Ok(graph)
}

fn check_entity(
    entity: &Entity,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = BTreeSet::new();
    for field in &entity.fields {
        if !valid_name(&field.name, false) {
            diagnostics.push(Diagnostic::error(
                "AXL-N101",
                "names",
                format!("invalid field name '{}.{}'", entity.name, field.name),
                field.span.clone(),
            ));
        }
        if !names.insert(&field.name) {
            diagnostics.push(Diagnostic::error(
                "AXL-N102",
                "names",
                format!("duplicate field '{}.{}'", entity.name, field.name),
                field.span.clone(),
            ));
        }
        check_type(&field.type_name, &field.span, declarations, diagnostics);
        for qualifier in &field.qualifiers {
            if !matches!(
                qualifier.as_str(),
                "key" | "required" | "optional" | "unique" | "index" | "private" | "readonly"
            ) {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-T102",
                        "types",
                        format!("unknown field qualifier '{qualifier}'"),
                        field.span.clone(),
                    )
                    .expected(
                        "key|required|optional|unique|index|private|readonly",
                        qualifier,
                    ),
                );
            }
        }
    }
}

fn check_capacity(
    capacity: &Capacity,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut names = BTreeSet::new();
    for operation in &capacity.operations {
        if !names.insert(&operation.name) {
            diagnostics.push(Diagnostic::error(
                "AXL-N202",
                "names",
                format!("duplicate operation '{}.{}'", capacity.name, operation.name),
                operation.span.clone(),
            ));
        }
        check_type(&operation.input, &operation.span, declarations, diagnostics);
        check_type(
            &operation.output,
            &operation.span,
            declarations,
            diagnostics,
        );
    }
}

fn check_skill(
    skill: &Skill,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declarations.get(skill.provides.as_str()) {
        Some(Declaration::Capacity(_)) => {}
        Some(other) => diagnostics.push(
            Diagnostic::error(
                "AXL-T301",
                "types",
                format!("skill '{}' can only provide a capacity", skill.name),
                skill.span.clone(),
            )
            .expected("capacity", declaration_kind(other)),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-T302",
                "types",
                format!(
                    "skill '{}' provides unknown capacity '{}'",
                    skill.name, skill.provides
                ),
                skill.span.clone(),
            )
            .expected("declared capacity", &skill.provides),
        ),
    }
    if let Some(native) = &skill.native
        && !matches!(
            native.target.as_str(),
            "rust" | "react" | "sql" | "ai" | "iot" | "wasm"
        )
    {
        diagnostics.push(
            Diagnostic::error(
                "AXL-I301",
                "interop",
                format!("unsupported native target '{}'", native.target),
                native.span.clone(),
            )
            .expected("rust|react|sql|ai|iot|wasm", &native.target),
        );
    }
    check_grants(&skill.effects, "effect", &skill.span, diagnostics);
    check_grants(&skill.capabilities, "capability", &skill.span, diagnostics);
}

fn check_blueprint(
    blueprint: &Blueprint,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut ports = BTreeMap::new();
    for port in &blueprint.ports {
        if !valid_name(&port.name, true) {
            diagnostics.push(Diagnostic::error(
                "AXL-N401",
                "names",
                format!("invalid port name '{}.{}'", blueprint.name, port.name),
                port.span.clone(),
            ));
        }
        if ports.insert(port.name.as_str(), port).is_some() {
            diagnostics.push(Diagnostic::error(
                "AXL-N402",
                "names",
                format!("duplicate port '{}.{}'", blueprint.name, port.name),
                port.span.clone(),
            ));
        }
        check_type(&port.type_name, &port.span, declarations, diagnostics);
    }

    if !blueprint
        .ports
        .iter()
        .any(|port| port.kind.is_customization_surface())
    {
        diagnostics.push(
            Diagnostic::error(
                "AXL-O401",
                "openness",
                format!(
                    "blueprint '{}' has no customization surface",
                    blueprint.name
                ),
                blueprint.span.clone(),
            )
            .expected("in|slot|hook|param|action|policy", "closed blueprint")
            .repair(
                FixSafety::Manual,
                Repair {
                    kind: "open".into(),
                    target: blueprint.name.clone(),
                    replacement: None,
                    candidates: vec![
                        "param option: text = \"default\"".into(),
                        "slot content: Capacity = Provider".into(),
                        "hook before: Capacity = Provider".into(),
                    ],
                },
            ),
        );
    }

    let mut bindings = BTreeMap::new();
    for binding in &blueprint.bindings {
        if bindings.insert(binding.port.as_str(), binding).is_some() {
            diagnostics.push(Diagnostic::error(
                "AXL-P402",
                "ports",
                format!(
                    "port '{}.{}' is connected more than once",
                    blueprint.name, binding.port
                ),
                binding.span.clone(),
            ));
        }
        let Some(port) = ports.get(binding.port.as_str()) else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P403",
                    "ports",
                    format!(
                        "binding references unknown port '{}.{}'",
                        blueprint.name, binding.port
                    ),
                    binding.span.clone(),
                )
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "rename".into(),
                        target: binding.port.clone(),
                        replacement: None,
                        candidates: ports.keys().map(|value| (*value).to_string()).collect(),
                    },
                ),
            );
            continue;
        };
        if !port.kind.accepts_provider() {
            diagnostics.push(Diagnostic::error(
                "AXL-P404",
                "ports",
                format!(
                    "{} surface '{}' cannot consume a provider",
                    port.kind.keyword(),
                    port.name
                ),
                binding.span.clone(),
            ));
            continue;
        }
        check_provider(
            blueprint,
            port,
            &binding.provider,
            &binding.span,
            declarations,
            diagnostics,
        );
    }

    for port in &blueprint.ports {
        match (&port.kind, &port.default) {
            (PortKind::Parameter, Some(default)) => {
                check_parameter_default(port, default, diagnostics);
            }
            (PortKind::Parameter, None) => diagnostics.push(
                Diagnostic::error(
                    "AXL-V402",
                    "values",
                    format!(
                        "parameter '{}.{}' requires a default",
                        blueprint.name, port.name
                    ),
                    port.span.clone(),
                )
                .expected("param name: Type = value", "missing default"),
            ),
            (_, Some(default)) if port.kind.accepts_provider() => {
                check_provider(
                    blueprint,
                    port,
                    default,
                    &port.span,
                    declarations,
                    diagnostics,
                );
            }
            (_, Some(_)) => diagnostics.push(
                Diagnostic::error(
                    "AXL-P407",
                    "ports",
                    format!(
                        "{} surface '{}.{}' cannot declare a default",
                        port.kind.keyword(),
                        blueprint.name,
                        port.name
                    ),
                    port.span.clone(),
                )
                .expected(
                    format!("{} name: Type", port.kind.keyword()),
                    "default value",
                ),
            ),
            (_, None) => {}
        }
        if matches!(port.kind, PortKind::Input)
            && port.default.is_none()
            && !bindings.contains_key(port.name.as_str())
        {
            let candidates = provider_candidates(&port.type_name, declarations);
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P401",
                    "ports",
                    format!(
                        "blueprint '{}' requires open input '{}: {}'",
                        blueprint.name, port.name, port.type_name
                    ),
                    port.span.clone(),
                )
                .expected(format!("provider of {}", port.type_name), "unconnected")
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "connect".into(),
                        target: format!("{}.{}", blueprint.name, port.name),
                        replacement: None,
                        candidates,
                    },
                ),
            );
        }
    }

    for contract in &blueprint.contracts {
        if contract.expression.is_empty() {
            diagnostics.push(Diagnostic::error(
                "AXL-V401",
                "contracts",
                "a contract expression cannot be empty",
                contract.span.clone(),
            ));
        }
    }
    check_grants(&blueprint.effects, "effect", &blueprint.span, diagnostics);
    check_grants(
        &blueprint.capabilities,
        "capability",
        &blueprint.span,
        diagnostics,
    );
}

fn check_parameter_default(port: &Port, value: &str, diagnostics: &mut Vec<Diagnostic>) {
    let valid = match port.type_name.as_str() {
        "bool" => matches!(value, "true" | "false"),
        "int" => value.parse::<i64>().is_ok(),
        "float" | "money" => value.parse::<f64>().is_ok(),
        "text" | "string" | "email" | "uuid" | "datetime" | "duration" => {
            serde_json::from_str::<String>(value).is_ok()
        }
        _ => false,
    };
    if !valid {
        diagnostics.push(
            Diagnostic::error(
                "AXL-V403",
                "values",
                format!(
                    "default '{}' does not match parameter type '{}'",
                    value, port.type_name
                ),
                port.span.clone(),
            )
            .expected(parameter_value_hint(&port.type_name), value),
        );
    }
}

fn check_instance(
    instance: &Instance,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let blueprint = match declarations.get(instance.blueprint.as_str()) {
        Some(Declaration::Blueprint(blueprint)) => blueprint,
        Some(other) => {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-I601",
                    "instances",
                    format!("instance '{}' requires a blueprint base", instance.name),
                    instance.span.clone(),
                )
                .expected("blueprint", declaration_kind(other)),
            );
            return;
        }
        None => {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-I602",
                    "instances",
                    format!(
                        "instance '{}' references unknown blueprint '{}'",
                        instance.name, instance.blueprint
                    ),
                    instance.span.clone(),
                )
                .expected("declared blueprint", &instance.blueprint)
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "rename".into(),
                        target: instance.blueprint.clone(),
                        replacement: None,
                        candidates: declarations
                            .iter()
                            .filter_map(|(name, declaration)| {
                                matches!(declaration, Declaration::Blueprint(_))
                                    .then_some((*name).to_string())
                            })
                            .collect(),
                    },
                ),
            );
            return;
        }
    };

    let surfaces = blueprint
        .ports
        .iter()
        .map(|port| (port.name.as_str(), port))
        .collect::<BTreeMap<_, _>>();
    let parameter_names = blueprint
        .ports
        .iter()
        .filter(|port| matches!(port.kind, PortKind::Parameter))
        .map(|port| port.name.clone())
        .collect::<Vec<_>>();
    let provider_names = blueprint
        .ports
        .iter()
        .filter(|port| port.kind.accepts_provider())
        .map(|port| port.name.clone())
        .collect::<Vec<_>>();

    let mut settings = BTreeSet::new();
    for setting in &instance.settings {
        if !settings.insert(setting.parameter.as_str()) {
            diagnostics.push(Diagnostic::error(
                "AXL-I603",
                "instances",
                format!(
                    "instance '{}.{}' is set more than once",
                    instance.name, setting.parameter
                ),
                setting.span.clone(),
            ));
        }
        match surfaces.get(setting.parameter.as_str()) {
            Some(port) if matches!(port.kind, PortKind::Parameter) => {
                check_parameter_default(port, &setting.value, diagnostics);
            }
            Some(port) => diagnostics.push(
                Diagnostic::error(
                    "AXL-I604",
                    "instances",
                    format!(
                        "{} surface '{}.{}' cannot be set as a parameter",
                        port.kind.keyword(),
                        instance.name,
                        setting.parameter
                    ),
                    setting.span.clone(),
                )
                .expected("parameter surface", port.kind.keyword()),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-I605",
                    "instances",
                    format!(
                        "instance '{}' has no parameter '{}'",
                        instance.name, setting.parameter
                    ),
                    setting.span.clone(),
                )
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "rename".into(),
                        target: setting.parameter.clone(),
                        replacement: None,
                        candidates: parameter_names.clone(),
                    },
                ),
            ),
        }
    }

    let mut bindings = BTreeSet::new();
    for binding in &instance.bindings {
        if !bindings.insert(binding.port.as_str()) {
            diagnostics.push(Diagnostic::error(
                "AXL-I606",
                "instances",
                format!(
                    "instance override '{}.{}' is connected more than once",
                    instance.name, binding.port
                ),
                binding.span.clone(),
            ));
        }
        match surfaces.get(binding.port.as_str()) {
            Some(port) if port.kind.accepts_provider() => check_provider(
                blueprint,
                port,
                &binding.provider,
                &binding.span,
                declarations,
                diagnostics,
            ),
            Some(port) => diagnostics.push(
                Diagnostic::error(
                    "AXL-I607",
                    "instances",
                    format!(
                        "{} surface '{}.{}' cannot accept a provider override",
                        port.kind.keyword(),
                        instance.name,
                        binding.port
                    ),
                    binding.span.clone(),
                )
                .expected("provider surface", port.kind.keyword()),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-I608",
                    "instances",
                    format!(
                        "instance '{}' has no provider surface '{}'",
                        instance.name, binding.port
                    ),
                    binding.span.clone(),
                )
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "rename".into(),
                        target: binding.port.clone(),
                        replacement: None,
                        candidates: provider_names.clone(),
                    },
                ),
            ),
        }
    }
}

fn parameter_value_hint(type_name: &str) -> &'static str {
    match type_name {
        "bool" => "true or false",
        "int" => "integer literal",
        "float" | "money" => "numeric literal",
        "text" | "string" | "email" | "uuid" | "datetime" | "duration" => "JSON string literal",
        _ => "scalar parameter type",
    }
}

fn check_provider(
    blueprint: &Blueprint,
    port: &Port,
    provider: &str,
    span: &SourceSpan,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match provider_type(provider, declarations) {
        Some(provided) if provided == port.type_name => {}
        Some(provided) => diagnostics.push(
            Diagnostic::error(
                "AXL-P405",
                "ports",
                format!(
                    "provider '{}' does not satisfy '{}.{}'",
                    provider, blueprint.name, port.name
                ),
                span.clone(),
            )
            .expected(&port.type_name, provided)
            .repair(
                FixSafety::Risky,
                Repair {
                    kind: "connect".into(),
                    target: format!("{}.{}", blueprint.name, port.name),
                    replacement: None,
                    candidates: provider_candidates(&port.type_name, declarations),
                },
            ),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-P406",
                "ports",
                format!("unknown provider '{provider}'"),
                span.clone(),
            )
            .expected(
                format!("provider of {}", port.type_name),
                "unknown declaration",
            )
            .repair(
                FixSafety::Risky,
                Repair {
                    kind: "connect".into(),
                    target: format!("{}.{}", blueprint.name, port.name),
                    replacement: None,
                    candidates: provider_candidates(&port.type_name, declarations),
                },
            ),
        ),
    }
}

fn check_agent(agent: &Agent, diagnostics: &mut Vec<Diagnostic>) {
    if agent.goals.is_empty() {
        diagnostics.push(Diagnostic::error(
            "AXL-A501",
            "agents",
            format!("agent '{}' requires at least one goal", agent.name),
            agent.span.clone(),
        ));
    }
    if agent.plans.is_empty() {
        diagnostics.push(Diagnostic::error(
            "AXL-A502",
            "agents",
            format!("agent '{}' requires at least one plan", agent.name),
            agent.span.clone(),
        ));
    }
    check_grants(&agent.effects, "effect", &agent.span, diagnostics);
    check_grants(&agent.capabilities, "capability", &agent.span, diagnostics);
}

fn check_type(
    type_name: &str,
    span: &SourceSpan,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if type_name.is_empty() {
        diagnostics.push(Diagnostic::error(
            "AXL-T001",
            "types",
            "type cannot be empty",
            span.clone(),
        ));
        return;
    }
    for reference in type_references(type_name) {
        if builtin_type(reference) || declarations.contains_key(reference) {
            continue;
        }
        diagnostics.push(
            Diagnostic::error(
                "AXL-T101",
                "types",
                format!("unknown type '{reference}'"),
                span.clone(),
            )
            .expected("built-in or declared type", reference),
        );
    }
}

fn type_references(type_name: &str) -> Vec<&str> {
    type_name
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|value| !value.is_empty())
        .collect()
}

fn builtin_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "unit"
            | "bool"
            | "int"
            | "float"
            | "text"
            | "string"
            | "email"
            | "uuid"
            | "datetime"
            | "money"
            | "bytes"
            | "duration"
            | "Result"
            | "Option"
            | "List"
            | "Set"
            | "Map"
            | "Stream"
            | "Future"
            | "UI"
            | "CrudApi"
    )
}

fn check_grants(
    values: &[String],
    kind: &str,
    span: &SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut unique = BTreeSet::new();
    for value in values {
        if value.is_empty() || !valid_name(value, true) {
            diagnostics.push(Diagnostic::error(
                "AXL-E001",
                "effects",
                format!("invalid {kind} '{value}'"),
                span.clone(),
            ));
        } else if !unique.insert(value) {
            diagnostics.push(Diagnostic::error(
                "AXL-E002",
                "effects",
                format!("duplicate {kind} '{value}'"),
                span.clone(),
            ));
        }
    }
}

fn provider_type(provider: &str, declarations: &BTreeMap<&str, &Declaration>) -> Option<String> {
    if let Some(Declaration::Skill(skill)) = declarations.get(provider) {
        return Some(skill.provides.clone());
    }
    let (blueprint_name, port_name) = provider.split_once('.')?;
    let Declaration::Blueprint(blueprint) = declarations.get(blueprint_name)? else {
        return None;
    };
    blueprint
        .ports
        .iter()
        .find(|port| matches!(port.kind, PortKind::Output) && port.name == port_name)
        .map(|port| port.type_name.clone())
}

fn provider_candidates(expected: &str, declarations: &BTreeMap<&str, &Declaration>) -> Vec<String> {
    let mut candidates = Vec::new();
    for (name, declaration) in declarations {
        match declaration {
            Declaration::Skill(skill) if skill.provides == expected => {
                candidates.push((*name).to_string())
            }
            Declaration::Blueprint(blueprint) => {
                for port in &blueprint.ports {
                    if matches!(port.kind, PortKind::Output) && port.type_name == expected {
                        candidates.push(format!("{}.{}", blueprint.name, port.name));
                    }
                }
            }
            _ => {}
        }
    }
    candidates
}

fn valid_name(value: &str, allow_dots: bool) -> bool {
    if value.is_empty() {
        return false;
    }
    value.split('.').all(|segment| {
        (!segment.is_empty() || !allow_dots)
            && segment
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
            && segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '_' || character == '-'
            })
    }) && (allow_dots || !value.contains('.'))
}

fn declaration_kind(declaration: &Declaration) -> &'static str {
    match declaration {
        Declaration::Entity(_) => "entity",
        Declaration::Capacity(_) => "capacity",
        Declaration::Skill(_) => "skill",
        Declaration::Blueprint(_) => "blueprint",
        Declaration::Instance(_) => "instance",
        Declaration::Agent(_) => "agent",
    }
}

fn lower(program: &Program) -> GraphIr {
    let mut graph = GraphIr {
        schema: "ax-ir/4.0".into(),
        app: program.name.clone(),
        nodes: vec![GraphNode {
            id: format!("app.{}", program.name),
            kind: "app".into(),
            name: program.name.clone(),
            type_name: None,
            implementation: None,
            metadata: BTreeMap::new(),
        }],
        edges: Vec::new(),
        contracts: Vec::new(),
        effects: Vec::new(),
        capabilities: Vec::new(),
    };

    for declaration in &program.declarations {
        match declaration {
            Declaration::Entity(entity) => lower_entity(entity, &mut graph),
            Declaration::Capacity(capacity) => lower_capacity(capacity, &mut graph),
            Declaration::Skill(skill) => lower_skill(skill, &mut graph),
            Declaration::Blueprint(blueprint) => lower_blueprint(blueprint, &mut graph),
            Declaration::Instance(instance) => lower_instance(instance, program, &mut graph),
            Declaration::Agent(agent) => lower_agent(agent, &mut graph),
        }
    }
    graph
}

fn lower_entity(entity: &Entity, graph: &mut GraphIr) {
    let entity_id = format!("entity.{}", entity.name);
    graph.nodes.push(node(&entity_id, "entity", &entity.name));
    for field in &entity.fields {
        let id = format!("{entity_id}.field.{}", field.name);
        let mut value = node(&id, "field", &field.name);
        value.type_name = Some(field.type_name.clone());
        if !field.qualifiers.is_empty() {
            value
                .metadata
                .insert("qualifiers".into(), field.qualifiers.join(","));
        }
        graph.nodes.push(value);
        graph.edges.push(edge(&entity_id, &id, "owns", None));
    }
}

fn lower_capacity(capacity: &Capacity, graph: &mut GraphIr) {
    let capacity_id = format!("capacity.{}", capacity.name);
    graph
        .nodes
        .push(node(&capacity_id, "capacity", &capacity.name));
    for operation in &capacity.operations {
        let id = format!("{capacity_id}.op.{}", operation.name);
        let mut value = node(&id, "operation", &operation.name);
        value.type_name = Some(format!("{}->{}", operation.input, operation.output));
        graph.nodes.push(value);
        graph.edges.push(edge(&capacity_id, &id, "owns", None));
    }
}

fn lower_skill(skill: &Skill, graph: &mut GraphIr) {
    let id = format!("skill.{}", skill.name);
    let mut value = node(&id, "skill", &skill.name);
    value.type_name = Some(skill.provides.clone());
    value.implementation = skill
        .native
        .as_ref()
        .map(|native| format!("{}::{}", native.target, native.symbol));
    graph.nodes.push(value);
    graph.edges.push(edge(
        &id,
        &format!("capacity.{}", skill.provides),
        "provides",
        Some(&skill.provides),
    ));
    append_grants(&id, &skill.effects, &mut graph.effects);
    append_grants(&id, &skill.capabilities, &mut graph.capabilities);
}

fn lower_blueprint(blueprint: &Blueprint, graph: &mut GraphIr) {
    let blueprint_id = format!("blueprint.{}", blueprint.name);
    graph
        .nodes
        .push(node(&blueprint_id, "blueprint", &blueprint.name));
    for port in &blueprint.ports {
        let port_kind = port.kind.graph_kind();
        let id = format!("{blueprint_id}.{port_kind}.{}", port.name);
        let mut value = node(&id, port_kind, &port.name);
        value.type_name = Some(port.type_name.clone());
        if let Some(default) = &port.default {
            value.metadata.insert("default".into(), default.clone());
        }
        graph.nodes.push(value);
        graph.edges.push(edge(&blueprint_id, &id, "owns", None));
        if let Some(default) = &port.default
            && port.kind.accepts_provider()
        {
            graph.edges.push(edge(
                &id,
                &provider_id(default),
                "default",
                Some(&port.type_name),
            ));
        }
    }
    for binding in &blueprint.bindings {
        if let Some(port) = blueprint
            .ports
            .iter()
            .find(|port| port.name == binding.port)
        {
            let port_kind = port.kind.graph_kind();
            graph.edges.push(edge(
                &format!("{blueprint_id}.{port_kind}.{}", port.name),
                &provider_id(&binding.provider),
                "bind",
                Some(&port.type_name),
            ));
        }
    }
    for contract in &blueprint.contracts {
        graph.contracts.push(GraphContract {
            owner: blueprint_id.clone(),
            kind: match contract.kind {
                ContractKind::Requires => "requires",
                ContractKind::Ensures => "ensures",
                ContractKind::Invariant => "invariant",
            }
            .into(),
            expression: contract.expression.clone(),
        });
    }
    append_grants(&blueprint_id, &blueprint.effects, &mut graph.effects);
    append_grants(
        &blueprint_id,
        &blueprint.capabilities,
        &mut graph.capabilities,
    );
}

fn lower_agent(agent: &Agent, graph: &mut GraphIr) {
    let agent_id = format!("agent.{}", agent.name);
    graph.nodes.push(node(&agent_id, "agent", &agent.name));
    for (kind, values) in [
        ("belief", &agent.beliefs),
        ("goal", &agent.goals),
        ("plan", &agent.plans),
    ] {
        for (index, value) in values.iter().enumerate() {
            let id = format!("{agent_id}.{kind}.{index}");
            let mut child = node(&id, kind, value);
            child.metadata.insert("order".into(), index.to_string());
            graph.nodes.push(child);
            graph.edges.push(edge(&agent_id, &id, "owns", None));
        }
    }
    append_grants(&agent_id, &agent.effects, &mut graph.effects);
    append_grants(&agent_id, &agent.capabilities, &mut graph.capabilities);
}

fn lower_instance(instance: &Instance, program: &Program, graph: &mut GraphIr) {
    let instance_id = format!("instance.{}", instance.name);
    let mut instance_node = node(&instance_id, "instance", &instance.name);
    instance_node.type_name = Some(instance.blueprint.clone());
    graph.nodes.push(instance_node);
    graph.edges.push(edge(
        &instance_id,
        &format!("blueprint.{}", instance.blueprint),
        "instantiates",
        Some(&instance.blueprint),
    ));

    let blueprint = program
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Blueprint(blueprint) if blueprint.name == instance.blueprint => {
                Some(blueprint)
            }
            _ => None,
        });
    let Some(blueprint) = blueprint else {
        return;
    };

    for setting in &instance.settings {
        let Some(port) = blueprint
            .ports
            .iter()
            .find(|port| port.name == setting.parameter)
        else {
            continue;
        };
        let id = format!("{instance_id}.setting.{}", setting.parameter);
        let mut value = node(&id, "setting", &setting.parameter);
        value.type_name = Some(port.type_name.clone());
        value.metadata.insert("value".into(), setting.value.clone());
        graph.nodes.push(value);
        graph.edges.push(edge(&instance_id, &id, "owns", None));
    }

    for binding in &instance.bindings {
        let Some(port) = blueprint
            .ports
            .iter()
            .find(|port| port.name == binding.port)
        else {
            continue;
        };
        let id = format!("{instance_id}.override.{}", binding.port);
        let mut value = node(&id, "override", &binding.port);
        value.type_name = Some(port.type_name.clone());
        value
            .metadata
            .insert("provider".into(), binding.provider.clone());
        graph.nodes.push(value);
        graph.edges.push(edge(&instance_id, &id, "owns", None));
        graph.edges.push(edge(
            &id,
            &provider_id(&binding.provider),
            "bind",
            Some(&port.type_name),
        ));
    }
}

fn node(id: &str, kind: &str, name: &str) -> GraphNode {
    GraphNode {
        id: id.into(),
        kind: kind.into(),
        name: name.into(),
        type_name: None,
        implementation: None,
        metadata: BTreeMap::new(),
    }
}

fn edge(from: &str, to: &str, kind: &str, interface: Option<&str>) -> GraphEdge {
    GraphEdge {
        from: from.into(),
        to: to.into(),
        kind: kind.into(),
        interface: interface.map(str::to_string),
    }
}

fn append_grants(owner: &str, values: &[String], grants: &mut Vec<GraphGrant>) {
    for value in values {
        grants.push(GraphGrant {
            owner: owner.into(),
            name: value.clone(),
        });
    }
}

fn provider_id(provider: &str) -> String {
    if let Some((blueprint, port)) = provider.split_once('.') {
        format!("blueprint.{blueprint}.output.{port}")
    } else {
        format!("skill.{provider}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::parser;

    #[test]
    fn reports_open_port_with_provider_candidate() {
        let source = r#"axl 4
app Demo
entity Customer
  id: uuid key
capacity CustomerStore
  op save Customer -> Result<Customer>
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
blueprint CRM
  param page_size: int = 25
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text
  in store: CustomerStore
"#;
        let program = parser::parse(source).unwrap();
        let diagnostics = analyze(&program).unwrap_err();
        let open = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "AXL-P401")
            .unwrap();
        assert_eq!(open.repairs[0].candidates, ["SqliteCustomers"]);
    }

    #[test]
    fn lowers_open_blueprint_to_typed_graph() {
        let source = r#"axl 4
app Demo
entity Customer
  id: uuid key
capacity CustomerStore
  op save Customer -> Result<Customer>
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
blueprint CRM
  param page_size: int = 25
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text
  in store: CustomerStore
  out api: CrudApi<Customer>
  use store = SqliteCustomers
  invariant Customer.id unique
"#;
        let graph = analyze(&parser::parse(source).unwrap()).unwrap();
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == "bind"
                && edge.to == "skill.SqliteCustomers"
                && edge.interface.as_deref() == Some("CustomerStore")
        }));
        assert_eq!(graph.contracts[0].kind, "invariant");
        assert!(graph.nodes.iter().any(|node| {
            node.kind == "parameter"
                && node.name == "page_size"
                && node.metadata.get("default").map(String::as_str) == Some("25")
        }));
        assert!(graph.nodes.iter().any(|node| node.kind == "state"));
        assert!(graph.nodes.iter().any(|node| node.kind == "event"));
        assert!(graph.nodes.iter().any(|node| node.kind == "error"));
    }

    #[test]
    fn rejects_a_blueprint_without_customization_surfaces() {
        let source = r#"axl 4
app Demo
blueprint Closed
  out view: UI
"#;
        let diagnostics = analyze(&parser::parse(source).unwrap()).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "AXL-O401")
        );
    }

    #[test]
    fn rejects_a_parameter_default_with_the_wrong_type() {
        let source = r#"axl 4
app Demo
blueprint Configurable
  param page_size: int = "many"
"#;
        let diagnostics = analyze(&parser::parse(source).unwrap()).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "AXL-V403")
        );
    }
}
