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
    for directory in [&rust_dir, &react_dir, &sql_dir, &agent_dir, &block_dir] {
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
    format!(
        "// Generated from AXL Semantic Graph IR. Do not edit.\n\nexport const axlSlots = {} as const;\n",
        serde_json::to_string_pretty(&slots).expect("slot registry is JSON serializable")
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
    json!({
        "schema": graph.schema,
        "app": graph.app,
        "protocol": "axl-open-block/1",
        "blocks": blocks,
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
            "blocks": "blocks/open-blocks.json"
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

fn sql_identifier(value: &str) -> String {
    rust_identifier(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::compile_source;

    const SOURCE: &str = r#"axl 4
app Demo
entity Customer
  id: uuid key
  email: email required unique
capacity CustomerStore
  op save Customer -> Result<Customer>
capacity CustomerRow
  op render Customer -> UI
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
skill DefaultRow provides CustomerRow
  native react crm::DefaultRow
blueprint CRM
  param page_size: int = 25
  state selected: Option<Customer>
  event customer.selected: Customer
  error load.failed: text
  in store: CustomerStore
  slot table.row: CustomerRow = DefaultRow
  use store = SqliteCustomers
agent Sales
  goal qualify
  plan automatic
"#;

    #[test]
    fn emits_target_contracts_from_one_graph() {
        let graph = compile_source(SOURCE).unwrap().graph;
        let rust = rust_contracts(&graph);
        let react = react_slots(&graph);
        let sql = sql_schema(&graph);
        let blocks = open_block_manifest(&graph);
        assert!(rust.contains("pub trait CustomerStore"));
        assert!(rust.contains("Result<Customer, String>"));
        assert!(react.contains("crm::DefaultRow"));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS customers"));
        assert!(sql.contains("email TEXT NOT NULL UNIQUE"));
        assert_eq!(blocks["protocol"], "axl-open-block/1");
        assert_eq!(blocks["blocks"][0]["name"], "CRM");
        assert_eq!(blocks["blocks"][0]["open_surface_count"], 3);
        assert!(
            blocks["blocks"][0]["surfaces"]
                .as_array()
                .is_some_and(|surfaces| surfaces.iter().any(|surface| {
                    surface["kind"] == "parameter" && surface["default"] == "25"
                }))
        );
    }
}
