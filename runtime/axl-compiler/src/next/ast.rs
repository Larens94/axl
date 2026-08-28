use serde::{Deserialize, Serialize};

use super::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub version: u16,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub span: SourceSpan,
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
    Event(EventDecl),
    Subscription(Subscription),
    Job(JobDecl),
    Api(Api),
    Ui(Ui),
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
            Self::Event(value) => &value.name,
            Self::Subscription(value) => &value.flow,
            Self::Job(value) => &value.name,
            Self::Api(value) => &value.name,
            Self::Ui(value) => &value.name,
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
            Self::Event(value) => &value.span,
            Self::Subscription(value) => &value.span,
            Self::Job(value) => &value.span,
            Self::Api(value) => &value.span,
            Self::Ui(value) => &value.span,
            Self::Agent(value) => &value.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDecl {
    pub name: String,
    pub payload: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    pub event: String,
    pub payload: String,
    pub flow: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobDecl {
    pub name: String,
    pub flow: String,
    pub schedule: Option<String>,
    pub retry: u32,
    pub idempotent: bool,
    pub store_capacity: String,
    pub store_provider: String,
    pub span: SourceSpan,
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
    pub idempotent: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub provides: String,
    pub native: Option<NativeBinding>,
    pub configs: Vec<SkillConfig>,
    pub effects: Vec<String>,
    pub capabilities: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillConfig {
    pub name: String,
    pub type_name: String,
    pub value: String,
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
    Attempt {
        name: String,
        dependency: String,
        operation: String,
        argument: String,
        propagate: bool,
        retry: u32,
        timeout_ms: u64,
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
    Map {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        expression: String,
        span: SourceSpan,
    },
    Filter {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        predicate: String,
        span: SourceSpan,
    },
    Sort {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        key: String,
        direction: String,
        span: SourceSpan,
    },
    Group {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        key: String,
        span: SourceSpan,
    },
    Parallel {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        flow: String,
        argument: String,
        propagate: bool,
        span: SourceSpan,
    },
    Race {
        name: String,
        type_name: String,
        collection: String,
        item: String,
        flow: String,
        argument: String,
        propagate: bool,
        span: SourceSpan,
    },
    Emit {
        event: String,
        argument: String,
        span: SourceSpan,
    },
    Enqueue {
        job: String,
        argument: String,
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
            | Self::Attempt { span, .. }
            | Self::Make { span, .. }
            | Self::Fold { span, .. }
            | Self::Run { span, .. }
            | Self::Match { span, .. }
            | Self::Map { span, .. }
            | Self::Filter { span, .. }
            | Self::Sort { span, .. }
            | Self::Group { span, .. }
            | Self::Parallel { span, .. }
            | Self::Race { span, .. }
            | Self::Emit { span, .. }
            | Self::Enqueue { span, .. }
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
pub struct Api {
    pub name: String,
    pub middlewares: Vec<ApiMiddleware>,
    pub auth: Option<ApiAuth>,
    pub routes: Vec<ApiRoute>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiMiddleware {
    pub phase: String,
    pub capacity: String,
    pub provider: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiAuth {
    pub scheme: String,
    pub capacity: String,
    pub provider: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRoute {
    pub method: String,
    pub path: String,
    pub input: String,
    pub output: String,
    pub flow: String,
    pub input_source: String,
    pub input_name: Option<String>,
    pub bindings: Vec<HttpRequestBinding>,
    pub guards: Vec<ApiRouteGuard>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRouteGuard {
    pub kind: String,
    pub flow: String,
    pub param: Option<String>,
    pub source: String,
    pub name: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ui {
    pub name: String,
    pub pages: Vec<UiPage>,
    pub forms: Vec<UiForm>,
    pub actions: Vec<UiAction>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPage {
    pub path: String,
    pub input: String,
    pub output: String,
    pub flow: String,
    pub input_source: String,
    pub input_name: Option<String>,
    pub bindings: Vec<HttpRequestBinding>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiForm {
    pub path: String,
    pub entity: String,
    pub output: String,
    pub flow: String,
    pub submit: Option<String>,
    pub redirect: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAction {
    pub path: String,
    pub method: String,
    pub submit: String,
    pub on: Option<String>,
    pub redirect: Option<String>,
    pub clear_cookie: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequestBinding {
    pub target: Option<String>,
    pub source: String,
    pub name: Option<String>,
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
