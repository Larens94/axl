use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// Value System
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Embedding(Vec<i64>),
    AgentRef(String),
    Null,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::String(s) => { 0u8.hash(state); s.hash(state); }
            Value::Int(i) => { 1u8.hash(state); i.hash(state); }
            Value::Bool(b) => { 2u8.hash(state); b.hash(state); }
            Value::List(items) => { 3u8.hash(state); items.hash(state); }
            Value::Map(entries) => {
                4u8.hash(state);
                let mut pairs: Vec<_> = entries.iter().collect();
                pairs.sort_by_key(|(k, v)| {
                    let mut h = DefaultHasher::new();
                    k.hash(&mut h);
                    v.hash(&mut h);
                    h.finish()
                });
                pairs.hash(state);
            }
            Value::Embedding(v) => { 6u8.hash(state); v.len().hash(state); }
            Value::AgentRef(name) => { 7u8.hash(state); name.hash(state); }
            Value::Null => { 8u8.hash(state); }
        }
    }
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::String(_) => "string".into(),
            Value::Int(_) => "int".into(),
            Value::Bool(_) => "bool".into(),
            Value::List(items) => {
                let item_type = if items.is_empty() { "any".to_string() } else { items[0].type_name() };
                format!("list<{item_type}>")
            }
            Value::Map(entries) => {
                let (kt, vt) = if entries.is_empty() {
                    ("any".to_string(), "any".to_string())
                } else {
                    (entries[0].0.type_name(), entries[0].1.type_name())
                };
                format!("map<{kt},{vt}>")
            }
            Value::Embedding(_) => "embedding".into(),
            Value::AgentRef(_) => "agent_ref".into(),
            Value::Null => "null".into(),
        }
    }
}

// ============================================================================
// Expressions — LLM-native primitives
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Recall(String),

    // Tool system
    ToolCall { name: String, arguments: Vec<Expression> },
    FunctionCall { name: String, arguments: Vec<Expression> },

    // Collections
    ListExpression(Vec<Expression>),
    MapExpression(Vec<(Expression, Expression)>),

    // Operators
    Binary { left: Box<Expression>, operator: String, right: Box<Expression> },

    // ========================================================================
    // LLM-Native Primitives (NEW in 3.0)
    // ========================================================================

    /// Chain-of-thought reasoning
    /// `reason("step by step", problem)` → string
    Reason { instruction: Box<Expression>, input: Box<Expression> },

    /// Classification
    /// `classify("news or opinion", text, ["news", "opinion"])` → string
    Classify { instruction: Box<Expression>, input: Box<Expression>, labels: Vec<Expression> },

    /// Entity extraction
    /// `extract("person, org", text)` → list<string>
    Extract { schema: Box<Expression>, input: Box<Expression> },

    /// Text generation
    /// `generate("system prompt", messages)` → string
    Generate { system: Box<Expression>, messages: Vec<Expression> },

    /// Structured generation
    /// `generate_json("extract", text, schema)` → map
    GenerateJson { instruction: Box<Expression>, input: Box<Expression>, schema: Box<Expression> },

    /// Embedding generation
    /// `embed("text to vector")` → embedding
    Embed(Box<Expression>),

    /// Semantic similarity
    /// `similarity(emb1, emb2)` → float
    Similarity { left: Box<Expression>, right: Box<Expression> },

    // ========================================================================
    // Memory Semantic Primitives (NEW in 3.0)
    // ========================================================================

    /// Semantic recall — finds relevant memories by meaning
    /// `recall_semantic("what does user like", scope: "user:1")` → list
    RecallSemantic { query: Box<Expression>, scope: Option<Box<Expression>> },

    /// Semantic search — finds similar content
    /// `search_similar("similar to this", embedding)` → list
    SearchSimilar { query: Box<Expression>, embedding: Box<Expression> },

    // ========================================================================
    // Inter-Agent Communication (NEW in 3.0)
    // ========================================================================

    /// Direct message to agent
    /// `send(other_agent, "task", data)` → null
    Send { target: Box<Expression>, message: Box<Expression>, data: Box<Expression> },

    /// Synchronous delegation
    /// `delegate(other_agent, "method", args...)` → value
    Delegate { target: Box<Expression>, method: Box<Expression>, arguments: Vec<Expression> },

    /// Find agents by capability
    /// `find_agents(capability: "search")` → list<agent_ref>
    FindAgents { capability: Box<Expression> },

    /// Agent reference
    /// `agent_ref("search_agent")` → agent_ref
    AgentRef(String),
}

// ============================================================================
// Agent Structure (Enhanced in 3.0)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWrite {
    pub key: String,
    pub value: Expression,
    pub confidence: i32,
    pub ttl_seconds: Option<i64>,
    pub source: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: Expression,
    pub body: Vec<Instruction>,
    pub else_body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct While {
    pub condition: Expression,
    pub body: Vec<Instruction>,
}

