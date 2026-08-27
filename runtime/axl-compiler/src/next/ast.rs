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
    Enum(Enum),
    Entity(Entity),
    Capacity(Capacity),
    Skill(Skill),
    Blueprint(Blueprint),
    Instance(Instance),
    Flow(Flow),
    Agent(Agent),
}

impl Declaration {
    pub fn name(&self) -> &str {
        match self {
            Self::Enum(value) => &value.name,
            Self::Entity(value) => &value.name,
            Self::Capacity(value) => &value.name,
            Self::Skill(value) => &value.name,
            Self::Blueprint(value) => &value.name,
            Self::Instance(value) => &value.name,
            Self::Flow(value) => &value.name,
            Self::Agent(value) => &value.name,
        }
    }

    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Enum(value) => &value.span,
            Self::Entity(value) => &value.span,
            Self::Capacity(value) => &value.span,
            Self::Skill(value) => &value.span,
            Self::Blueprint(value) => &value.span,
            Self::Instance(value) => &value.span,
            Self::Flow(value) => &value.span,
            Self::Agent(value) => &value.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub span: SourceSpan,
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
    Parameter,
    State,
    Event,
    Action,
    Error,
    Policy,
}

impl PortKind {
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Input => "in",
            Self::Output => "out",
            Self::Slot => "slot",
            Self::Hook => "hook",
            Self::Parameter => "param",
            Self::State => "state",
            Self::Event => "event",
            Self::Action => "action",
            Self::Error => "error",
            Self::Policy => "policy",
        }
    }

    pub fn graph_kind(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Slot => "slot",
            Self::Hook => "hook",
            Self::Parameter => "parameter",
            Self::State => "state",
            Self::Event => "event",
            Self::Action => "action",
            Self::Error => "error",
            Self::Policy => "policy",
        }
    }

    pub fn accepts_provider(&self) -> bool {
        matches!(
            self,
            Self::Input | Self::Slot | Self::Hook | Self::Action | Self::Policy
        )
    }

    pub fn is_customization_surface(&self) -> bool {
        matches!(
            self,
            Self::Input | Self::Slot | Self::Hook | Self::Parameter | Self::Action | Self::Policy
        )
    }
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
pub struct Instance {
    pub name: String,
    pub blueprint: String,
    pub settings: Vec<Setting>,
    pub bindings: Vec<Binding>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    pub parameter: String,
    pub value: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub input: String,
    pub output: String,
    pub dependencies: Vec<FlowDependency>,
    pub bindings: Vec<Binding>,
    pub statements: Vec<FlowStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowDependency {
    pub name: String,
    pub capacity: String,
    pub default: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FlowStatement {
    Let {
        name: String,
        expression: String,
        span: SourceSpan,
    },
    Require {
        expression: String,
        message: String,
        span: SourceSpan,
    },
    Call {
        name: String,
        dependency: String,
        operation: String,
        argument: String,
        propagate: bool,
        span: SourceSpan,
    },
    Make {
        name: String,
        type_name: String,
        fields: Vec<RecordFieldValue>,
        span: SourceSpan,
    },
    Fold {
        name: String,
        type_name: String,
        collection: String,
        initial: String,
        item: String,
        update: String,
        span: SourceSpan,
    },
    Run {
        name: String,
        flow: String,
        argument: String,
        propagate: bool,
        span: SourceSpan,
    },
    Match {
        name: String,
        type_name: String,
        subject: String,
        cases: Vec<MatchCase>,
        span: SourceSpan,
    },
    Return {
        expression: String,
        span: SourceSpan,
    },
}

impl FlowStatement {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Let { span, .. }
            | Self::Require { span, .. }
            | Self::Call { span, .. }
            | Self::Make { span, .. }
            | Self::Fold { span, .. }
            | Self::Run { span, .. }
            | Self::Match { span, .. }
            | Self::Return { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordFieldValue {
    pub name: String,
    pub expression: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchCase {
    pub variant: String,
    pub expression: String,
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
