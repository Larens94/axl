use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use super::ir::{GraphIr, GraphNode};

pub fn generate(graph: &GraphIr, output: &Path) -> Result<()> {
    let rust_dir = output.join("rust");
    let react_dir = output.join("react");
    let sql_dir = output.join("sql");
    let agent_dir = output.join("agents");
    let block_dir = output.join("blocks");
    let flow_dir = output.join("flows");
    let http_dir = output.join("http");
    for directory in [
        &rust_dir, &react_dir, &sql_dir, &agent_dir, &block_dir, &flow_dir, &http_dir,
    ] {
        std::fs::create_dir_all(directory)?;
    }
    std::fs::write(rust_dir.join("axl_contracts.rs"), rust_contracts(graph))?;
    std::fs::write(react_dir.join("axl_slots.ts"), react_slots(graph))?;
    std::fs::write(sql_dir.join("schema.sql"), sql_schema(graph))?;
    std::fs::write(
        agent_dir.join("agents.json"),
        serde_json::to_string_pretty(&agent_manifest(graph))?,
    )?;
    std::fs::write(
        block_dir.join("open-blocks.json"),
        serde_json::to_string_pretty(&open_block_manifest(graph))?,
    )?;
    std::fs::write(
        flow_dir.join("flows.json"),
        serde_json::to_string_pretty(&flow_manifest(graph))?,
    )?;
    std::fs::write(
        http_dir.join("routes.json"),
        serde_json::to_string_pretty(&http_manifest(graph))?,
    )?;
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_string_pretty(&target_manifest(graph))?,
    )?;
    Ok(())
}

pub fn rust_contracts(graph: &GraphIr) -> String {
    let mut output = vec![
        "// Generated from AXL Semantic Graph IR. Do not edit.".to_string(),
        "#![allow(dead_code)]".to_string(),
        String::new(),
        "#[derive(Debug, Clone)]".to_string(),
        "pub struct NativeBinding {".to_string(),
        "    pub skill: &'static str,".to_string(),
        "    pub capacity: &'static str,".to_string(),
        "    pub target: &'static str,".to_string(),
        "}".to_string(),
        String::new(),
    ];

    for value in nodes(graph, "enum") {
        output.push("#[derive(Debug, Clone, PartialEq, Eq)]".into());
        output.push(format!("pub enum {} {{", value.name));
        for variant in children(graph, &value.id, "variant") {
            output.push(format!("    {},", rust_variant(&variant.name)));
        }
        output.push("}".into());
        output.push(String::new());
    }

    for entity in nodes(graph, "entity") {
        output.push("#[derive(Debug, Clone)]".into());
        output.push(format!("pub struct {} {{", entity.name));
        for field in children(graph, &entity.id, "field") {
            output.push(format!(
                "    pub {}: {},",
                rust_identifier(&field.name),
                rust_type(field.type_name.as_deref().unwrap_or("unit"))
            ));
        }
        output.push("}".into());
        output.push(String::new());
    }

    for capacity in nodes(graph, "capacity") {
        output.push(format!("pub trait {} {{", capacity.name));
        for operation in children(graph, &capacity.id, "operation") {
            let signature = operation.type_name.as_deref().unwrap_or("unit->unit");
            let (input, result) = signature.split_once("->").unwrap_or(("unit", "unit"));
            output.push(format!(
                "    fn {}(&self, input: {}) -> {};",
                rust_identifier(&operation.name),
                rust_type(input),
                rust_type(result)
            ));
        }
        output.push("}".into());
        output.push(String::new());
    }

    output.push("pub const AXL_NATIVE_BINDINGS: &[NativeBinding] = &[".into());
    for skill in nodes(graph, "skill") {
        if let Some(implementation) = &skill.implementation {
            output.push(format!(
                "    NativeBinding {{ skill: {:?}, capacity: {:?}, target: {:?} }},",
                skill.name,
                skill.type_name.as_deref().unwrap_or(""),
                implementation
            ));
        }
    }
    output.push("];".into());
    output.push(String::new());
    output.join("\n")
}

