use std::collections::{BTreeMap, BTreeSet};

use super::ast::*;
use super::diagnostic::{Diagnostic, FixSafety, Repair, SourceSpan};
use super::expression::{self, BinaryOp, Expr, UnaryOp};
use super::ir::{GraphContract, GraphEdge, GraphGrant, GraphIr, GraphNode};

pub fn analyze(program: &Program) -> Result<GraphIr, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut declarations = BTreeMap::new();

    for declaration in &program.declarations {
        if matches!(declaration, Declaration::Subscription(_)) {
            continue;
        }
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
            Declaration::Enum(value) => check_enum(value, &mut diagnostics),
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
            Declaration::Flow(flow) => check_flow(flow, &declarations, &mut diagnostics),
            Declaration::Event(event) => check_event(event, &declarations, &mut diagnostics),
            Declaration::Subscription(_) => {}
            Declaration::Job(job) => check_job(job, &declarations, &mut diagnostics),
            Declaration::Api(api) => check_api(api, &declarations, &mut diagnostics),
            Declaration::Agent(agent) => check_agent(agent, &mut diagnostics),
        }
    }
    check_subscriptions(program, &declarations, &mut diagnostics);
    check_global_api_routes(program, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut graph = lower(program);
    graph.canonicalize();
    Ok(graph)
}

fn check_enum(value: &Enum, diagnostics: &mut Vec<Diagnostic>) {
    if value.variants.is_empty() {
        diagnostics.push(Diagnostic::error(
            "AXL-T701",
            "types",
            format!("enum '{}' requires at least one variant", value.name),
            value.span.clone(),
        ));
    }
    let mut variants = BTreeSet::new();
    for variant in &value.variants {
        if !valid_name(&variant.name, false) {
            diagnostics.push(Diagnostic::error(
                "AXL-N701",
                "names",
                format!("invalid enum variant '{}.{}'", value.name, variant.name),
                variant.span.clone(),
            ));
        }
        if !variants.insert(variant.name.as_str()) {
            diagnostics.push(Diagnostic::error(
                "AXL-N702",
                "names",
                format!("duplicate enum variant '{}.{}'", value.name, variant.name),
                variant.span.clone(),
            ));
        }
    }
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
    let mut config_names = BTreeSet::new();
    for config in &skill.configs {
        if !valid_name(&config.name, false) {
            diagnostics.push(Diagnostic::error(
                "AXL-N303",
                "names",
                format!("invalid skill config '{}.{}'", skill.name, config.name),
                config.span.clone(),
            ));
        }
        if !config_names.insert(&config.name) {
            diagnostics.push(Diagnostic::error(
                "AXL-N304",
                "names",
                format!("duplicate skill config '{}.{}'", skill.name, config.name),
                config.span.clone(),
            ));
        }
        check_type(&config.type_name, &config.span, declarations, diagnostics);
        if !scalar_value_matches(&config.type_name, &config.value) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-V305",
                    "values",
                    format!(
                        "config '{}' does not match type '{}'",
                        config.value, config.type_name
                    ),
                    config.span.clone(),
                )
                .expected(parameter_value_hint(&config.type_name), &config.value),
            );
        }
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
    let valid = scalar_value_matches(&port.type_name, value);
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

