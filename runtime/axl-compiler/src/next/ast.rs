use serde::{Deserialize, Serialize};

use super::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub version: u16,
    pub name: String,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Declaration {
    Entity(Entity),
    Capacity(Capacity),
    Skill(Skill),
    Blueprint(Blueprint),
    Agent(Agent),
}

impl Declaration {
    pub fn name(&self) -> &str {
        match self {
            Self::Entity(value) => &value.name,
            Self::Capacity(value) => &value.name,
            Self::Skill(value) => &value.name,
            Self::Blueprint(value) => &value.name,
            Self::Agent(value) => &value.name,
        }
    }

    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Entity(value) => &value.span,
            Self::Capacity(value) => &value.span,
            Self::Skill(value) => &value.span,
            Self::Blueprint(value) => &value.span,
            Self::Agent(value) => &value.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<EntityField>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityField {
    pub name: String,
    pub type_name: String,
    pub qualifiers: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capacity {
    pub name: String,
    pub operations: Vec<Operation>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub input: String,
    pub output: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub provides: String,
    pub native: Option<NativeBinding>,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeBinding {
    pub target: String,
    pub symbol: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub ports: Vec<Port>,
    pub bindings: Vec<Binding>,
    pub contracts: Vec<Contract>,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    Input,
    Output,
    Slot,
    Hook,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub kind: PortKind,
    pub name: String,
    pub type_name: String,
    pub default: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub port: String,
    pub provider: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractKind {
    Requires,
    Ensures,
    Invariant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
    pub kind: ContractKind,
    pub expression: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub beliefs: Vec<String>,
    pub goals: Vec<String>,
    pub plans: Vec<String>,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
    pub span: SourceSpan,
}
