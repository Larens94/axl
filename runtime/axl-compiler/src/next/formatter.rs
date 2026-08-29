use super::ast::*;

pub fn format(program: &Program) -> String {
    let mut output = vec![
        format!("axl {}", program.version),
        format!("app {}", program.name),
    ];
    for import in &program.imports {
        output.push(String::new());
        output.push(format!("import \"{}\"", import.path));
    }
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
                        "  op {} {} -> {}{}",
                        operation.name,
                        operation.input,
                        operation.output,
                        if operation.idempotent {
                            " idempotent"
                        } else {
                            ""
                        }
                    ));
                }
            }
            Declaration::Skill(skill) => {
                output.push(format!("skill {} provides {}", skill.name, skill.provides));
                if let Some(native) = &skill.native {
                    output.push(format!("  native {} {}", native.target, native.symbol));
                }
                for config in &skill.configs {
                    let value = config
                        .secret_ref
                        .as_ref()
                        .map(|name| format!("secret(\"{name}\")"))
                        .unwrap_or_else(|| config.value.clone());
                    output.push(format!(
                        "  config {}: {} = {}",
                        config.name, config.type_name, value
                    ));
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
                        } => {
                            if let Some(items) = list_literal_items(expression) {
                                output.push(format!("  let {name} = ["));
                                for (index, item) in items.iter().enumerate() {
                                    let comma = if index + 1 == items.len() { "" } else { "," };
                                    output.push(format!("    {item}{comma}"));
                                }
                                output.push("  ]".into());
                            } else {
                                output.push(format!("  let {name} = {expression}"));
                            }
                        }
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
                        FlowStatement::Attempt {
                            name,
                            dependency,
                            operation,
                            argument,
                            propagate,
                            retry,
                            timeout_ms,
                            ..
                        } => {
                            output.push(format!(
                                "  attempt {name} = {dependency}.{operation}({argument}){}",
                                if *propagate { "?" } else { "" }
                            ));
                            output.push(format!("    retry = {retry}"));
                            output.push(format!("    timeout_ms = {timeout_ms}"));
                        }
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
                        FlowStatement::Sort {
                            name,
                            type_name,
                            collection,
                            item,
                            key,
                            direction,
                            ..
                        } => {
                            output.push(format!(
                                "  sort {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!("    by = {key}"));
                            output.push(format!("    direction = {direction}"));
                        }
                        FlowStatement::Group {
                            name,
                            type_name,
                            collection,
                            item,
                            key,
                            ..
                        } => {
                            output.push(format!(
                                "  group {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!("    by = {key}"));
                        }
                        FlowStatement::Parallel {
                            name,
                            type_name,
                            collection,
                            item,
                            flow,
                            argument,
                            propagate,
                            ..
                        } => {
                            output.push(format!(
                                "  parallel {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!(
                                "    run = {flow}({argument}){}",
                                if *propagate { "?" } else { "" }
                            ));
                        }
                        FlowStatement::Race {
                            name,
                            type_name,
                            collection,
                            item,
                            flow,
                            argument,
                            propagate,
                            ..
                        } => {
                            output.push(format!(
                                "  race {name}: {type_name} = {collection} as {item}"
                            ));
                            output.push(format!(
                                "    run = {flow}({argument}){}",
                                if *propagate { "?" } else { "" }
                            ));
                        }
                        FlowStatement::Emit {
                            event, argument, ..
                        } => output.push(format!("  emit {event}({argument})")),
                        FlowStatement::Enqueue { job, argument, .. } => {
                            output.push(format!("  enqueue {job}({argument})"))
                        }
                        FlowStatement::Return { expression, .. } => {
                            output.push(format!("  return {expression}"));
                        }
                    }
                }
            }
            Declaration::Event(event) => {
                output.push(format!("event {}: {}", event.name, event.payload));
            }
            Declaration::Subscription(subscription) => {
                output.push(format!(
                    "on {} {} = {}",
                    subscription.event, subscription.payload, subscription.flow
                ));
            }
            Declaration::Job(job) => {
                output.push(format!("job {}", job.name));
                if let Some(schedule) = &job.schedule {
                    output.push(format!("  schedule \"{schedule}\""));
                }
                output.push(format!("  run {}", job.flow));
                output.push(format!("  retry {}", job.retry));
                if job.idempotent {
                    output.push("  idempotent".into());
                }
                output.push(format!(
                    "  in store: {} = {}",
                    job.store_capacity, job.store_provider
                ));
            }
            Declaration::Api(api) => {
                output.push(format!("api {}", api.name));
                for middleware in &api.middlewares {
                    output.push(format!(
                        "  middleware {}: {} = {}",
                        middleware.phase, middleware.capacity, middleware.provider
                    ));
                }
                if let Some(auth) = &api.auth {
                    output.push(format!(
                        "  auth {}: {} = {}",
                        auth.scheme, auth.capacity, auth.provider
                    ));
                }
                for route in &api.routes {
                    let binding = route
                        .input_name
                        .as_ref()
                        .map(|name| format!(" from {}.{name}", route.input_source))
                        .unwrap_or_default();
                    output.push(format!(
                        "  {} {} {} -> {} = {}{}",
                        route.method, route.path, route.input, route.output, route.flow, binding
                    ));
                    if route.input_source == "composite" {
                        for binding in &route.bindings {
                            let target = binding.target.as_deref().unwrap_or_default();
                            let source = binding
                                .name
                                .as_ref()
                                .map(|name| format!("{}.{name}", binding.source))
                                .unwrap_or_else(|| binding.source.clone());
                            output.push(format!("    bind {target} = {source}"));
                        }
                    }
                    for guard in &route.guards {
                        let binding = guard
                            .name
                            .as_ref()
                            .map(|name| format!(" from {}.{name}", guard.source))
                            .unwrap_or_else(|| format!(" from {}", guard.source));
                        let param = guard
                            .param
                            .as_ref()
                            .map(|value| format!(" \"{value}\""))
                            .unwrap_or_default();
                        output.push(format!(
                            "    guard {} {}{}{}",
                            guard.kind, guard.flow, param, binding
                        ));
                    }
                }
            }
            Declaration::Ui(ui) => {
                output.push(format!("ui {}", ui.name));
                for page in &ui.pages {
                    let binding = page
                        .input_name
                        .as_deref()
                        .map(|name| format!(" from {}.{name}", page.input_source))
                        .unwrap_or_default();
                    output.push(format!(
                        "  page {} {} -> {} = {}{}",
                        page.path, page.input, page.output, page.flow, binding
                    ));
                    if page.input_source == "composite" {
                        for binding in &page.bindings {
                            let target = binding.target.as_deref().unwrap_or_default();
                            let source = binding
                                .name
                                .as_ref()
                                .map(|name| format!("{}.{name}", binding.source))
                                .unwrap_or_else(|| binding.source.clone());
                            let is_filter = page.filters.iter().any(|filter| {
                                filter.target == binding.target
                                    && filter.source == binding.source
                                    && filter.name == binding.name
                            });
                            let is_pagination = page.pagination.iter().any(|pagination| {
                                pagination.field == binding.target.as_deref().unwrap_or_default()
                            });
                            if is_filter {
                                output.push(format!("    filter {target} = {source}"));
                            } else if is_pagination {
                                let default = page
                                    .pagination
                                    .iter()
                                    .find(|pagination| pagination.field == target)
                                    .and_then(|pagination| pagination.default.as_deref())
                                    .map(|value| format!(" default {value}"))
                                    .unwrap_or_default();
                                output.push(format!("    pagination {target} = {source}{default}"));
                            } else {
                                output.push(format!("    bind {target} = {source}"));
                            }
                        }
                    }
                    for kpi in &page.kpis {
                        let hint = kpi
                            .hint
                            .as_ref()
                            .map(|value| format!(" \"{}\"", value.replace('"', "\\\"")))
                            .unwrap_or_default();
                        output.push(format!(
                            "    kpi {} \"{}\"{hint}",
                            kpi.field,
                            kpi.label.replace('"', "\\\"")
                        ));
                    }
                    for chart in &page.charts {
                        output.push(format!(
                            "    chart {} \"{}\"",
                            chart.field,
                            chart.label.replace('"', "\\\"")
                        ));
                    }
                }
                for slot in &ui.slots {
                    output.push(format!("  slot {} = {}", slot.name, slot.component));
                }
                for form in &ui.forms {
                    let submit = form
                        .submit
                        .as_deref()
                        .map(|path| format!(" submit {path}"))
                        .unwrap_or_default();
                    let redirect = form
                        .redirect
                        .as_deref()
                        .map(|path| format!(" redirect {path}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  form {} {} -> {} = {}{}{}",
                        form.path, form.entity, form.output, form.flow, submit, redirect
                    ));
                }
                for action in &ui.actions {
                    let on = action
                        .on
                        .as_deref()
                        .map(|path| format!(" on {path}"))
                        .unwrap_or_default();
                    let redirect = action
                        .redirect
                        .as_deref()
                        .map(|path| format!(" redirect {path}"))
                        .unwrap_or_default();
                    let clear_cookie = action
                        .clear_cookie
                        .as_deref()
                        .map(|name| format!(" clear_cookie {name}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  action {} {} {}{}{}{}",
                        action.path, action.method, action.submit, on, redirect, clear_cookie
                    ));
                }
                for drawer in &ui.drawers {
                    let binding = drawer
                        .input_name
                        .as_deref()
                        .map(|name| format!(" from {}.{name}", drawer.input_source))
                        .unwrap_or_default();
                    let on = drawer
                        .on
                        .as_ref()
                        .map(|path| format!(" on {path}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  drawer {} {} -> {} = {}{}{}",
                        drawer.path, drawer.input, drawer.output, drawer.flow, binding, on
                    ));
                }
                for modal in &ui.modals {
                    let binding = modal
                        .input_name
                        .as_deref()
                        .map(|name| format!(" from {}.{name}", modal.input_source))
                        .unwrap_or_default();
                    let on = modal
                        .on
                        .as_ref()
                        .map(|path| format!(" on {path}"))
                        .unwrap_or_default();
                    output.push(format!(
                        "  modal {} {} -> {} = {}{}{}",
                        modal.path, modal.input, modal.output, modal.flow, binding, on
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

fn list_literal_items(source: &str) -> Option<Vec<&str>> {
    let source = source.trim();
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut items = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in inner.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                let item = inner[start..index].trim();
                if item.is_empty() {
                    return None;
                }
                items.push(item);
                start = index + 1;
            }
            _ => {}
        }
    }
    let item = inner[start..].trim();
    if quoted || depth != 0 || item.is_empty() {
        return None;
    }
    items.push(item);
    Some(items)
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