fn scalar_value_matches(type_name: &str, value: &str) -> bool {
    match type_name {
        "bool" => matches!(value, "true" | "false"),
        "int" => value.parse::<i64>().is_ok(),
        "float" | "money" => value.parse::<f64>().is_ok(),
        "text" | "string" | "email" | "uuid" | "datetime" | "duration" => {
            serde_json::from_str::<String>(value).is_ok()
        }
        _ => false,
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

fn check_flow(
    flow: &Flow,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_type(&flow.input, &flow.span, declarations, diagnostics);
    check_type(&flow.output, &flow.span, declarations, diagnostics);

    let result_type = generic_inner(&flow.output, "Result");
    let expected_return = result_type.unwrap_or(&flow.output);
    let mut variables = BTreeMap::from([("input".to_string(), flow.input.clone())]);
    let mut dependencies = BTreeMap::new();
    let mut bindings = BTreeMap::new();
    let mut return_count = 0;

    for dependency in &flow.dependencies {
        if !valid_name(&dependency.name, false) {
            diagnostics.push(Diagnostic::error(
                "AXL-N803",
                "names",
                format!("invalid flow dependency '{}'", dependency.name),
                dependency.span.clone(),
            ));
        }
        if dependencies
            .insert(dependency.name.as_str(), dependency)
            .is_some()
        {
            diagnostics.push(Diagnostic::error(
                "AXL-N804",
                "names",
                format!("duplicate flow dependency '{}'", dependency.name),
                dependency.span.clone(),
            ));
        }
        match declarations.get(dependency.capacity.as_str()) {
            Some(Declaration::Capacity(_)) => {}
            Some(found) => diagnostics.push(
                Diagnostic::error(
                    "AXL-X811",
                    "execution",
                    format!(
                        "flow dependency '{}.{}' requires a capacity",
                        flow.name, dependency.name
                    ),
                    dependency.span.clone(),
                )
                .expected("capacity", declaration_kind(found)),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-X811",
                    "execution",
                    format!(
                        "flow dependency '{}.{}' references unknown capacity '{}'",
                        flow.name, dependency.name, dependency.capacity
                    ),
                    dependency.span.clone(),
                )
                .expected("declared capacity", &dependency.capacity),
            ),
        }
        if let Some(provider) = &dependency.default {
            check_flow_provider(
                flow,
                dependency,
                provider,
                &dependency.span,
                declarations,
                diagnostics,
            );
        }
    }

    for binding in &flow.bindings {
        if bindings.insert(binding.port.as_str(), binding).is_some() {
            diagnostics.push(Diagnostic::error(
                "AXL-X812",
                "execution",
                format!(
                    "flow dependency '{}.{}' is connected more than once",
                    flow.name, binding.port
                ),
                binding.span.clone(),
            ));
        }
        match dependencies.get(binding.port.as_str()) {
            Some(dependency) => check_flow_provider(
                flow,
                dependency,
                &binding.provider,
                &binding.span,
                declarations,
                diagnostics,
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-X813",
                    "execution",
                    format!("flow '{}' has no dependency '{}'", flow.name, binding.port),
                    binding.span.clone(),
                )
                .expected("declared flow dependency", binding.port.as_str()),
            ),
        }
    }

    for dependency in &flow.dependencies {
        if dependency.default.is_none() && !bindings.contains_key(dependency.name.as_str()) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-X814",
                    "execution",
                    format!(
                        "flow dependency '{}.{}' has no provider",
                        flow.name, dependency.name
                    ),
                    dependency.span.clone(),
                )
                .expected(
                    format!("provider of {}", dependency.capacity),
                    "unconnected",
                )
                .repair(
                    FixSafety::Risky,
                    Repair {
                        kind: "connect".into(),
                        target: format!("{}.{}", flow.name, dependency.name),
                        replacement: None,
                        candidates: provider_candidates(&dependency.capacity, declarations),
                    },
                ),
            );
        }
    }

    for (index, statement) in flow.statements.iter().enumerate() {
        match statement {
            FlowStatement::Let {
                name,
                expression,
                span,
            } => {
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                if let Some(type_name) =
                    infer_source_expression(expression, span, &variables, declarations, diagnostics)
                    && variables.insert(name.clone(), type_name).is_some()
                {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Require {
                expression, span, ..
            } => {
                if result_type.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X804",
                            "execution",
                            format!(
                                "flow '{}' uses require but does not return Result",
                                flow.name
                            ),
                            span.clone(),
                        )
                        .expected("Result<T>", &flow.output),
                    );
                }
                if let Some(found) =
                    infer_source_expression(expression, span, &variables, declarations, diagnostics)
                    && found != "bool"
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X803",
                            "execution",
                            "require expression must be boolean",
                            span.clone(),
                        )
                        .expected("bool", found),
                    );
                }
            }
            FlowStatement::Call {
                name,
                dependency,
                operation,
                argument,
                propagate,
                span,
            } => {
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                let Some(dependency_value) = dependencies.get(dependency.as_str()) else {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X815",
                        "execution",
                        format!("call references unknown flow dependency '{dependency}'"),
                        span.clone(),
                    ));
                    continue;
                };
                let Some(Declaration::Capacity(capacity)) =
                    declarations.get(dependency_value.capacity.as_str())
                else {
                    continue;
                };
                let Some(operation_value) = capacity
                    .operations
                    .iter()
                    .find(|candidate| candidate.name == *operation)
                else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X816",
                            "execution",
                            format!(
                                "capacity '{}' has no operation '{}'",
                                capacity.name, operation
                            ),
                            span.clone(),
                        )
                        .expected("declared capacity operation", operation.as_str()),
                    );
                    continue;
                };
                if let Some(found) =
                    infer_source_expression(argument, span, &variables, declarations, diagnostics)
                    && !same_type(&found, &operation_value.input)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X817",
                            "execution",
                            format!(
                                "call '{}.{}' receives the wrong argument type",
                                dependency, operation
                            ),
                            span.clone(),
                        )
                        .expected(&operation_value.input, found),
                    );
                    continue;
                }
                let variable_type = match generic_inner(&operation_value.output, "Result") {
                    Some(inner) if *propagate && result_type.is_some() => inner.to_string(),
                    Some(_) if !*propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X818",
                                "execution",
                                "a Result operation call requires '?' propagation",
                                span.clone(),
                            )
                            .expected("call value = port.operation(argument)?", "missing ?"),
                        );
                        continue;
                    }
                    Some(_) => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X819",
                                "execution",
                                "'?' requires the containing flow to return Result<T>",
                                span.clone(),
                            )
                            .expected("Result<T> flow output", &flow.output),
                        );
                        continue;
                    }
                    None if *propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X820",
                                "execution",
                                "'?' can only propagate a Result<T> operation",
                                span.clone(),
                            )
                            .expected("Result<T> operation output", &operation_value.output),
                        );
                        continue;
                    }
                    None => operation_value.output.clone(),
                };
                if variables.insert(name.clone(), variable_type).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Attempt {
                name,
                dependency,
                operation,
                argument,
                propagate,
                retry,
                timeout_ms,
                span,
            } => {
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                if *retry > 10 {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X906",
                            "resilience",
                            "attempt retry count exceeds the safety limit",
                            span.clone(),
                        )
                        .expected("0..10", retry.to_string()),
                    );
                }
                if !(1..=60_000).contains(timeout_ms) {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X907",
                            "resilience",
                            "attempt timeout is outside the safety limits",
                            span.clone(),
                        )
                        .expected("1..60000 milliseconds", timeout_ms.to_string()),
                    );
                }
                let Some(dependency_value) = dependencies.get(dependency.as_str()) else {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X901",
                        "resilience",
                        format!("attempt references unknown dependency '{dependency}'"),
                        span.clone(),
                    ));
                    continue;
                };
                let Some(Declaration::Capacity(capacity)) =
                    declarations.get(dependency_value.capacity.as_str())
                else {
                    continue;
                };
                let Some(operation_value) = capacity
                    .operations
                    .iter()
                    .find(|candidate| candidate.name == *operation)
                else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X902",
                            "resilience",
                            format!(
                                "capacity '{}' has no resilient operation '{}'",
                                capacity.name, operation
                            ),
                            span.clone(),
                        )
                        .expected("declared capacity operation", operation),
                    );
                    continue;
                };
                if !operation_value.idempotent {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X905",
                            "resilience",
                            format!(
                                "attempt requires idempotent operation '{}.{}'",
                                capacity.name, operation
                            ),
                            span.clone(),
                        )
                        .expected("operation qualifier 'idempotent'", "not idempotent"),
                    );
                }
                if let Some(found) =
                    infer_source_expression(argument, span, &variables, declarations, diagnostics)
                    && found != operation_value.input
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X903",
                            "resilience",
                            "attempt operation receives the wrong argument type",
                            span.clone(),
                        )
                        .expected(&operation_value.input, found),
                    );
                }
                let variable_type = match generic_inner(&operation_value.output, "Result") {
                    Some(inner) if *propagate && result_type.is_some() => Some(inner.to_string()),
                    Some(_) if !*propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X904",
                                "resilience",
                                "a resilient Result operation requires '?' propagation",
                                span.clone(),
                            )
                            .expected("attempt value = port.operation(argument)?", "missing ?"),
                        );
                        None
                    }
                    Some(_) => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X904",
                                "resilience",
                                "attempt '?' requires a Result<T> containing flow",
                                span.clone(),
                            )
                            .expected("Result<T> flow output", &flow.output),
                        );
                        None
                    }
                    None if *propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X904",
                                "resilience",
                                "attempt '?' requires a Result<T> operation",
                                span.clone(),
                            )
                            .expected("Result<T> operation output", &operation_value.output),
                        );
                        None
                    }
                    None => Some(operation_value.output.clone()),
                };
                if let Some(variable_type) = variable_type
                    && variables.insert(name.clone(), variable_type).is_some()
                {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Make {
                name,
                type_name,
                fields,
                span,
            } => {
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                let Some(Declaration::Entity(entity)) = declarations.get(type_name.as_str()) else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X831",
                            "execution",
                            format!("record constructor requires entity type '{type_name}'"),
                            span.clone(),
                        )
                        .expected("declared entity", type_name),
                    );
                    continue;
                };
                let mut assigned = BTreeSet::new();
                for field in fields {
                    if !assigned.insert(field.name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "AXL-X834",
                            "execution",
                            format!(
                                "record '{}: {}' assigns field '{}' more than once",
                                name, type_name, field.name
                            ),
                            field.span.clone(),
                        ));
                    }
                    let Some(entity_field) = entity
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == field.name)
                    else {
                        diagnostics.push(Diagnostic::error(
                            "AXL-X832",
                            "execution",
                            format!("entity '{}' has no field '{}'", entity.name, field.name),
                            field.span.clone(),
                        ));
                        continue;
                    };
                    if let Some(found) = infer_source_expression(
                        &field.expression,
                        &field.span,
                        &variables,
                        declarations,
                        diagnostics,
                    ) && !same_type(&found, &entity_field.type_name)
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X833",
                                "execution",
                                format!(
                                    "constructed field '{}.{}' has the wrong type",
                                    entity.name, field.name
                                ),
                                field.span.clone(),
                            )
                            .expected(&entity_field.type_name, found),
                        );
                    }
                }
                for field in &entity.fields {
                    let optional = field.qualifiers.iter().any(|value| value == "optional")
                        || generic_inner(&field.type_name, "Option").is_some();
                    if !optional && !assigned.contains(field.name.as_str()) {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X835",
                                "execution",
                                format!(
                                    "record '{}: {}' is missing required field '{}'",
                                    name, type_name, field.name
                                ),
                                span.clone(),
                            )
                            .expected(format!("field {} = expression", field.name), "missing"),
                        );
                    }
                }
                if variables.insert(name.clone(), type_name.clone()).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Fold {
                name,
                type_name,
                collection,
                initial,
                item,
                update,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                if !valid_name(item, false) || matches!(item.as_str(), "input" | "value") {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N805",
                        "names",
                        format!("invalid fold item '{item}'"),
                        span.clone(),
                    ));
                }
                let collection_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let item_type = collection_type.as_deref().and_then(|found| {
                    generic_inner(found, "List").or_else(|| generic_inner(found, "Set"))
                });
                if collection_type.is_some() && item_type.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X841",
                            "execution",
                            "fold source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            collection_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                if let Some(found) =
                    infer_source_expression(initial, span, &variables, declarations, diagnostics)
                    && !fold_compatible(&found, type_name)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X842",
                            "execution",
                            "fold initial value does not match its result type",
                            span.clone(),
                        )
                        .expected(type_name, found),
                    );
                }
                if let Some(item_type) = item_type {
                    let mut fold_variables = variables.clone();
                    fold_variables.insert("value".into(), type_name.clone());
                    fold_variables.insert(item.clone(), item_type.to_string());
                    if let Some(found) = infer_source_expression(
                        update,
                        span,
                        &fold_variables,
                        declarations,
                        diagnostics,
                    ) && !fold_compatible(&found, type_name)
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X843",
                                "execution",
                                "fold next value does not match its result type",
                                span.clone(),
                            )
                            .expected(type_name, found),
                        );
                    }
                }
                if variables.insert(name.clone(), type_name.clone()).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Run {
                name,
                flow: target_name,
                argument,
                propagate,
                span,
            } => {
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                let Some(Declaration::Flow(target)) = declarations.get(target_name.as_str()) else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X851",
                            "execution",
                            format!("run references unknown flow '{target_name}'"),
                            span.clone(),
                        )
                        .expected("declared flow", target_name),
                    );
                    continue;
                };
                if target.name == flow.name {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X856",
                        "execution",
                        format!("flow '{}' cannot directly run itself", flow.name),
                        span.clone(),
                    ));
                    continue;
                }
                if let Some(found) =
                    infer_source_expression(argument, span, &variables, declarations, diagnostics)
                    && !same_type(&found, &target.input)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X852",
                            "execution",
                            format!("flow '{}' receives the wrong argument type", target.name),
                            span.clone(),
                        )
                        .expected(&target.input, found),
                    );
                    continue;
                }
                let variable_type = match generic_inner(&target.output, "Result") {
                    Some(inner) if *propagate && result_type.is_some() => inner.to_string(),
                    Some(_) if !*propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X853",
                                "execution",
                                "a Result flow run requires '?' propagation",
                                span.clone(),
                            )
                            .expected("run value = Flow(argument)?", "missing ?"),
                        );
                        continue;
                    }
                    Some(_) => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X854",
                                "execution",
                                "'?' requires the containing flow to return Result<T>",
                                span.clone(),
                            )
                            .expected("Result<T> flow output", &flow.output),
                        );
                        continue;
                    }
                    None if *propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X855",
                                "execution",
                                "'?' can only propagate a Result<T> flow",
                                span.clone(),
                            )
                            .expected("Result<T> target flow output", &target.output),
                        );
                        continue;
                    }
                    None => target.output.clone(),
                };
                if variables.insert(name.clone(), variable_type).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Match {
                name,
                type_name,
                subject,
                cases,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                if !valid_name(name, false) || name == "input" {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N801",
                        "names",
                        format!("invalid flow variable '{name}'"),
                        span.clone(),
                    ));
                }
                let subject_type =
                    infer_source_expression(subject, span, &variables, declarations, diagnostics);
                let value_enum = subject_type
                    .as_deref()
                    .and_then(|found| declarations.get(found))
                    .and_then(|declaration| match declaration {
                        Declaration::Enum(value) => Some(value),
                        _ => None,
                    });
                if subject_type.is_some() && value_enum.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X861",
                            "execution",
                            "match subject must be an enum",
                            span.clone(),
                        )
                        .expected(
                            "declared enum",
                            subject_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                if let Some(value_enum) = value_enum {
                    let mut matched = BTreeSet::new();
                    for case in cases {
                        if !matched.insert(case.variant.as_str()) {
                            diagnostics.push(Diagnostic::error(
                                "AXL-X863",
                                "execution",
                                format!(
                                    "duplicate match case '{}.{}'",
                                    value_enum.name, case.variant
                                ),
                                case.span.clone(),
                            ));
                        }
                        if !value_enum
                            .variants
                            .iter()
                            .any(|variant| variant.name == case.variant)
                        {
                            diagnostics.push(Diagnostic::error(
                                "AXL-X862",
                                "execution",
                                format!(
                                    "enum '{}' has no variant '{}'",
                                    value_enum.name, case.variant
                                ),
                                case.span.clone(),
                            ));
                        }
                        if let Some(found) = infer_source_expression(
                            &case.expression,
                            &case.span,
                            &variables,
                            declarations,
                            diagnostics,
                        ) && !fold_compatible(&found, type_name)
                        {
                            diagnostics.push(
                                Diagnostic::error(
                                    "AXL-X865",
                                    "execution",
                                    format!("match case '{}' has the wrong type", case.variant),
                                    case.span.clone(),
                                )
                                .expected(type_name, found),
                            );
                        }
                    }
                    let missing = value_enum
                        .variants
                        .iter()
                        .filter(|variant| !matched.contains(variant.name.as_str()))
                        .map(|variant| variant.name.clone())
                        .collect::<Vec<_>>();
                    if !missing.is_empty() {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X864",
                                "execution",
                                format!("match on '{}' is not exhaustive", value_enum.name),
                                span.clone(),
                            )
                            .expected(
                                format!("cases for {}", missing.join(", ")),
                                "missing variants",
                            ),
                        );
                    }
                }
                if variables.insert(name.clone(), type_name.clone()).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-N802",
                        "names",
                        format!("flow variable '{name}' is defined more than once"),
                        span.clone(),
                    ));
                }
            }
            FlowStatement::Map {
                name,
                type_name,
                collection,
                item,
                expression,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X871",
                            "execution",
                            "map source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                let output_item = collection_inner(type_name).map(|(_, inner)| inner);
                if output_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X872",
                            "execution",
                            "map result must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected("List<T>|Set<T>", type_name),
                    );
                }
                if let (Some(source_item), Some(output_item)) = (source_item, output_item) {
                    let mut transform_variables = variables.clone();
                    transform_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        expression,
                        span,
                        &transform_variables,
                        declarations,
                        diagnostics,
                    ) && !fold_compatible(&found, output_item)
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X875",
                                "execution",
                                "map value does not match its collection item type",
                                span.clone(),
                            )
                            .expected(output_item, found),
                        );
                    }
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Filter {
                name,
                type_name,
                collection,
                item,
                predicate,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X871",
                            "execution",
                            "filter source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                if source_type
                    .as_deref()
                    .is_some_and(|found| found != type_name)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X873",
                            "execution",
                            "filter result must preserve its source collection type",
                            span.clone(),
                        )
                        .expected(source_type.as_deref().unwrap_or("unknown"), type_name),
                    );
                }
                if collection_inner(type_name).is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X872",
                            "execution",
                            "filter result must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected("List<T>|Set<T>", type_name),
                    );
                }
                if let Some(source_item) = source_item {
                    let mut transform_variables = variables.clone();
                    transform_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        predicate,
                        span,
                        &transform_variables,
                        declarations,
                        diagnostics,
                    ) && found != "bool"
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X874",
                                "execution",
                                "filter predicate must be boolean",
                                span.clone(),
                            )
                            .expected("bool", found),
                        );
                    }
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Sort {
                name,
                type_name,
                collection,
                item,
                key,
                direction,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X876",
                            "execution",
                            "sort source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                let output_item = generic_inner(type_name, "List");
                if let Some(source_item) = source_item {
                    if output_item != Some(source_item) {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X877",
                                "execution",
                                "sort result must be List<T> with the source item type",
                                span.clone(),
                            )
                            .expected(format!("List<{source_item}>"), type_name),
                        );
                    }
                    let mut sort_variables = variables.clone();
                    sort_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        key,
                        span,
                        &sort_variables,
                        declarations,
                        diagnostics,
                    ) && !sortable_type(&found, declarations)
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X878",
                                "execution",
                                "sort key must be an ordered scalar or enum",
                                span.clone(),
                            )
                            .expected("ordered scalar|enum", found),
                        );
                    }
                } else if output_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X877",
                            "execution",
                            "sort result must be List<T>",
                            span.clone(),
                        )
                        .expected("List<T>", type_name),
                    );
                }
                if !matches!(direction.as_str(), "asc" | "desc") {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X879",
                            "execution",
                            "sort direction must be asc or desc",
                            span.clone(),
                        )
                        .expected("asc|desc", direction),
                    );
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Group {
                name,
                type_name,
                collection,
                item,
                key,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X881",
                            "execution",
                            "group source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                let output = generic_pair(type_name, "Map");
                if let Some(source_item) = source_item {
                    let output_shape_valid = output.is_some_and(|(_, values)| {
                        generic_inner(values, "List") == Some(source_item)
                    });
                    if !output_shape_valid {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X882",
                                "execution",
                                "group result must be Map<K,List<T>> with the source item type",
                                span.clone(),
                            )
                            .expected(format!("Map<K,List<{source_item}>>"), type_name),
                        );
                    }
                    let mut group_variables = variables.clone();
                    group_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        key,
                        span,
                        &group_variables,
                        declarations,
                        diagnostics,
                    ) {
                        if !ordered_type(&found, declarations) {
                            diagnostics.push(
                                Diagnostic::error(
                                    "AXL-X883",
                                    "execution",
                                    "group key must be a string-like scalar or enum",
                                    span.clone(),
                                )
                                .expected("string-like scalar|enum", &found),
                            );
                        }
                        if let Some((declared_key, _)) = output
                            && declared_key != found
                        {
                            diagnostics.push(
                                Diagnostic::error(
                                    "AXL-X884",
                                    "execution",
                                    "group key does not match the declared Map key type",
                                    span.clone(),
                                )
                                .expected(declared_key, found),
                            );
                        }
                    }
                } else if output.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X882",
                            "execution",
                            "group result must be Map<K,List<T>>",
                            span.clone(),
                        )
                        .expected("Map<K,List<T>>", type_name),
                    );
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Parallel {
                name,
                type_name,
                collection,
                item,
                flow: target_name,
                argument,
                propagate,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X891",
                            "execution",
                            "parallel source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                let target = declarations.get(target_name.as_str()).and_then(|value| {
                    if let Declaration::Flow(target) = value {
                        Some(target)
                    } else {
                        None
                    }
                });
                let Some(target) = target else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X892",
                            "execution",
                            format!("parallel references unknown flow '{target_name}'"),
                            span.clone(),
                        )
                        .expected("declared flow", target_name),
                    );
                    continue;
                };
                if target.name == flow.name {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X892",
                        "execution",
                        format!("flow '{}' cannot run itself in parallel", flow.name),
                        span.clone(),
                    ));
                    continue;
                }
                if let Some(source_item) = source_item {
                    let mut parallel_variables = variables.clone();
                    parallel_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        argument,
                        span,
                        &parallel_variables,
                        declarations,
                        diagnostics,
                    ) && found != target.input
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X893",
                                "execution",
                                format!(
                                    "parallel flow '{}' receives the wrong argument type",
                                    target.name
                                ),
                                span.clone(),
                            )
                            .expected(&target.input, found),
                        );
                    }
                }
                let effective_output = match generic_inner(&target.output, "Result") {
                    Some(inner) if *propagate && result_type.is_some() => Some(inner),
                    Some(_) if !*propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X894",
                                "execution",
                                "parallel Result flow requires '?' propagation",
                                span.clone(),
                            )
                            .expected("run = Flow(argument)?", "missing ?"),
                        );
                        None
                    }
                    Some(_) => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X894",
                                "execution",
                                "parallel '?' requires a Result<T> containing flow",
                                span.clone(),
                            )
                            .expected("Result<T> flow output", &flow.output),
                        );
                        None
                    }
                    None if *propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X894",
                                "execution",
                                "parallel '?' requires a Result<T> target flow",
                                span.clone(),
                            )
                            .expected("Result<T> target output", &target.output),
                        );
                        None
                    }
                    None => Some(target.output.as_str()),
                };
                if let Some(effective_output) = effective_output {
                    let expected = format!("List<{effective_output}>");
                    if type_name != &expected {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X895",
                                "execution",
                                "parallel result must list the target flow output",
                                span.clone(),
                            )
                            .expected(expected, type_name),
                        );
                    }
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Race {
                name,
                type_name,
                collection,
                item,
                flow: target_name,
                argument,
                propagate,
                span,
            } => {
                check_type(type_name, span, declarations, diagnostics);
                check_transform_names(name, item, span, diagnostics);
                let source_type = infer_source_expression(
                    collection,
                    span,
                    &variables,
                    declarations,
                    diagnostics,
                );
                let source_item = source_type
                    .as_deref()
                    .and_then(collection_inner)
                    .map(|(_, inner)| inner);
                if source_type.is_some() && source_item.is_none() {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X911",
                            "resilience",
                            "race source must be List<T> or Set<T>",
                            span.clone(),
                        )
                        .expected(
                            "List<T>|Set<T>",
                            source_type.as_deref().unwrap_or("unknown"),
                        ),
                    );
                }
                let target = declarations.get(target_name.as_str()).and_then(|value| {
                    if let Declaration::Flow(target) = value {
                        Some(target)
                    } else {
                        None
                    }
                });
                let Some(target) = target else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X912",
                            "resilience",
                            format!("race references unknown flow '{target_name}'"),
                            span.clone(),
                        )
                        .expected("declared flow", target_name),
                    );
                    continue;
                };
                if target.name == flow.name {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X912",
                        "resilience",
                        format!("flow '{}' cannot race itself", flow.name),
                        span.clone(),
                    ));
                    continue;
                }
                if !flow_is_idempotent(target, declarations, &mut BTreeSet::new()) {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X916",
                            "resilience",
                            format!("race target flow '{}' is not idempotent", target.name),
                            span.clone(),
                        )
                        .expected("flow using only idempotent operations", &target.name),
                    );
                }
                if let Some(source_item) = source_item {
                    let mut race_variables = variables.clone();
                    race_variables.insert(item.clone(), source_item.to_string());
                    if let Some(found) = infer_source_expression(
                        argument,
                        span,
                        &race_variables,
                        declarations,
                        diagnostics,
                    ) && found != target.input
                    {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X913",
                                "resilience",
                                format!(
                                    "race flow '{}' receives the wrong argument type",
                                    target.name
                                ),
                                span.clone(),
                            )
                            .expected(&target.input, found),
                        );
                    }
                }
                let effective_output = match generic_inner(&target.output, "Result") {
                    Some(inner) if *propagate && result_type.is_some() => Some(inner),
                    Some(_) if !*propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X914",
                                "resilience",
                                "race Result flow requires '?' propagation",
                                span.clone(),
                            )
                            .expected("run = Flow(argument)?", "missing ?"),
                        );
                        None
                    }
                    Some(_) => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X914",
                                "resilience",
                                "race '?' requires a Result<T> containing flow",
                                span.clone(),
                            )
                            .expected("Result<T> flow output", &flow.output),
                        );
                        None
                    }
                    None if *propagate => {
                        diagnostics.push(
                            Diagnostic::error(
                                "AXL-X914",
                                "resilience",
                                "race '?' requires a Result<T> target flow",
                                span.clone(),
                            )
                            .expected("Result<T> target output", &target.output),
                        );
                        None
                    }
                    None => Some(target.output.as_str()),
                };
                if let Some(effective_output) = effective_output
                    && type_name != effective_output
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X915",
                            "resilience",
                            "race result must match the target flow output",
                            span.clone(),
                        )
                        .expected(effective_output, type_name),
                    );
                }
                bind_transform_result(name, type_name, span, &mut variables, diagnostics);
            }
            FlowStatement::Emit {
                event,
                argument,
                span,
            } => {
                let Some(Declaration::Event(event_decl)) = declarations.get(event.as_str()) else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-E905",
                            "events",
                            format!("unknown event '{event}'"),
                            span.clone(),
                        )
                        .expected("declared event", event),
                    );
                    continue;
                };
                if let Some(found) =
                    infer_source_expression(argument, span, &variables, declarations, diagnostics)
                    && !same_type(&found, &event_decl.payload)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-E906",
                            "events",
                            format!("emit payload type must match event '{event}'"),
                            span.clone(),
                        )
                        .expected(&event_decl.payload, found),
                    );
                }
            }
            FlowStatement::Enqueue {
                job,
                argument,
                span,
            } => {
                let Some(Declaration::Job(job_decl)) = declarations.get(job.as_str()) else {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-J907",
                            "jobs",
                            format!("unknown job '{job}'"),
                            span.clone(),
                        )
                        .expected("declared job", job),
                    );
                    continue;
                };
                let Some(Declaration::Flow(target)) = declarations.get(job_decl.flow.as_str())
                else {
                    continue;
                };
                if let Some(found) =
                    infer_source_expression(argument, span, &variables, declarations, diagnostics)
                    && !same_type(&found, &target.input)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-J908",
                            "jobs",
                            format!("enqueue payload type must match job '{job}'"),
                            span.clone(),
                        )
                        .expected(&target.input, found),
                    );
                }
            }
            FlowStatement::Return { expression, span } => {
                return_count += 1;
                if index + 1 != flow.statements.len() {
                    diagnostics.push(Diagnostic::error(
                        "AXL-X805",
                        "execution",
                        "return must be the final flow statement",
                        span.clone(),
                    ));
                }
                if let Some(found) =
                    infer_source_expression(expression, span, &variables, declarations, diagnostics)
                    && !same_type(&found, expected_return)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-X806",
                            "execution",
                            format!("flow '{}' returns the wrong type", flow.name),
                            span.clone(),
                        )
                        .expected(expected_return, found),
                    );
                }
            }
        }
    }

    if return_count != 1 {
        diagnostics.push(
            Diagnostic::error(
                "AXL-X807",
                "execution",
                format!("flow '{}' requires exactly one return", flow.name),
                flow.span.clone(),
            )
            .expected("one final return", return_count.to_string()),
        );
    }
}

