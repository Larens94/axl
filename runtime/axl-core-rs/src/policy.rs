use crate::ir::Value;

const RESERVED: &[&str] = &[
    "agent", "call", "else", "emit", "end", "false", "forget", "if",
    "let", "memory", "meta", "recall", "run", "true", "uses", "while", "workflow",
];

pub type ToolHandler = Box<dyn Fn(&[Value]) -> Result<Value, String> + Send + Sync>;

pub struct Tool {
    pub name: String,
    pub handler: ToolHandler,
    pub effect: String,
    pub approval: bool,
}

impl Tool {
    pub fn new(name: impl Into<String>, handler: ToolHandler) -> Self {
        Self { name: name.into(), handler, effect: "read".into(), approval: false }
    }

    pub fn with_effect(mut self, effect: impl Into<String>) -> Self {
        self.effect = effect.into();
        self
    }

    pub fn with_approval(mut self, approval: bool) -> Self {
        self.approval = approval;
        self
    }
}

pub fn validate_tool(tool: &Tool) -> Result<(), String> {
    if tool.name.is_empty() || !tool.name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("invalid tool name".into());
    }
    if !tool.name.as_bytes()[0].is_ascii_alphabetic() && tool.name.as_bytes()[0] != b'_' {
        return Err("invalid tool name".into());
    }
    if RESERVED.contains(&tool.name.as_str()) {
        return Err(format!("reserved tool name '{}'", tool.name));
    }
    if tool.effect.is_empty() {
        return Err(format!("tool '{}' effect must be a non-empty string", tool.name));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub tool: String,
    pub arguments: Vec<Value>,
    pub effect: String,
}

#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub timestamp: String,
    pub tool: String,
    pub arguments: Vec<Value>,
    pub effect: String,
    pub decision: String,
}

impl AuditEvent {
    pub fn create(request: &ApprovalRequest, decision: &str) -> Self {
        Self {
            timestamp: super::memory::now_iso(),
            tool: request.tool.clone(),
            arguments: request.arguments.clone(),
            effect: request.effect.clone(),
            decision: decision.into(),
        }
    }
}

#[derive(Debug)]
pub struct ApprovalRequired(pub String);

impl std::fmt::Display for ApprovalRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ApprovalRequired {}
