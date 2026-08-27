use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphIr {
    pub schema: String,
    pub app: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub contracts: Vec<GraphContract>,
    pub effects: Vec<GraphGrant>,
    pub capabilities: Vec<GraphGrant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphContract {
    pub owner: String,
    pub kind: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphGrant {
    pub owner: String,
    pub name: String,
}

impl GraphIr {
    pub fn canonicalize(&mut self) {
        self.nodes.sort_by(|left, right| left.id.cmp(&right.id));
        self.edges.sort_by(|left, right| {
            (&left.from, &left.kind, &left.to, &left.interface).cmp(&(
                &right.from,
                &right.kind,
                &right.to,
                &right.interface,
            ))
        });
        self.contracts.sort_by(|left, right| {
            (&left.owner, &left.kind, &left.expression).cmp(&(
                &right.owner,
                &right.kind,
                &right.expression,
            ))
        });
        self.effects
            .sort_by(|left, right| (&left.owner, &left.name).cmp(&(&right.owner, &right.name)));
        self.capabilities
            .sort_by(|left, right| (&left.owner, &left.name).cmp(&(&right.owner, &right.name)));
    }
}