fn flow_is_idempotent(
    flow: &Flow,
    declarations: &BTreeMap<&str, &Declaration>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if !visiting.insert(flow.name.clone()) {
        return false;
    }
    let safe = flow.statements.iter().all(|statement| match statement {
        FlowStatement::Call {
            dependency,
            operation,
            ..
        }
        | FlowStatement::Attempt {
            dependency,
            operation,
            ..
        } => flow
            .dependencies
            .iter()
            .find(|candidate| candidate.name == *dependency)
            .and_then(|dependency| declarations.get(dependency.capacity.as_str()))
            .and_then(|declaration| match declaration {
                Declaration::Capacity(capacity) => capacity
                    .operations
                    .iter()
                    .find(|candidate| candidate.name == *operation),
                _ => None,
            })
            .is_some_and(|operation| operation.idempotent),
        FlowStatement::Run { flow, .. }
        | FlowStatement::Parallel { flow, .. }
        | FlowStatement::Race { flow, .. } => declarations
            .get(flow.as_str())
            .and_then(|declaration| match declaration {
                Declaration::Flow(flow) => Some(flow),
                _ => None,
            })
            .is_some_and(|flow| flow_is_idempotent(flow, declarations, visiting)),
        FlowStatement::Emit { .. } | FlowStatement::Enqueue { .. } => false,
        _ => true,
    });
    visiting.remove(&flow.name);
    safe
}