/// Tool definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDef {
    pub name: String,
    pub effect: String,
    pub approval: bool,
    pub timeout_ms: Option<i64>,
    pub retries: i32,
    pub parameters: Vec<Parameter>,
    pub return_type: String,
}

/// Agent memory definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDef {
    pub name: String,
    pub scope: Option<String>,
    pub ttl_seconds: Option<i64>,
    pub confidence: Option<i32>,
}

/// Event handler
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHandler {
    pub event_type: String, // "query", "message", "schedule", "tool_result"
    pub parameters: Vec<Parameter>,
    pub body: Vec<Instruction>,
}

/// Agent definition (Enhanced in 3.0)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub name: String,
    pub goal: Option<String>,
    pub tools: Vec<String>,
    pub tool_defs: Vec<ToolDef>,
    pub memory_defs: Vec<MemoryDef>,
    pub handlers: Vec<EventHandler>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub name: String,
    pub handlers: Vec<EventHandler>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: String,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub kind: i32,
    pub target: i32,
    pub value: String,
}

// ============================================================================
// UI Types (unchanged)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiProperty {
    pub property_id: i32,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEvent {
    pub event_id: i32,
    pub action_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiNode {
    pub node_id: i32,
    pub component_id: i32,
    pub properties: Vec<UiProperty>,
    pub events: Vec<UiEvent>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiView {
    pub view_id: i32,
    pub root: UiNode,
}

// ============================================================================
// Instructions
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Let { target: String, value: Expression, type_name: Option<String> },
    Return(Expression),
    Emit(Expression),
    MemoryWrite(MemoryWrite),
    Forget(String),
    If(If),
    While(While),

    // Agent constructs
    Agent(Agent),
    Workflow(Workflow),
    Run(String),
    Function(Function),

    // Event handlers
    OnEvent(EventHandler),

    // Inter-agent
    Send { target: Expression, message: Expression, data: Expression },
    Delegate { target: Expression, method: Expression, arguments: Vec<Expression> },
    Broadcast { message: Expression, data: Expression },

    // Scheduling
    Schedule { cron: String, body: Vec<Instruction> },

    // Observability
    Trace { message: Expression, data: Option<Expression> },
    Metric { name: String, value: Expression },

    // Legacy
    Annotation(Annotation),
    UiView(UiView),
}

// ============================================================================
// Program
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

// ============================================================================
// Value JSON Serialization
// ============================================================================

impl Value {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Int(n) => serde_json::json!(n),
            Value::Bool(b) => serde_json::json!(b),
            Value::List(items) => serde_json::Value::Array(items.iter().map(|i| i.to_json_value()).collect()),
            Value::Map(entries) => {
                let pairs: Vec<serde_json::Value> = entries.iter()
                    .map(|(k, v)| serde_json::json!([k.to_json_value(), v.to_json_value()]))
                    .collect();
                serde_json::json!({"$ax.map": pairs})
            }
            Value::Embedding(v) => serde_json::json!({"$ax.embedding": v}),
            Value::AgentRef(name) => serde_json::json!({"$ax.agent": name}),
            Value::Null => serde_json::Value::Null,
        }
    }

    pub fn from_json_value(val: &serde_json::Value) -> Result<Self, String> {
        match val {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            serde_json::Value::Number(n) => {
                Ok(Value::Int(n.as_i64().unwrap_or(0)))
            }
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Array(arr) => {
                if arr.len() == 1 {
                    if let serde_json::Value::Object(obj) = &arr[0] {
                        if let Some(serde_json::Value::Array(pairs)) = obj.get("$ax.map") {
                            let mut entries = Vec::new();
                            for pair in pairs {
                                if let serde_json::Value::Array(kv) = pair {
                                    if kv.len() == 2 {
                                        entries.push((Self::from_json_value(&kv[0])?, Self::from_json_value(&kv[1])?));
                                    }
                                }
                            }
                            return Ok(Value::Map(entries));
                        }
                        if let Some(serde_json::Value::Array(v)) = obj.get("$ax.embedding") {
                            let ints: Vec<i64> = v.iter().filter_map(|n| n.as_i64()).collect();
                            return Ok(Value::Embedding(ints));
                        }
                        if let Some(serde_json::Value::String(name)) = obj.get("$ax.agent") {
                            return Ok(Value::AgentRef(name.clone()));
                        }
                    }
                }
                let items: Vec<Value> = arr.iter().map(|i| Self::from_json_value(i)).collect::<Result<_, _>>()?;
                Ok(Value::List(items))
            }
            serde_json::Value::Object(obj) => {
                if let Some(serde_json::Value::Array(v)) = obj.get("$ax.embedding") {
                    let ints: Vec<i64> = v.iter().filter_map(|n| n.as_i64()).collect();
                    return Ok(Value::Embedding(ints));
                }
                if let Some(serde_json::Value::String(name)) = obj.get("$ax.agent") {
                    return Ok(Value::AgentRef(name.clone()));
                }
                Err("invalid value in JSON (unknown object)".into())
            }
        }
    }
}