pub fn react_slots(graph: &GraphIr) -> String {
    let mut slots = Vec::new();
    for node in graph
        .nodes
        .iter()
        .filter(|node| matches!(node.kind.as_str(), "slot" | "hook"))
    {
        let owner = parent(graph, &node.id)
            .map(|value| value.name.clone())
            .unwrap_or_default();
        let provider = provider_for(graph, &node.id);
        let implementation = provider
            .as_ref()
            .and_then(|provider| graph.nodes.iter().find(|node| node.id == *provider))
            .and_then(|node| node.implementation.clone());
        slots.push(json!({
            "id": format!("{}.{}", owner, node.name),
            "kind": node.kind,
            "interface": node.type_name,
            "provider": provider.and_then(|id| graph.nodes.iter().find(|node| node.id == id).map(|node| node.name.clone())),
            "implementation": implementation,
        }));
    }
    let instances = open_block_manifest(graph)["instances"].clone();
    format!(
        "// Generated from AXL Semantic Graph IR. Do not edit.\n\nexport const axlSlots = {} as const;\n\nexport const axlInstances = {} as const;\n",
        serde_json::to_string_pretty(&slots).expect("slot registry is JSON serializable"),
        serde_json::to_string_pretty(&instances).expect("instance registry is JSON serializable")
    )
}