fn check_flow_provider(
    flow: &Flow,
    dependency: &FlowDependency,
    provider: &str,
    span: &SourceSpan,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match provider_type(provider, declarations) {
        Some(provided) if provided == dependency.capacity => {}
        Some(provided) => diagnostics.push(
            Diagnostic::error(
                "AXL-X821",
                "execution",
                format!(
                    "provider '{}' does not satisfy flow dependency '{}.{}'",
                    provider, flow.name, dependency.name
                ),
                span.clone(),
            )
            .expected(&dependency.capacity, provided),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-X822",
                "execution",
                format!("unknown flow provider '{provider}'"),
                span.clone(),
            )
            .expected(
                format!("provider of {}", dependency.capacity),
                "unknown declaration",
            ),
        ),
    }
}

fn check_api_middleware(
    middleware: &ApiMiddleware,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expected_contract = match middleware.phase.as_str() {
        "request" => Some("op process HttpRequest -> Result<HttpRequest> idempotent"),
        "response" => Some("op process HttpResponse -> Result<HttpResponse> idempotent"),
        _ => {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H918",
                    "http",
                    format!("unsupported middleware phase '{}'", middleware.phase),
                    middleware.span.clone(),
                )
                .expected("request or response", &middleware.phase),
            );
            None
        }
    };
    match declarations.get(middleware.capacity.as_str()) {
        Some(Declaration::Capacity(capacity)) => {
            if let Some(expected) = expected_contract {
                let process = capacity.operations.iter().find(|op| op.name == "process");
                let valid = process.is_some_and(|op| {
                    op.idempotent
                        && generic_inner(&op.output, "Result") == Some(op.input.as_str())
                        && match middleware.phase.as_str() {
                            "request" => http_request_envelope(&op.input, declarations),
                            "response" => http_response_envelope(&op.input, declarations),
                            _ => false,
                        }
                });
                if !valid {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-H919",
                            "http",
                            format!(
                                "middleware capacity '{}' has an invalid contract",
                                middleware.capacity
                            ),
                            middleware.span.clone(),
                        )
                        .expected(expected, "missing or incompatible operation"),
                    );
                }
            }
        }
        Some(found) => diagnostics.push(
            Diagnostic::error(
                "AXL-H920",
                "http",
                format!(
                    "middleware type '{}' is not a capacity",
                    middleware.capacity
                ),
                middleware.span.clone(),
            )
            .expected("capacity", declaration_kind(found)),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-H920",
                "http",
                format!("unknown middleware capacity '{}'", middleware.capacity),
                middleware.span.clone(),
            )
            .expected("declared capacity", &middleware.capacity),
        ),
    }
    match provider_type(&middleware.provider, declarations) {
        Some(provided) if provided == middleware.capacity => {}
        Some(provided) => diagnostics.push(
            Diagnostic::error(
                "AXL-H921",
                "http",
                format!(
                    "middleware provider '{}' is incompatible",
                    middleware.provider
                ),
                middleware.span.clone(),
            )
            .expected(&middleware.capacity, provided),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-H922",
                "http",
                format!("unknown middleware provider '{}'", middleware.provider),
                middleware.span.clone(),
            )
            .expected(
                format!("provider of {}", middleware.capacity),
                "unknown provider",
            ),
        ),
    }
}

fn http_request_envelope(type_name: &str, declarations: &BTreeMap<&str, &Declaration>) -> bool {
    let Some(Declaration::Entity(entity)) = declarations.get(type_name) else {
        return false;
    };
    let method = entity.fields.iter().find(|field| field.name == "method");
    let path = entity.fields.iter().find(|field| field.name == "path");
    let headers = entity.fields.iter().find(|field| field.name == "headers");
    method.is_some_and(|field| field.type_name == "text")
        && path.is_some_and(|field| field.type_name == "text")
        && headers.is_some_and(|field| field.type_name == "Map<text,text>")
}

