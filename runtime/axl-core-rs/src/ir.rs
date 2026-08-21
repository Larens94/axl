use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Recall(String),
    ToolCall { name: String, arguments: Vec<Expression> },
    FunctionCall { name: String, arguments: Vec<Expression> },
    ListExpression(Vec<Expression>),
    MapExpression(Vec<(Expression, Expression)>),
    Binary { left: Box<Expression>, operator: String, right: Box<Expression> },
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub name: String,
    pub tools: Vec<String>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub name: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Let { target: String, value: Expression, type_name: Option<String> },
    Return(Expression),
    Emit(Expression),
    MemoryWrite(MemoryWrite),
    Forget(String),
    If(If),
    While(While),
    Agent(Agent),
    Workflow(Workflow),
    Run(String),
    Function(Function),
    Annotation(Annotation),
    UiView(UiView),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

// Manual Serialize/Deserialize for Value (used in memory persistence)
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
        }
    }

    pub fn from_json_value(val: &serde_json::Value) -> Result<Self, String> {
        match val {
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            serde_json::Value::Number(n) => Ok(Value::Int(n.as_i64().unwrap_or(0))),
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Array(arr) => {
                // Check for $ax.map format
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
                    }
                }
                let items: Vec<Value> = arr.iter().map(|i| Self::from_json_value(i)).collect::<Result<_, _>>()?;
                Ok(Value::List(items))
            }
            _ => Err("invalid value in JSON".into()),
        }
    }
}