pub fn open_block_manifest(graph: &GraphIr) -> serde_json::Value {
    let provider_kinds = ["input", "slot", "hook", "action", "policy"];
    let blocks = nodes(graph, "blueprint")
        .into_iter()
        .map(|blueprint| {
            let surfaces = graph
                .nodes
                .iter()
                .filter(|node| {
                    parent(graph, &node.id).is_some_and(|owner| owner.id == blueprint.id)
                })
                .map(|surface| {
                    let provider = provider_for(graph, &surface.id);
                    let provider_name = provider.as_ref().and_then(|id| {
                        graph
                            .nodes
                            .iter()
                            .find(|node| node.id == *id)
                            .map(|node| node.name.clone())
                    });
                    json!({
                        "name": surface.name,
                        "kind": surface.kind,
                        "type": surface.type_name,
                        "default": surface.metadata.get("default"),
                        "provider": provider_name,
                        "accepts_provider": provider_kinds.contains(&surface.kind.as_str()),
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "name": blueprint.name,
                "open_surface_count": surfaces.iter().filter(|surface| {
                    matches!(
                        surface.get("kind").and_then(serde_json::Value::as_str),
                        Some("input" | "slot" | "hook" | "parameter" | "action" | "policy")
                    )
                }).count(),
                "surfaces": surfaces,
                "contracts": graph.contracts.iter()
                    .filter(|contract| contract.owner == blueprint.id)
                    .collect::<Vec<_>>(),
                "effects": grants_for(&graph.effects, &blueprint.id),
                "capabilities": grants_for(&graph.capabilities, &blueprint.id),
            })
        })
        .collect::<Vec<_>>();
    let instances = nodes(graph, "instance")
        .into_iter()
        .map(|instance| {
            let settings = children(graph, &instance.id, "setting")
                .into_iter()
                .map(|setting| {
                    json!({
                        "name": setting.name,
                        "type": setting.type_name,
                        "value": setting.metadata.get("value"),
                    })
                })
                .collect::<Vec<_>>();
            let overrides = children(graph, &instance.id, "override")
                .into_iter()
                .map(|surface| {
                    let provider = provider_for(graph, &surface.id).and_then(|id| {
                        graph
                            .nodes
                            .iter()
                            .find(|node| node.id == id)
                            .map(|node| node.name.clone())
                    });
                    json!({
                        "name": surface.name,
                        "type": surface.type_name,
                        "provider": provider,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "name": instance.name,
                "blueprint": instance.type_name,
                "settings": settings,
                "overrides": overrides,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "protocol": "axl-open-block/2",
        "blocks": blocks,
        "instances": instances,
    })
}

pub fn flow_manifest(graph: &GraphIr) -> serde_json::Value {
    let flows = nodes(graph, "flow")
        .into_iter()
        .map(|flow| {
            let (input, output) = flow
                .type_name
                .as_deref()
                .and_then(|value| value.split_once("->"))
                .unwrap_or(("", ""));
            let mut statements = children(graph, &flow.id, "let")
                .into_iter()
                .chain(children(graph, &flow.id, "require"))
                .chain(children(graph, &flow.id, "call"))
                .chain(children(graph, &flow.id, "make"))
                .chain(children(graph, &flow.id, "fold"))
                .chain(children(graph, &flow.id, "run"))
                .chain(children(graph, &flow.id, "match"))
                .chain(children(graph, &flow.id, "map"))
                .chain(children(graph, &flow.id, "filter"))
                .chain(children(graph, &flow.id, "return"))
                .collect::<Vec<_>>();
            statements.sort_by_key(|statement| {
                statement
                    .metadata
                    .get("order")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(usize::MAX)
            });
            let dependencies = children(graph, &flow.id, "input")
                .into_iter()
                .map(|dependency| {
                    let provider = provider_for(graph, &dependency.id).and_then(|id| {
                        graph
                            .nodes
                            .iter()
                            .find(|node| node.id == id)
                            .map(|node| node.name.clone())
                    });
                    json!({
                        "name": dependency.name,
                        "capacity": dependency.type_name,
                        "provider": provider,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "name": flow.name,
                "input": input,
                "output": output,
                "dependencies": dependencies,
                "statements": statements.into_iter().map(|statement| {
                    let fields = children(graph, &statement.id, "assign")
                        .into_iter()
                        .map(|field| json!({
                            "name": field.name,
                            "expression": field.metadata.get("expression"),
                        }))
                        .collect::<Vec<_>>();
                    let cases = children(graph, &statement.id, "case")
                        .into_iter()
                        .map(|case| json!({
                            "variant": case.name,
                            "expression": case.metadata.get("expression"),
                        }))
                        .collect::<Vec<_>>();
                    json!({
                        "kind": statement.kind,
                        "name": statement.name,
                        "expression": statement.metadata.get("expression"),
                        "message": statement.metadata.get("message"),
                        "dependency": statement.metadata.get("dependency"),
                        "operation": statement.metadata.get("operation"),
                        "argument": statement.metadata.get("argument"),
                        "propagate": statement.metadata.get("propagate")
                            .and_then(|value| value.parse::<bool>().ok()),
                        "type": statement.type_name,
                        "fields": fields,
                        "collection": statement.metadata.get("collection"),
                        "initial": statement.metadata.get("initial"),
                        "item": statement.metadata.get("item"),
                        "update": statement.metadata.get("update"),
                        "flow": statement.metadata.get("flow"),
                        "subject": statement.metadata.get("subject"),
                        "cases": cases,
                        "predicate": statement.metadata.get("predicate"),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "runtime": "axl-flow/2",
        "flows": flows,
    })
}

pub fn http_manifest(graph: &GraphIr) -> serde_json::Value {
    let apis = nodes(graph, "api")
        .into_iter()
        .map(|api| {
            let mut routes = children(graph, &api.id, "route");
            routes.sort_by_key(|route| {
                route
                    .metadata
                    .get("order")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(usize::MAX)
            });
            json!({
                "name": api.name,
                "routes": routes.into_iter().map(|route| {
                    let (input, output) = route.type_name.as_deref()
                        .and_then(|value| value.split_once("->"))
                        .unwrap_or(("", ""));
                    json!({
                        "method": route.metadata.get("method"),
                        "path": route.metadata.get("path"),
                        "input": input,
                        "output": output,
                        "flow": route.metadata.get("flow"),
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "protocol": "axl-http/1",
        "apis": apis,
    })
}

pub fn sql_schema(graph: &GraphIr) -> String {
    let mut output = vec!["-- Generated from AXL Semantic Graph IR. Do not edit.".to_string()];
    for entity in nodes(graph, "entity") {
        output.push(String::new());
        output.push(format!(
            "CREATE TABLE IF NOT EXISTS {} (",
            table_name(&entity.name)
        ));
        let fields = children(graph, &entity.id, "field");
        for (index, field) in fields.iter().enumerate() {
            let qualifiers = field
                .metadata
                .get("qualifiers")
                .map(|value| value.split(',').collect::<Vec<_>>())
                .unwrap_or_default();
            let mut line = format!(
                "  {} {}",
                sql_identifier(&field.name),
                sql_type(field.type_name.as_deref().unwrap_or("text"))
            );
            if qualifiers.contains(&"key") {
                line.push_str(" PRIMARY KEY");
            }
            if qualifiers.contains(&"required") {
                line.push_str(" NOT NULL");
            }
            if qualifiers.contains(&"unique") {
                line.push_str(" UNIQUE");
            }
            if index + 1 != fields.len() {
                line.push(',');
            }
            output.push(line);
        }
        output.push(");".into());
    }
    output.push(String::new());
    output.join("\n")
}

fn target_manifest(graph: &GraphIr) -> serde_json::Value {
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "targets": {
            "rust": "rust/axl_contracts.rs",
            "react": "react/axl_slots.ts",
            "sql": "sql/schema.sql",
            "agents": "agents/agents.json",
            "blocks": "blocks/open-blocks.json",
            "flows": "flows/flows.json",
            "http": "http/routes.json"
        },
        "counts": {
            "nodes": graph.nodes.len(),
            "edges": graph.edges.len(),
            "contracts": graph.contracts.len(),
            "effects": graph.effects.len(),
            "capabilities": graph.capabilities.len()
        }
    })
}

fn agent_manifest(graph: &GraphIr) -> serde_json::Value {
    let agents = nodes(graph, "agent")
        .into_iter()
        .map(|agent| {
            json!({
                "name": agent.name,
                "beliefs": child_names(graph, &agent.id, "belief"),
                "goals": child_names(graph, &agent.id, "goal"),
                "plans": child_names(graph, &agent.id, "plan"),
                "effects": grants_for(&graph.effects, &agent.id),
                "capabilities": grants_for(&graph.capabilities, &agent.id)
            })
        })
        .collect::<Vec<_>>();
    json!({ "agents": agents })
}

fn nodes<'a>(graph: &'a GraphIr, kind: &str) -> Vec<&'a GraphNode> {
    graph
        .nodes
        .iter()
        .filter(|node| node.kind == kind)
        .collect()
}

fn children<'a>(graph: &'a GraphIr, owner: &str, kind: &str) -> Vec<&'a GraphNode> {
    let by_id = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns" && edge.from == owner)
        .filter_map(|edge| by_id.get(edge.to.as_str()).copied())
        .filter(|node| node.kind == kind)
        .collect()
}

fn child_names(graph: &GraphIr, owner: &str, kind: &str) -> Vec<String> {
    children(graph, owner, kind)
        .into_iter()
        .map(|node| node.name.clone())
        .collect()
}

fn parent<'a>(graph: &'a GraphIr, child: &str) -> Option<&'a GraphNode> {
    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.kind == "owns" && edge.to == child)?;
    graph.nodes.iter().find(|node| node.id == edge.from)
}

fn provider_for(graph: &GraphIr, port: &str) -> Option<String> {
    graph
        .edges
        .iter()
        .find(|edge| edge.from == port && matches!(edge.kind.as_str(), "bind" | "default"))
        .map(|edge| edge.to.clone())
}

fn grants_for(grants: &[super::ir::GraphGrant], owner: &str) -> Vec<String> {
    grants
        .iter()
        .filter(|grant| grant.owner == owner)
        .map(|grant| grant.name.clone())
        .collect()
}

fn rust_type(value: &str) -> String {
    if let Some(inner) = generic(value, "Result") {
        return format!("Result<{}, String>", rust_type(inner));
    }
    if let Some(inner) = generic(value, "Option") {
        return format!("Option<{}>", rust_type(inner));
    }
    if let Some(inner) = generic(value, "List") {
        return format!("Vec<{}>", rust_type(inner));
    }
    match value {
        "unit" => "()".into(),
        "bool" => "bool".into(),
        "int" => "i64".into(),
        "float" => "f64".into(),
        "money" => "i64".into(),
        "text" | "string" | "email" | "uuid" | "datetime" | "duration" => "String".into(),
        "bytes" => "Vec<u8>".into(),
        "UI" => "String".into(),
        other => other.to_string(),
    }
}

fn sql_type(value: &str) -> &'static str {
    match value {
        "bool" => "BOOLEAN",
        "int" | "money" => "INTEGER",
        "float" => "REAL",
        "bytes" => "BLOB",
        _ => "TEXT",
    }
}

fn generic<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value
        .strip_prefix(name)?
        .strip_prefix('<')?
        .strip_suffix('>')
}

fn table_name(value: &str) -> String {
    format!("{}s", snake_case(value))
}

fn snake_case(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    output
}

fn rust_identifier(value: &str) -> String {
    value.replace(['-', '.'], "_")
}

fn rust_variant(value: &str) -> String {
    value
        .split(['-', '_', '.'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut characters = segment.chars();
            match characters.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn sql_identifier(value: &str) -> String {
    rust_identifier(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::compile_source;

    const SOURCE: &str = r#"axl 4
app Demo
enum CustomerStatus
  active
  inactive
entity Customer
  id: uuid key
  email: email required unique
  status: CustomerStatus required
capacity CustomerStore
  op save Customer -> Result<Customer>
capacity CustomerRow
  op render Customer -> UI
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
skill DefaultRow provides CustomerRow
  native react crm::DefaultRow
skill CompactRow provides CustomerRow
  native react crm::CompactRow
blueprint CRM
  param page_size: int = 25
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text
  in store: CustomerStore
  slot table.row: CustomerRow = DefaultRow
  use store = SqliteCustomers
instance CompactCRM of CRM
  set page_size = 10
  use table.row = CompactRow
agent Sales
  goal qualify
  plan automatic
flow Identity Customer -> Customer
  return input
flow Save Customer -> Result<Customer>
  in store: CustomerStore = SqliteCustomers
  call saved = store.save(input)?
  return saved
api DemoApi
  post /customers Customer -> Customer = Identity
"#;

    #[test]
    fn emits_target_contracts_from_one_graph() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let rust = rust_contracts(&graph);
        let react = react_slots(&graph);
        let sql = sql_schema(&graph);
        let blocks = open_block_manifest(&graph);
        let flows = flow_manifest(&graph);
        let http = http_manifest(&graph);
        assert!(rust.contains("pub enum CustomerStatus"));
        assert!(rust.contains("Active,"));
        assert!(rust.contains("pub trait CustomerStore"));
        assert!(rust.contains("Result<Customer, String>"));
        assert!(react.contains("crm::DefaultRow"));
        assert!(react.contains("axlInstances"));
        assert!(react.contains("CompactRow"));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS customers"));
        assert!(sql.contains("email TEXT NOT NULL UNIQUE"));
        assert_eq!(blocks["protocol"], "axl-open-block/2");
        assert_eq!(blocks["blocks"][0]["name"], "CRM");
        assert_eq!(blocks["blocks"][0]["open_surface_count"], 3);
        assert!(
            blocks["blocks"][0]["surfaces"]
                .as_array()
                .is_some_and(|surfaces| surfaces.iter().any(|surface| {
                    surface["kind"] == "parameter" && surface["default"] == "25"
                }))
        );
        assert_eq!(blocks["instances"][0]["name"], "CompactCRM");
        assert_eq!(blocks["instances"][0]["blueprint"], "CRM");
        assert_eq!(blocks["instances"][0]["settings"][0]["value"], "10");
        assert_eq!(
            blocks["instances"][0]["overrides"][0]["provider"],
            "CompactRow"
        );
        assert_eq!(flows["runtime"], "axl-flow/2");
        assert_eq!(flows["flows"][0]["name"], "Identity");
        assert_eq!(flows["flows"][0]["statements"][0]["kind"], "return");
        let save = flows["flows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|flow| flow["name"] == "Save")
            .unwrap();
        assert_eq!(save["dependencies"][0]["provider"], "SqliteCustomers");
        assert_eq!(save["statements"][0]["kind"], "call");
        assert_eq!(save["statements"][0]["operation"], "save");
        assert_eq!(save["statements"][0]["propagate"], true);
        assert_eq!(http["protocol"], "axl-http/1");
        assert_eq!(http["apis"][0]["routes"][0]["path"], "/customers");
        assert_eq!(http["apis"][0]["routes"][0]["flow"], "Identity");
    }
}