fn http_response_envelope(type_name: &str, declarations: &BTreeMap<&str, &Declaration>) -> bool {
    let Some(Declaration::Entity(entity)) = declarations.get(type_name) else {
        return false;
    };
    let status = entity.fields.iter().find(|field| field.name == "status");
    let headers = entity.fields.iter().find(|field| field.name == "headers");
    let body = entity.fields.iter().find(|field| field.name == "body");
    status.is_some_and(|field| field.type_name == "int")
        && headers.is_some_and(|field| field.type_name == "Map<text,text>")
        && body.is_some_and(|field| field.type_name == "text")
}

fn check_api(
    api: &Api,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for middleware in &api.middlewares {
        check_api_middleware(middleware, declarations, diagnostics);
    }
    if let Some(auth) = &api.auth {
        if auth.scheme != "bearer" {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H908",
                    "http",
                    format!("unsupported auth scheme '{}'", auth.scheme),
                    auth.span.clone(),
                )
                .expected("bearer", &auth.scheme),
            );
        }
        match declarations.get(auth.capacity.as_str()) {
            Some(Declaration::Capacity(capacity)) => {
                let authorize = capacity.operations.iter().find(|op| op.name == "authorize");
                if !authorize.is_some_and(|op| {
                    op.input == "text" && op.output == "Result<bool>" && op.idempotent
                }) {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-H909",
                            "http",
                            format!("auth capacity '{}' has an invalid contract", auth.capacity),
                            auth.span.clone(),
                        )
                        .expected(
                            "op authorize text -> Result<bool> idempotent",
                            "missing or incompatible operation",
                        ),
                    );
                }
            }
            Some(found) => diagnostics.push(
                Diagnostic::error(
                    "AXL-H910",
                    "http",
                    format!("auth type '{}' is not a capacity", auth.capacity),
                    auth.span.clone(),
                )
                .expected("capacity", declaration_kind(found)),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-H910",
                    "http",
                    format!("unknown auth capacity '{}'", auth.capacity),
                    auth.span.clone(),
                )
                .expected("declared capacity", &auth.capacity),
            ),
        }
        match provider_type(&auth.provider, declarations) {
            Some(provided) if provided == auth.capacity => {}
            Some(provided) => diagnostics.push(
                Diagnostic::error(
                    "AXL-H911",
                    "http",
                    format!("auth provider '{}' is incompatible", auth.provider),
                    auth.span.clone(),
                )
                .expected(&auth.capacity, provided),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-H912",
                    "http",
                    format!("unknown auth provider '{}'", auth.provider),
                    auth.span.clone(),
                )
                .expected(format!("provider of {}", auth.capacity), "unknown provider"),
            ),
        }
    }
    if api.routes.is_empty() {
        diagnostics.push(Diagnostic::error(
            "AXL-H901",
            "http",
            format!("api '{}' requires at least one route", api.name),
            api.span.clone(),
        ));
    }
    let mut routes = BTreeSet::new();
    for route in &api.routes {
        if !matches!(
            route.method.as_str(),
            "get" | "post" | "put" | "patch" | "delete"
        ) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H902",
                    "http",
                    format!("unsupported HTTP method '{}'", route.method),
                    route.span.clone(),
                )
                .expected("get|post|put|patch|delete", &route.method),
            );
        }
        if !valid_http_path(&route.path) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H903",
                    "http",
                    format!("invalid HTTP path '{}'", route.path),
                    route.span.clone(),
                )
                .expected("absolute path without query or fragment", &route.path),
            );
        }
        if !routes.insert((route.method.clone(), normalized_http_path(&route.path))) {
            diagnostics.push(Diagnostic::error(
                "AXL-H904",
                "http",
                format!(
                    "api '{}' declares route '{} {}' more than once",
                    api.name, route.method, route.path
                ),
                route.span.clone(),
            ));
        }
        check_type(&route.input, &route.span, declarations, diagnostics);
        check_type(&route.output, &route.span, declarations, diagnostics);
        check_request_bindings(route, declarations, diagnostics);
        match declarations.get(route.flow.as_str()) {
            Some(Declaration::Flow(flow)) => {
                if flow.input != route.input || flow.output != route.output {
                    diagnostics.push(
                        Diagnostic::error(
                            "AXL-H906",
                            "http",
                            format!(
                                "route '{} {}' does not match flow '{}'",
                                route.method, route.path, route.flow
                            ),
                            route.span.clone(),
                        )
                        .expected(
                            format!("{} -> {}", flow.input, flow.output),
                            format!("{} -> {}", route.input, route.output),
                        ),
                    );
                }
            }
            Some(found) => diagnostics.push(
                Diagnostic::error(
                    "AXL-H905",
                    "http",
                    format!("route target '{}' is not a flow", route.flow),
                    route.span.clone(),
                )
                .expected("flow", declaration_kind(found)),
            ),
            None => diagnostics.push(
                Diagnostic::error(
                    "AXL-H905",
                    "http",
                    format!("route references unknown flow '{}'", route.flow),
                    route.span.clone(),
                )
                .expected("declared flow", &route.flow),
            ),
        }
    }
}

fn check_request_bindings(
    route: &ApiRoute,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if route.input_source != "composite" {
        if route.input_source == "body" {
            return;
        }
        let binding = route.bindings.first();
        let name = binding
            .and_then(|value| value.name.as_deref())
            .unwrap_or_default();
        check_request_binding_name_and_path(route, binding, name, diagnostics);
        if !http_binding_type(&route.input, declarations) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H915",
                    "http",
                    format!("request binding cannot construct type '{}'", route.input),
                    route.span.clone(),
                )
                .expected("scalar or enum route input", &route.input),
            );
        }
        return;
    }

    let Some(Declaration::Entity(entity)) = declarations.get(route.input.as_str()) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-H915",
                "http",
                format!("composite request requires entity input '{}'", route.input),
                route.span.clone(),
            )
            .expected("declared entity", &route.input),
        );
        return;
    };
    let mut targets = BTreeSet::new();
    for binding in &route.bindings {
        let target = binding.target.as_deref().unwrap_or_default();
        if !valid_name(target, false) {
            diagnostics.push(Diagnostic::error(
                "AXL-H913",
                "http",
                format!("invalid request target '{target}'"),
                binding.span.clone(),
            ));
        }
        let Some(field) = entity.fields.iter().find(|field| field.name == target) else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H916",
                    "http",
                    format!("unknown composite request field '{}.{target}'", entity.name),
                    binding.span.clone(),
                )
                .expected("declared entity field", target),
            );
            continue;
        };
        if !targets.insert(target) {
            diagnostics.push(Diagnostic::error(
                "AXL-H916",
                "http",
                format!(
                    "duplicate composite request field '{}.{target}'",
                    entity.name
                ),
                binding.span.clone(),
            ));
        }
        let name = binding.name.as_deref().unwrap_or_default();
        check_request_binding_name_and_path(route, Some(binding), name, diagnostics);
        if matches!(
            binding.source.as_str(),
            "path" | "query" | "header" | "cookie"
        ) && !http_binding_type(&field.type_name, declarations)
        {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-H915",
                    "http",
                    format!(
                        "request source cannot construct field '{}.{target}'",
                        entity.name
                    ),
                    binding.span.clone(),
                )
                .expected("scalar or enum field", &field.type_name),
            );
        }
    }
    for field in &entity.fields {
        let optional = field.qualifiers.iter().any(|value| value == "optional")
            || generic_inner(&field.type_name, "Option").is_some();
        if !optional && !targets.contains(field.name.as_str()) {
            diagnostics.push(Diagnostic::error(
                "AXL-H917",
                "http",
                format!(
                    "composite request is missing '{}.{}'",
                    entity.name, field.name
                ),
                route.span.clone(),
            ));
        }
    }
}

fn check_request_binding_name_and_path(
    route: &ApiRoute,
    binding: Option<&HttpRequestBinding>,
    name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(binding) = binding else { return };
    if binding.source != "body" && !valid_name(name, false) {
        diagnostics.push(Diagnostic::error(
            "AXL-H913",
            "http",
            format!("invalid request binding name '{name}'"),
            binding.span.clone(),
        ));
    }
    if binding.source == "path"
        && !route
            .path
            .split('/')
            .any(|segment| segment == format!("{{{name}}}"))
    {
        diagnostics.push(
            Diagnostic::error(
                "AXL-H914",
                "http",
                format!("path binding '{name}' has no matching placeholder"),
                binding.span.clone(),
            )
            .expected(format!("{{{name}}}"), &route.path),
        );
    }
}

fn check_global_api_routes(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let mut routes = BTreeMap::new();
    for api in program
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Api(api) => Some(api),
            _ => None,
        })
    {
        for route in &api.routes {
            let key = (route.method.clone(), normalized_http_path(&route.path));
            if routes
                .insert(key, api.name.as_str())
                .is_some_and(|owner| owner != api.name)
            {
                diagnostics.push(Diagnostic::error(
                    "AXL-H907",
                    "http",
                    format!(
                        "HTTP route '{} {}' conflicts across APIs",
                        route.method, route.path
                    ),
                    route.span.clone(),
                ));
            }
        }
    }
}

fn valid_http_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.contains(['?', '#', ' '])
        && !path.contains("//")
        && path.split('/').all(|segment| {
            if !segment.contains(['{', '}']) {
                return true;
            }
            segment
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
                .is_some_and(|name| valid_name(name, false))
        })
}

fn normalized_http_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                "{}"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn http_binding_type(type_name: &str, declarations: &BTreeMap<&str, &Declaration>) -> bool {
    matches!(
        type_name,
        "bool"
            | "int"
            | "float"
            | "money"
            | "text"
            | "string"
            | "email"
            | "uuid"
            | "datetime"
            | "duration"
    ) || matches!(declarations.get(type_name), Some(Declaration::Enum(_)))
}

