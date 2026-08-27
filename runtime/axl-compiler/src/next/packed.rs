use std::collections::BTreeMap;
use std::fmt;

use super::ir::{GraphContract, GraphEdge, GraphGrant, GraphIr, GraphNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedError(pub String);

impl fmt::Display for PackedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for PackedError {}

pub fn encode(graph: &GraphIr) -> Result<String, PackedError> {
    let mut graph = graph.clone();
    graph.canonicalize();
    let node_indexes = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let parents = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == "owns")
        .map(|edge| (edge.to.as_str(), edge.from.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut frames = vec!["4".to_string(), format!("1|{}", field(&graph.app))];
    for (index, node) in graph.nodes.iter().enumerate() {
        let metadata = serde_json::to_string(&node.metadata)
            .map_err(|error| PackedError(format!("cannot encode node metadata: {error}")))?;
        frames.push(format!(
            "10|{}|{}|{}|{}|{}|{}|{}",
            index,
            node_kind_code(&node.kind),
            field(&node.name),
            field(node.type_name.as_deref().unwrap_or("")),
            field(node.implementation.as_deref().unwrap_or("")),
            if node.metadata.is_empty() {
                "~"
            } else {
                &metadata
            },
            match parents.get(node.id.as_str()) {
                Some(parent) =>
                    node_indexes
                        .get(parent)
                        .map(usize::to_string)
                        .ok_or_else(|| {
                            PackedError(format!(
                                "node '{}' has missing parent '{}'",
                                node.id, parent
                            ))
                        })?,
                None => "~".into(),
            },
        ));
    }
    for edge in &graph.edges {
        if edge.kind == "owns" {
            continue;
        }
        let from = index_of(&node_indexes, &edge.from)?;
        let to = index_of(&node_indexes, &edge.to)?;
        frames.push(format!(
            "11|{}|{}|{}|{}",
            from,
            to,
            edge_kind_code(&edge.kind),
            field(edge.interface.as_deref().unwrap_or("")),
        ));
    }
    for contract in &graph.contracts {
        frames.push(format!(
            "20|{}|{}|{}",
            index_of(&node_indexes, &contract.owner)?,
            contract_kind_code(&contract.kind),
            field(&contract.expression),
        ));
    }
    for effect in &graph.effects {
        frames.push(format!(
            "21|{}|{}",
            index_of(&node_indexes, &effect.owner)?,
            field(&effect.name),
        ));
    }
    for capability in &graph.capabilities {
        frames.push(format!(
            "22|{}|{}",
            index_of(&node_indexes, &capability.owner)?,
            field(&capability.name),
        ));
    }
    Ok(frames.join(";"))
}

pub fn decode(source: &str) -> Result<GraphIr, PackedError> {
    let frames = split(source, ';')?
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if frames.first().map(String::as_str) != Some("4") {
        return Err(PackedError(
            "packed Graph IR requires version header '4'".into(),
        ));
    }
    let app_frame = frames
        .get(1)
        .ok_or_else(|| PackedError("packed Graph IR is missing app frame 1".into()))?;
    let app_fields = split(app_frame, '|')?;
    if app_fields.len() != 2 || app_fields[0] != "1" {
        return Err(PackedError(
            "packed Graph IR expects app frame '1|name'".into(),
        ));
    }
    let mut graph = GraphIr {
        schema: "ax-ir/4.0".into(),
        app: parse_field(&app_fields[1])?,
        nodes: Vec::new(),
        edges: Vec::new(),
        contracts: Vec::new(),
        effects: Vec::new(),
        capabilities: Vec::new(),
    };

    for (index, frame) in frames.iter().enumerate().skip(2) {
        let fields = split(frame, '|')?;
        match fields.first().map(String::as_str) {
            Some("10") if fields.len() == 8 => {
                let expected_index = graph.nodes.len();
                let found_index = fields[1].parse::<usize>().map_err(|_| {
                    PackedError(format!("frame {index}: invalid node index '{}'", fields[1]))
                })?;
                if found_index != expected_index {
                    return Err(PackedError(format!(
                        "frame {index}: expected node index {expected_index}, found {found_index}"
                    )));
                }
                let metadata: BTreeMap<String, String> = if fields[6] == "~" {
                    BTreeMap::new()
                } else {
                    serde_json::from_str(&fields[6]).map_err(|error| {
                        PackedError(format!("frame {index}: invalid node metadata: {error}"))
                    })?
                };
                let kind = node_kind_from_code(&fields[2])?;
                let name = parse_field(&fields[3])?;
                let parent = if fields[7] == "~" {
                    None
                } else {
                    let parent_index = fields[7].parse::<usize>().map_err(|_| {
                        PackedError(format!(
                            "frame {index}: invalid parent reference '{}'",
                            fields[7]
                        ))
                    })?;
                    Some(
                        graph
                            .nodes
                            .get(parent_index)
                            .ok_or_else(|| {
                                PackedError(format!(
                                    "frame {index}: parent {parent_index} must precede its child"
                                ))
                            })?
                            .id
                            .clone(),
                    )
                };
                let id = reconstruct_id(&kind, &name, parent.as_deref(), &metadata)?;
                graph.nodes.push(GraphNode {
                    id: id.clone(),
                    kind,
                    name,
                    type_name: nonempty(parse_field(&fields[4])?),
                    implementation: nonempty(parse_field(&fields[5])?),
                    metadata,
                });
                if let Some(parent) = parent {
                    graph.edges.push(GraphEdge {
                        from: parent,
                        to: id,
                        kind: "owns".into(),
                        interface: None,
                    });
                }
            }
            Some("11") if fields.len() == 5 => {
                let from = node_id(&graph.nodes, &fields[1], index)?;
                let to = node_id(&graph.nodes, &fields[2], index)?;
                graph.edges.push(GraphEdge {
                    from,
                    to,
                    kind: edge_kind_from_code(&fields[3])?,
                    interface: nonempty(parse_field(&fields[4])?),
                });
            }
            Some("20") if fields.len() == 4 => graph.contracts.push(GraphContract {
                owner: node_id(&graph.nodes, &fields[1], index)?,
                kind: contract_kind_from_code(&fields[2])?,
                expression: parse_field(&fields[3])?,
            }),
            Some("21") if fields.len() == 3 => graph.effects.push(GraphGrant {
                owner: node_id(&graph.nodes, &fields[1], index)?,
                name: parse_field(&fields[2])?,
            }),
            Some("22") if fields.len() == 3 => graph.capabilities.push(GraphGrant {
                owner: node_id(&graph.nodes, &fields[1], index)?,
                name: parse_field(&fields[2])?,
            }),
            Some(opcode) => {
                return Err(PackedError(format!(
                    "frame {index}: invalid opcode or arity '{opcode}'"
                )));
            }
            None => return Err(PackedError(format!("frame {index}: empty frame"))),
        }
    }
    graph.canonicalize();
    Ok(graph)
}

pub fn matrix(source: &str, max_width: usize) -> Result<String, PackedError> {
    if max_width < 32 {
        return Err(PackedError("matrix width must be at least 32".into()));
    }
    let frames = split(source, ';')?;
    let mut lines = Vec::new();
    let mut current = String::new();
    for frame in frames.into_iter().filter(|frame| !frame.trim().is_empty()) {
        let rendered = format!("{};", frame.trim());
        if current.is_empty() {
            current.push_str(&rendered);
        } else if current.chars().count() + rendered.chars().count() < max_width {
            current.push(' ');
            current.push_str(&rendered);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(&rendered);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    Ok(lines.join("\n"))
}

fn field(value: &str) -> String {
    if value.is_empty() {
        return "~".into();
    }
    if value
        .chars()
        .all(|character| !character.is_whitespace() && !matches!(character, ';' | '|' | '"'))
    {
        value.into()
    } else {
        serde_json::to_string(value).expect("Rust strings are JSON encodable")
    }
}

fn parse_field(value: &str) -> Result<String, PackedError> {
    if value == "~" {
        return Ok(String::new());
    }
    if value.starts_with('"') {
        serde_json::from_str(value)
            .map_err(|error| PackedError(format!("invalid packed string {value}: {error}")))
    } else {
        Ok(value.to_string())
    }
}

fn nonempty(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn index_of(indexes: &BTreeMap<&str, usize>, id: &str) -> Result<usize, PackedError> {
    indexes
        .get(id)
        .copied()
        .ok_or_else(|| PackedError(format!("Graph IR references missing node '{id}'")))
}

fn node_id(nodes: &[GraphNode], value: &str, frame: usize) -> Result<String, PackedError> {
    let index = value
        .parse::<usize>()
        .map_err(|_| PackedError(format!("frame {frame}: invalid node reference '{value}'")))?;
    nodes
        .get(index)
        .map(|node| node.id.clone())
        .ok_or_else(|| PackedError(format!("frame {frame}: unknown node reference {index}")))
}

fn reconstruct_id(
    kind: &str,
    name: &str,
    parent: Option<&str>,
    metadata: &BTreeMap<String, String>,
) -> Result<String, PackedError> {
    match parent {
        None if matches!(
            kind,
            "app"
                | "enum"
                | "entity"
                | "capacity"
                | "skill"
                | "blueprint"
                | "instance"
                | "flow"
                | "api"
                | "agent"
        ) =>
        {
            Ok(format!("{kind}.{name}"))
        }
        Some(parent) if kind == "operation" => Ok(format!("{parent}.op.{name}")),
        Some(parent) if kind == "variant" => Ok(format!("{parent}.variant.{name}")),
        Some(parent) if matches!(kind, "belief" | "goal" | "plan") => {
            let order = metadata.get("order").ok_or_else(|| {
                PackedError(format!("{kind} node '{name}' is missing order metadata"))
            })?;
            Ok(format!("{parent}.{kind}.{order}"))
        }
        Some(parent)
            if matches!(
                kind,
                "let"
                    | "require"
                    | "call"
                    | "make"
                    | "fold"
                    | "run"
                    | "match"
                    | "map"
                    | "filter"
                    | "sort"
                    | "group"
                    | "parallel"
                    | "return"
            ) =>
        {
            let order = metadata.get("order").ok_or_else(|| {
                PackedError(format!("{kind} node '{name}' is missing order metadata"))
            })?;
            Ok(format!("{parent}.{kind}.{order}"))
        }
        Some(parent) if kind == "assign" => Ok(format!("{parent}.assign.{name}")),
        Some(parent) if kind == "case" => Ok(format!("{parent}.case.{name}")),
        Some(parent) if kind == "route" => {
            let order = metadata.get("order").ok_or_else(|| {
                PackedError(format!("route node '{name}' is missing order metadata"))
            })?;
            Ok(format!("{parent}.route.{order}"))
        }
        Some(parent)
            if matches!(
                kind,
                "field"
                    | "input"
                    | "output"
                    | "slot"
                    | "hook"
                    | "parameter"
                    | "state"
                    | "event"
                    | "action"
                    | "error"
                    | "policy"
                    | "setting"
                    | "override"
            ) =>
        {
            Ok(format!("{parent}.{kind}.{name}"))
        }
        None => Err(PackedError(format!("node kind '{kind}' requires a parent"))),
        Some(_) => Err(PackedError(format!(
            "node kind '{kind}' cannot be reconstructed"
        ))),
    }
}

fn node_kind_code(kind: &str) -> &str {
    match kind {
        "app" => "0",
        "entity" => "1",
        "field" => "2",
        "capacity" => "3",
        "operation" => "4",
        "skill" => "5",
        "blueprint" => "6",
        "input" => "7",
        "output" => "8",
        "slot" => "9",
        "hook" => "10",
        "agent" => "11",
        "belief" => "12",
        "goal" => "13",
        "plan" => "14",
        "parameter" => "15",
        "state" => "16",
        "event" => "17",
        "action" => "18",
        "error" => "19",
        "policy" => "20",
        "instance" => "21",
        "setting" => "22",
        "override" => "23",
        "enum" => "24",
        "variant" => "25",
        "flow" => "26",
        "let" => "27",
        "require" => "28",
        "return" => "29",
        "call" => "30",
        "make" => "31",
        "assign" => "32",
        "fold" => "33",
        "run" => "34",
        "match" => "35",
        "case" => "36",
        "map" => "37",
        "filter" => "38",
        "api" => "39",
        "route" => "40",
        "sort" => "41",
        "group" => "42",
        "parallel" => "43",
        other => other,
    }
}

fn node_kind_from_code(code: &str) -> Result<String, PackedError> {
    Ok(match code {
        "0" => "app",
        "1" => "entity",
        "2" => "field",
        "3" => "capacity",
        "4" => "operation",
        "5" => "skill",
        "6" => "blueprint",
        "7" => "input",
        "8" => "output",
        "9" => "slot",
        "10" => "hook",
        "11" => "agent",
        "12" => "belief",
        "13" => "goal",
        "14" => "plan",
        "15" => "parameter",
        "16" => "state",
        "17" => "event",
        "18" => "action",
        "19" => "error",
        "20" => "policy",
        "21" => "instance",
        "22" => "setting",
        "23" => "override",
        "24" => "enum",
        "25" => "variant",
        "26" => "flow",
        "27" => "let",
        "28" => "require",
        "29" => "return",
        "30" => "call",
        "31" => "make",
        "32" => "assign",
        "33" => "fold",
        "34" => "run",
        "35" => "match",
        "36" => "case",
        "37" => "map",
        "38" => "filter",
        "39" => "api",
        "40" => "route",
        "41" => "sort",
        "42" => "group",
        "43" => "parallel",
        _ => return Err(PackedError(format!("unknown node kind code '{code}'"))),
    }
    .into())
}

fn edge_kind_code(kind: &str) -> &str {
    match kind {
        "owns" => "0",
        "provides" => "1",
        "bind" => "2",
        "default" => "3",
        "instantiates" => "4",
        "dispatch" => "5",
        other => other,
    }
}

fn edge_kind_from_code(code: &str) -> Result<String, PackedError> {
    Ok(match code {
        "0" => "owns",
        "1" => "provides",
        "2" => "bind",
        "3" => "default",
        "4" => "instantiates",
        "5" => "dispatch",
        _ => return Err(PackedError(format!("unknown edge kind code '{code}'"))),
    }
    .into())
}

fn contract_kind_code(kind: &str) -> &str {
    match kind {
        "requires" => "0",
        "ensures" => "1",
        "invariant" => "2",
        other => other,
    }
}

fn contract_kind_from_code(code: &str) -> Result<String, PackedError> {
    Ok(match code {
        "0" => "requires",
        "1" => "ensures",
        "2" => "invariant",
        _ => return Err(PackedError(format!("unknown contract kind code '{code}'"))),
    }
    .into())
}

fn split(source: &str, delimiter: char) -> Result<Vec<String>, PackedError> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in source.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if quoted && character == '\\' {
            current.push(character);
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            current.push(character);
            continue;
        }
        if character == delimiter && !quoted {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(character);
        }
    }
    if quoted || escaped {
        return Err(PackedError("unterminated quoted field".into()));
    }
    fields.push(current);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::next::{analyzer, parser};

    const SOURCE: &str = r#"axl 4
app Demo
entity Customer
  id: uuid key
capacity CustomerStore
  op save Customer -> Result<Customer>
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
  effect db.write
blueprint CRM
  in store: CustomerStore
  use store = SqliteCustomers
  requires customer.valid
"#;

    #[test]
    fn graph_round_trips_through_packed_ir() {
        let graph = analyzer::analyze(&parser::parse(SOURCE).unwrap()).unwrap();
        let packed = encode(&graph).unwrap();
        assert!(packed.starts_with("4;1|Demo;10|"));
        assert_eq!(decode(&packed).unwrap(), graph);
    }

    #[test]
    fn matrix_wraps_without_changing_semantics() {
        let graph = analyzer::analyze(&parser::parse(SOURCE).unwrap()).unwrap();
        let packed = encode(&graph).unwrap();
        let formatted = matrix(&packed, 90).unwrap();
        assert!(formatted.contains('\n'));
        assert_eq!(decode(&formatted).unwrap(), graph);
    }
}