fn infer_source_expression(
    source: &str,
    span: &SourceSpan,
    variables: &BTreeMap<String, String>,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    match expression::parse(source) {
        Ok(expression) => match infer_expression(&expression, variables, declarations) {
            Ok(type_name) => Some(type_name),
            Err(message) => {
                diagnostics.push(Diagnostic::error(
                    "AXL-X802",
                    "execution",
                    message,
                    span.clone(),
                ));
                None
            }
        },
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "AXL-X801",
                "execution",
                message,
                span.clone(),
            ));
            None
        }
    }
}

fn infer_expression(
    expression: &Expr,
    variables: &BTreeMap<String, String>,
    declarations: &BTreeMap<&str, &Declaration>,
) -> Result<String, String> {
    match expression {
        Expr::Path(path) => infer_path(path, variables, declarations),
        Expr::Bool(_) => Ok("bool".into()),
        Expr::Int(_) => Ok("int".into()),
        Expr::Float(_) => Ok("float".into()),
        Expr::String(_) => Ok("text".into()),
        Expr::List(items) => {
            let Some(first) = items.first() else {
                return Err("an empty list literal has no inferable item type".into());
            };
            let mut item_type = infer_expression(first, variables, declarations)?;
            for item in &items[1..] {
                let found = infer_expression(item, variables, declarations)?;
                if same_type(&item_type, &found) {
                    continue;
                }
                if numeric_type(&item_type) && numeric_type(&found) {
                    item_type = numeric_result(&item_type, &found).into();
                    continue;
                }
                return Err(format!(
                    "list literal items are incompatible: '{item_type}' and '{found}'"
                ));
            }
            Ok(format!("List<{item_type}>"))
        }
        Expr::Unary(operator, value) => {
            let found = infer_expression(value, variables, declarations)?;
            match operator {
                UnaryOp::Not if found == "bool" => Ok("bool".into()),
                UnaryOp::Negate if numeric_type(&found) => Ok(found),
                UnaryOp::Not => Err(format!("operator ! requires bool, found '{found}'")),
                UnaryOp::Negate => Err(format!("unary - requires a numeric type, found '{found}'")),
            }
        }
        Expr::Binary(left, operator, right) => {
            let left = infer_expression(left, variables, declarations)?;
            let right = infer_expression(right, variables, declarations)?;
            match operator {
                BinaryOp::Or | BinaryOp::And if left == "bool" && right == "bool" => {
                    Ok("bool".into())
                }
                BinaryOp::Or | BinaryOp::And => Err(format!(
                    "logical operator requires bool operands, found '{left}' and '{right}'"
                )),
                BinaryOp::Equal | BinaryOp::NotEqual
                    if same_type(&left, &right)
                        || (numeric_type(&left) && numeric_type(&right)) =>
                {
                    Ok("bool".into())
                }
                BinaryOp::Equal | BinaryOp::NotEqual => Err(format!(
                    "equality operands are incompatible: '{left}' and '{right}'"
                )),
                BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                    if (numeric_type(&left) && numeric_type(&right))
                        || (left == right && ordered_type(&left, declarations)) =>
                {
                    Ok("bool".into())
                }
                BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual => Err(format!(
                    "comparison operands are incompatible: '{left}' and '{right}'"
                )),
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
                    if numeric_type(&left) && numeric_type(&right) =>
                {
                    Ok(numeric_result(&left, &right).into())
                }
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => Err(
                    format!("arithmetic requires numeric operands, found '{left}' and '{right}'"),
                ),
            }
        }
        Expr::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            let condition_type = infer_expression(condition, variables, declarations)?;
            if condition_type != "bool" {
                return Err(format!(
                    "conditional requires bool, found '{condition_type}'"
                ));
            }
            let true_type = infer_expression(when_true, variables, declarations)?;
            let false_type = infer_expression(when_false, variables, declarations)?;
            if same_type(&true_type, &false_type) {
                Ok(true_type)
            } else if numeric_type(&true_type) && numeric_type(&false_type) {
                Ok(numeric_result(&true_type, &false_type).into())
            } else {
                Err(format!(
                    "conditional branches are incompatible: '{true_type}' and '{false_type}'"
                ))
            }
        }
    }
}

fn infer_path(
    path: &[String],
    variables: &BTreeMap<String, String>,
    declarations: &BTreeMap<&str, &Declaration>,
) -> Result<String, String> {
    let Some(first) = path.first() else {
        return Err("empty value path".into());
    };
    if let Some(Declaration::Enum(value)) = declarations.get(first.as_str()) {
        if path.len() == 2 && value.variants.iter().any(|variant| variant.name == path[1]) {
            return Ok(value.name.clone());
        }
        return Err(format!("unknown enum variant '{}'", path.join(".")));
    }

    let mut current = variables
        .get(first)
        .cloned()
        .ok_or_else(|| format!("unknown flow value '{first}'"))?;
    for segment in &path[1..] {
        let Some(Declaration::Entity(entity)) = declarations.get(current.as_str()) else {
            return Err(format!("type '{current}' has no field '{segment}'"));
        };
        current = entity
            .fields
            .iter()
            .find(|field| field.name == *segment)
            .map(|field| field.type_name.clone())
            .ok_or_else(|| format!("entity '{}' has no field '{segment}'", entity.name))?;
    }
    Ok(current)
}

fn numeric_type(value: &str) -> bool {
    matches!(value, "int" | "float" | "money")
}

fn fold_compatible(found: &str, expected: &str) -> bool {
    same_type(found, expected) || (numeric_type(found) && numeric_type(expected))
}

fn collection_inner(value: &str) -> Option<(&str, &str)> {
    for kind in ["List", "Set"] {
        if let Some(inner) = generic_inner(value, kind) {
            return Some((kind, inner));
        }
    }
    None
}

fn sortable_type(value: &str, declarations: &BTreeMap<&str, &Declaration>) -> bool {
    matches!(
        value,
        "int" | "float" | "money" | "text" | "string" | "email" | "uuid" | "datetime" | "duration"
    ) || matches!(declarations.get(value), Some(Declaration::Enum(_)))
}

fn check_transform_names(
    name: &str,
    item: &str,
    span: &SourceSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !valid_name(name, false) || name == "input" {
        diagnostics.push(Diagnostic::error(
            "AXL-N801",
            "names",
            format!("invalid flow variable '{name}'"),
            span.clone(),
        ));
    }
    if !valid_name(item, false) || matches!(item, "input" | "value") {
        diagnostics.push(Diagnostic::error(
            "AXL-N806",
            "names",
            format!("invalid collection item '{item}'"),
            span.clone(),
        ));
    }
}

fn bind_transform_result(
    name: &str,
    type_name: &str,
    span: &SourceSpan,
    variables: &mut BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if variables
        .insert(name.to_string(), type_name.to_string())
        .is_some()
    {
        diagnostics.push(Diagnostic::error(
            "AXL-N802",
            "names",
            format!("flow variable '{name}' is defined more than once"),
            span.clone(),
        ));
    }
}

fn numeric_result<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left == "float" || right == "float" {
        "float"
    } else if left == "money" || right == "money" {
        "money"
    } else {
        "int"
    }
}

fn ordered_type(value: &str, declarations: &BTreeMap<&str, &Declaration>) -> bool {
    matches!(
        value,
        "text" | "string" | "email" | "uuid" | "datetime" | "duration"
    ) || matches!(declarations.get(value), Some(Declaration::Enum(_)))
}

fn same_type(left: &str, right: &str) -> bool {
    left == right
}

fn generic_inner<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value
        .strip_prefix(name)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

fn generic_pair<'a>(value: &'a str, name: &str) -> Option<(&'a str, &'a str)> {
    let inner = generic_inner(value, name)?;
    let mut depth = 0usize;
    for (index, character) in inner.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                let left = inner[..index].trim();
                let right = inner[index + 1..].trim();
                return (!left.is_empty() && !right.is_empty()).then_some((left, right));
            }
            _ => {}
        }
    }
    None
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

fn check_event(
    event: &EventDecl,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if event.name.contains('.') || !valid_name(&event.name, false) {
        diagnostics.push(Diagnostic::error(
            "AXL-N001",
            "names",
            format!("invalid event name '{}'", event.name),
            event.span.clone(),
        ));
    }
    check_type(&event.payload, &event.span, declarations, diagnostics);
}

fn check_subscriptions(
    program: &Program,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for declaration in &program.declarations {
        let Declaration::Subscription(subscription) = declaration else {
            continue;
        };
        let Some(Declaration::Event(event)) = declarations.get(subscription.event.as_str()) else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-E901",
                    "events",
                    format!("unknown event '{}'", subscription.event),
                    subscription.span.clone(),
                )
                .expected("declared event", &subscription.event),
            );
            continue;
        };
        if !same_type(&subscription.payload, &event.payload) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-E902",
                    "events",
                    format!(
                        "subscription payload type must match event '{}'",
                        subscription.event
                    ),
                    subscription.span.clone(),
                )
                .expected(&event.payload, &subscription.payload),
            );
        }
        let Some(Declaration::Flow(flow)) = declarations.get(subscription.flow.as_str()) else {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-E903",
                    "events",
                    format!("unknown subscriber flow '{}'", subscription.flow),
                    subscription.span.clone(),
                )
                .expected("declared flow", &subscription.flow),
            );
            continue;
        };
        if !same_type(&flow.input, &event.payload) {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-E904",
                    "events",
                    format!(
                        "subscriber flow '{}' input must match event '{}'",
                        flow.name, event.name
                    ),
                    subscription.span.clone(),
                )
                .expected(&event.payload, &flow.input),
            );
        }
    }
}

fn check_job(
    job: &JobDecl,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if job.name.contains('.') || !valid_name(&job.name, false) {
        diagnostics.push(Diagnostic::error(
            "AXL-N001",
            "names",
            format!("invalid job name '{}'", job.name),
            job.span.clone(),
        ));
    }
    if job.retry > 10 {
        diagnostics.push(
            Diagnostic::error(
                "AXL-J904",
                "jobs",
                "job retry count exceeds the safety limit",
                job.span.clone(),
            )
            .expected("0..10", job.retry.to_string()),
        );
    }
    if job.retry > 0 && !job.idempotent {
        diagnostics.push(
            Diagnostic::error(
                "AXL-J904",
                "jobs",
                "job retry requires the idempotent qualifier",
                job.span.clone(),
            )
            .expected("idempotent", "missing idempotent"),
        );
    }
    if let Some(schedule) = &job.schedule
        && parse_schedule_millis(schedule).is_none()
    {
        diagnostics.push(
            Diagnostic::error(
                "AXL-J903",
                "jobs",
                format!("unsupported job schedule '{schedule}'"),
                job.span.clone(),
            )
            .expected("every <n>ms|s|m", schedule),
        );
    }
    let Some(Declaration::Flow(flow)) = declarations.get(job.flow.as_str()) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-J901",
                "jobs",
                format!("unknown job flow '{}'", job.flow),
                job.span.clone(),
            )
            .expected("declared flow", &job.flow),
        );
        check_job_store(job, declarations, diagnostics);
        return;
    };
    if job.schedule.is_some() && flow.input != "unit" {
        diagnostics.push(
            Diagnostic::error(
                "AXL-J902",
                "jobs",
                format!("scheduled job '{}' requires a unit flow input", job.name),
                job.span.clone(),
            )
            .expected("unit", &flow.input),
        );
    }
    check_job_store(job, declarations, diagnostics);
}

fn check_job_store(
    job: &JobDecl,
    declarations: &BTreeMap<&str, &Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match declarations.get(job.store_capacity.as_str()) {
        Some(Declaration::Capacity(capacity)) => {
            if !job_store_contract(capacity) {
                diagnostics.push(
                    Diagnostic::error(
                        "AXL-J906",
                        "jobs",
                        format!(
                            "job store capacity '{}' has an invalid contract",
                            job.store_capacity
                        ),
                        job.span.clone(),
                    )
                    .expected(
                        "op enqueue/claim/finish text|unit contracts",
                        "missing or incompatible operation",
                    ),
                );
            }
        }
        Some(found) => diagnostics.push(
            Diagnostic::error(
                "AXL-J905",
                "jobs",
                format!("job store type '{}' is not a capacity", job.store_capacity),
                job.span.clone(),
            )
            .expected("capacity", declaration_kind(found)),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-J905",
                "jobs",
                format!("unknown job store capacity '{}'", job.store_capacity),
                job.span.clone(),
            )
            .expected("declared capacity", &job.store_capacity),
        ),
    }
    match provider_type(&job.store_provider, declarations) {
        Some(provided) if provided == job.store_capacity => {}
        Some(provided) => diagnostics.push(
            Diagnostic::error(
                "AXL-J905",
                "jobs",
                format!(
                    "job store provider '{}' is incompatible",
                    job.store_provider
                ),
                job.span.clone(),
            )
            .expected(&job.store_capacity, provided),
        ),
        None => diagnostics.push(
            Diagnostic::error(
                "AXL-J905",
                "jobs",
                format!("unknown job store provider '{}'", job.store_provider),
                job.span.clone(),
            )
            .expected(
                format!("provider of {}", job.store_capacity),
                &job.store_provider,
            ),
        ),
    }
}

fn job_store_contract(capacity: &Capacity) -> bool {
    let enqueue = capacity.operations.iter().find(|op| op.name == "enqueue");
    let claim = capacity.operations.iter().find(|op| op.name == "claim");
    let finish = capacity.operations.iter().find(|op| op.name == "finish");
    enqueue.is_some_and(|op| {
        op.idempotent && op.input == "text" && generic_inner(&op.output, "Result") == Some("text")
    }) && claim.is_some_and(|op| {
        op.idempotent
            && op.input == "unit"
            && generic_inner(&op.output, "Result") == Some("List<text>")
    }) && finish
        .is_some_and(|op| op.input == "text" && generic_inner(&op.output, "Result") == Some("text"))
}

pub(crate) fn parse_schedule_millis(schedule: &str) -> Option<u64> {
    let rest = schedule.strip_prefix("every ")?.trim();
    let (number, unit) = rest.split_at(rest.find(|c: char| !c.is_ascii_digit())?);
    let amount = number.parse::<u64>().ok()?;
    match unit {
        "ms" => Some(amount),
        "s" => amount.checked_mul(1_000),
        "m" => amount.checked_mul(60_000),
        _ => None,
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
        Declaration::Enum(_) => "enum",
        Declaration::Entity(_) => "entity",
        Declaration::Capacity(_) => "capacity",
        Declaration::Skill(_) => "skill",
        Declaration::Blueprint(_) => "blueprint",
        Declaration::Instance(_) => "instance",
        Declaration::Flow(_) => "flow",
        Declaration::Event(_) => "event",
        Declaration::Subscription(_) => "subscription",
        Declaration::Job(_) => "job",
        Declaration::Api(_) => "api",
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
            Declaration::Enum(value) => lower_enum(value, &mut graph),
            Declaration::Entity(entity) => lower_entity(entity, &mut graph),
            Declaration::Capacity(capacity) => lower_capacity(capacity, &mut graph),
            Declaration::Skill(skill) => lower_skill(skill, &mut graph),
            Declaration::Blueprint(blueprint) => lower_blueprint(blueprint, &mut graph),
            Declaration::Instance(instance) => lower_instance(instance, program, &mut graph),
            Declaration::Flow(flow) => lower_flow(flow, &mut graph),
            Declaration::Event(event) => lower_event(event, &mut graph),
            Declaration::Subscription(_) => {}
            Declaration::Job(job) => lower_job(job, &mut graph),
            Declaration::Api(api) => lower_api(api, &mut graph),
            Declaration::Agent(agent) => lower_agent(agent, &mut graph),
        }
    }
    let mut subscription_order = 0usize;
    for declaration in &program.declarations {
        if let Declaration::Subscription(subscription) = declaration {
            lower_subscription(subscription, subscription_order, &mut graph);
            subscription_order += 1;
        }
    }
    graph
}

fn lower_event(event: &EventDecl, graph: &mut GraphIr) {
    let id = format!("event.{}", event.name);
    let mut value = node(&id, "event", &event.name);
    value.type_name = Some(event.payload.clone());
    graph.nodes.push(value);
}

fn lower_job(job: &JobDecl, graph: &mut GraphIr) {
    let id = format!("job.{}", job.name);
    let mut value = node(&id, "job", &job.name);
    value.type_name = Some(job.store_capacity.clone());
    value.metadata.insert("flow".into(), job.flow.clone());
    value.metadata.insert("retry".into(), job.retry.to_string());
    value
        .metadata
        .insert("idempotent".into(), job.idempotent.to_string());
    if let Some(schedule) = &job.schedule {
        value.metadata.insert("schedule".into(), schedule.clone());
    }
    value
        .metadata
        .insert("provider".into(), job.store_provider.clone());
    graph.nodes.push(value);
    graph
        .edges
        .push(edge(&id, &format!("flow.{}", job.flow), "dispatch", None));
    graph.edges.push(edge(
        &id,
        &provider_id(&job.store_provider),
        "default",
        Some(&job.store_capacity),
    ));
}

fn lower_subscription(subscription: &Subscription, order: usize, graph: &mut GraphIr) {
    let id = format!("subscription.{order}");
    let mut value = node(&id, "subscription", &subscription.event);
    value.type_name = Some(subscription.payload.clone());
    value.metadata.insert("order".into(), order.to_string());
    value
        .metadata
        .insert("flow".into(), subscription.flow.clone());
    graph.nodes.push(value);
    graph.edges.push(edge(
        &id,
        &format!("event.{}", subscription.event),
        "bind",
        Some(&subscription.payload),
    ));
    graph.edges.push(edge(
        &id,
        &format!("flow.{}", subscription.flow),
        "dispatch",
        Some(&subscription.payload),
    ));
}

fn lower_enum(value: &Enum, graph: &mut GraphIr) {
    let enum_id = format!("enum.{}", value.name);
    graph.nodes.push(node(&enum_id, "enum", &value.name));
    for variant in &value.variants {
        let id = format!("{enum_id}.variant.{}", variant.name);
        graph.nodes.push(node(&id, "variant", &variant.name));
        graph.edges.push(edge(&enum_id, &id, "owns", None));
    }
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
        value
            .metadata
            .insert("idempotent".into(), operation.idempotent.to_string());
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
    for config in &skill.configs {
        let config_id = format!("{id}.config.{}", config.name);
        let mut value = node(&config_id, "config", &config.name);
        value.type_name = Some(config.type_name.clone());
        value.metadata.insert("value".into(), config.value.clone());
        graph.nodes.push(value);
        graph.edges.push(edge(&id, &config_id, "owns", None));
    }
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

fn lower_flow(flow: &Flow, graph: &mut GraphIr) {
    let flow_id = format!("flow.{}", flow.name);
    let mut flow_node = node(&flow_id, "flow", &flow.name);
    flow_node.type_name = Some(format!("{}->{}", flow.input, flow.output));
    graph.nodes.push(flow_node);

    for dependency in &flow.dependencies {
        let id = format!("{flow_id}.input.{}", dependency.name);
        let mut value = node(&id, "input", &dependency.name);
        value.type_name = Some(dependency.capacity.clone());
        if let Some(default) = &dependency.default {
            value.metadata.insert("default".into(), default.clone());
            graph.edges.push(edge(
                &id,
                &provider_id(default),
                "default",
                Some(&dependency.capacity),
            ));
        }
        graph.nodes.push(value);
        graph.edges.push(edge(&flow_id, &id, "owns", None));
    }

    for binding in &flow.bindings {
        if let Some(dependency) = flow
            .dependencies
            .iter()
            .find(|dependency| dependency.name == binding.port)
        {
            graph.edges.push(edge(
                &format!("{flow_id}.input.{}", dependency.name),
                &provider_id(&binding.provider),
                "bind",
                Some(&dependency.capacity),
            ));
        }
    }

    for (index, statement) in flow.statements.iter().enumerate() {
        let (kind, name) = match statement {
            FlowStatement::Let { name, .. } => ("let", name.as_str()),
            FlowStatement::Require { .. } => ("require", "require"),
            FlowStatement::Call { name, .. } => ("call", name.as_str()),
            FlowStatement::Attempt { name, .. } => ("attempt", name.as_str()),
            FlowStatement::Make { name, .. } => ("make", name.as_str()),
            FlowStatement::Fold { name, .. } => ("fold", name.as_str()),
            FlowStatement::Run { name, .. } => ("run", name.as_str()),
            FlowStatement::Match { name, .. } => ("match", name.as_str()),
            FlowStatement::Map { name, .. } => ("map", name.as_str()),
            FlowStatement::Filter { name, .. } => ("filter", name.as_str()),
            FlowStatement::Sort { name, .. } => ("sort", name.as_str()),
            FlowStatement::Group { name, .. } => ("group", name.as_str()),
            FlowStatement::Parallel { name, .. } => ("parallel", name.as_str()),
            FlowStatement::Race { name, .. } => ("race", name.as_str()),
            FlowStatement::Emit { event, .. } => ("emit", event.as_str()),
            FlowStatement::Enqueue { job, .. } => ("enqueue", job.as_str()),
            FlowStatement::Return { .. } => ("return", "return"),
        };
        let id = format!("{flow_id}.{kind}.{index}");
        let mut value = node(&id, kind, name);
        value.metadata.insert("order".into(), index.to_string());
        match statement {
            FlowStatement::Let { expression, .. }
            | FlowStatement::Require { expression, .. }
            | FlowStatement::Return { expression, .. } => {
                value
                    .metadata
                    .insert("expression".into(), expression.clone());
            }
            FlowStatement::Emit {
                event, argument, ..
            } => {
                value.metadata.insert("event".into(), event.clone());
                value.metadata.insert("argument".into(), argument.clone());
            }
            FlowStatement::Enqueue { job, argument, .. } => {
                value.metadata.insert("job".into(), job.clone());
                value.metadata.insert("argument".into(), argument.clone());
            }
            FlowStatement::Call {
                dependency,
                operation,
                argument,
                propagate,
                ..
            } => {
                value
                    .metadata
                    .insert("dependency".into(), dependency.clone());
                value.metadata.insert("operation".into(), operation.clone());
                value.metadata.insert("argument".into(), argument.clone());
                value
                    .metadata
                    .insert("propagate".into(), propagate.to_string());
            }
            FlowStatement::Attempt {
                dependency,
                operation,
                argument,
                propagate,
                retry,
                timeout_ms,
                ..
            } => {
                value
                    .metadata
                    .insert("dependency".into(), dependency.clone());
                value.metadata.insert("operation".into(), operation.clone());
                value.metadata.insert("argument".into(), argument.clone());
                value
                    .metadata
                    .insert("propagate".into(), propagate.to_string());
                value.metadata.insert("retry".into(), retry.to_string());
                value
                    .metadata
                    .insert("timeout_ms".into(), timeout_ms.to_string());
            }
            FlowStatement::Make { type_name, .. } => {
                value.type_name = Some(type_name.clone());
            }
            FlowStatement::Fold {
                type_name,
                collection,
                initial,
                item,
                update,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("initial".into(), initial.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("update".into(), update.clone());
            }
            FlowStatement::Run {
                flow,
                argument,
                propagate,
                ..
            } => {
                value.metadata.insert("flow".into(), flow.clone());
                value.metadata.insert("argument".into(), argument.clone());
                value
                    .metadata
                    .insert("propagate".into(), propagate.to_string());
            }
            FlowStatement::Match {
                type_name, subject, ..
            } => {
                value.type_name = Some(type_name.clone());
                value.metadata.insert("subject".into(), subject.clone());
            }
            FlowStatement::Map {
                type_name,
                collection,
                item,
                expression,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value
                    .metadata
                    .insert("expression".into(), expression.clone());
            }
            FlowStatement::Filter {
                type_name,
                collection,
                item,
                predicate,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("predicate".into(), predicate.clone());
            }
            FlowStatement::Sort {
                type_name,
                collection,
                item,
                key,
                direction,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("key".into(), key.clone());
                value.metadata.insert("direction".into(), direction.clone());
            }
            FlowStatement::Group {
                type_name,
                collection,
                item,
                key,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("key".into(), key.clone());
            }
            FlowStatement::Parallel {
                type_name,
                collection,
                item,
                flow,
                argument,
                propagate,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("flow".into(), flow.clone());
                value.metadata.insert("argument".into(), argument.clone());
                value
                    .metadata
                    .insert("propagate".into(), propagate.to_string());
            }
            FlowStatement::Race {
                type_name,
                collection,
                item,
                flow,
                argument,
                propagate,
                ..
            } => {
                value.type_name = Some(type_name.clone());
                value
                    .metadata
                    .insert("collection".into(), collection.clone());
                value.metadata.insert("item".into(), item.clone());
                value.metadata.insert("flow".into(), flow.clone());
                value.metadata.insert("argument".into(), argument.clone());
                value
                    .metadata
                    .insert("propagate".into(), propagate.to_string());
            }
        }
        if let FlowStatement::Require { message, .. } = statement {
            value.metadata.insert("message".into(), message.clone());
        }
        graph.nodes.push(value);
        graph.edges.push(edge(&flow_id, &id, "owns", None));
        if let FlowStatement::Make { fields, .. } = statement {
            for field in fields {
                let assignment_id = format!("{id}.assign.{}", field.name);
                let mut assignment = node(&assignment_id, "assign", &field.name);
                assignment
                    .metadata
                    .insert("expression".into(), field.expression.clone());
                graph.nodes.push(assignment);
                graph.edges.push(edge(&id, &assignment_id, "owns", None));
            }
        }
        if let FlowStatement::Match { cases, .. } = statement {
            for case in cases {
                let case_id = format!("{id}.case.{}", case.variant);
                let mut case_node = node(&case_id, "case", &case.variant);
                case_node
                    .metadata
                    .insert("expression".into(), case.expression.clone());
                graph.nodes.push(case_node);
                graph.edges.push(edge(&id, &case_id, "owns", None));
            }
        }
    }
}

fn lower_api(api: &Api, graph: &mut GraphIr) {
    let api_id = format!("api.{}", api.name);
    graph.nodes.push(node(&api_id, "api", &api.name));
    for (index, middleware) in api.middlewares.iter().enumerate() {
        let id = format!("{api_id}.middleware.{index}");
        let mut value = node(&id, "middleware", &middleware.phase);
        value.type_name = Some(middleware.capacity.clone());
        value
            .metadata
            .insert("provider".into(), middleware.provider.clone());
        value.metadata.insert("order".into(), index.to_string());
        value
            .metadata
            .insert("phase".into(), middleware.phase.clone());
        graph.nodes.push(value);
        graph.edges.push(edge(&api_id, &id, "owns", None));
        graph.edges.push(edge(
            &id,
            &provider_id(&middleware.provider),
            "bind",
            Some(&middleware.capacity),
        ));
    }
    if let Some(auth) = &api.auth {
        let id = format!("{api_id}.auth.{}", auth.scheme);
        let mut value = node(&id, "auth", &auth.scheme);
        value.type_name = Some(auth.capacity.clone());
        value
            .metadata
            .insert("provider".into(), auth.provider.clone());
        graph.nodes.push(value);
        graph.edges.push(edge(&api_id, &id, "owns", None));
        graph.edges.push(edge(
            &id,
            &provider_id(&auth.provider),
            "bind",
            Some(&auth.capacity),
        ));
    }
    for (index, route) in api.routes.iter().enumerate() {
        let id = format!("{api_id}.route.{index}");
        let mut value = node(&id, "route", &format!("{} {}", route.method, route.path));
        value.type_name = Some(format!("{}->{}", route.input, route.output));
        value.metadata.insert("method".into(), route.method.clone());
        value.metadata.insert("path".into(), route.path.clone());
        value.metadata.insert("flow".into(), route.flow.clone());
        value
            .metadata
            .insert("input_source".into(), route.input_source.clone());
        if let Some(name) = &route.input_name {
            value.metadata.insert("input_name".into(), name.clone());
        }
        value.metadata.insert("order".into(), index.to_string());
        graph.nodes.push(value);
        graph.edges.push(edge(&api_id, &id, "owns", None));
        for (binding_index, binding) in route.bindings.iter().enumerate() {
            let binding_id = format!("{id}.request_binding.{binding_index}");
            let mut value = node(
                &binding_id,
                "request_binding",
                binding.target.as_deref().unwrap_or("$"),
            );
            value
                .metadata
                .insert("source".into(), binding.source.clone());
            if let Some(name) = &binding.name {
                value.metadata.insert("name".into(), name.clone());
            }
            value
                .metadata
                .insert("order".into(), binding_index.to_string());
            graph.nodes.push(value);
            graph.edges.push(edge(&id, &binding_id, "owns", None));
        }
        graph.edges.push(edge(
            &id,
            &format!("flow.{}", route.flow),
            "dispatch",
            Some(&format!("{}->{}", route.input, route.output)),
        ));
    }
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
