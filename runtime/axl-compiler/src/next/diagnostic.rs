use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixSafety {
    Safe,
    Likely,
    Risky,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SourceSpan {
    pub fn line(line: usize, source: &str) -> Self {
        Self {
            line,
            column: 1,
            length: source.chars().count().max(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repair {
    pub kind: String,
    pub target: String,
    pub replacement: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub phase: String,
    pub severity: Severity,
    pub message: String,
    pub span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
    pub fix_safety: FixSafety,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repairs: Vec<Repair>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        phase: impl Into<String>,
        message: impl Into<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            code: code.into(),
            phase: phase.into(),
            severity: Severity::Error,
            message: message.into(),
            span,
            expected: None,
            found: None,
            fix_safety: FixSafety::Manual,
            repairs: Vec::new(),
        }
    }

    pub fn expected(mut self, expected: impl Into<String>, found: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self.found = Some(found.into());
        self
    }

    pub fn repair(mut self, safety: FixSafety, repair: Repair) -> Self {
        self.fix_safety = safety;
        self.repairs.push(repair);
        self
    }

    pub fn human(&self) -> String {
        let mut rendered = format!(
            "error[{}] {}:{}:{}: {}",
            self.code, self.phase, self.span.line, self.span.column, self.message
        );
        if let (Some(expected), Some(found)) = (&self.expected, &self.found) {
            rendered.push_str(&format!("\n  expected: {expected}\n  found: {found}"));
        }
        for repair in &self.repairs {
            if repair.candidates.is_empty() {
                if let Some(replacement) = &repair.replacement {
                    rendered.push_str(&format!(
                        "\n  repair({:?}): {} {} -> {}",
                        self.fix_safety, repair.kind, repair.target, replacement
                    ));
                }
            } else {
                rendered.push_str(&format!(
                    "\n  repair({:?}): {} {} using [{}]",
                    self.fix_safety,
                    repair.kind,
                    repair.target,
                    repair.candidates.join(", ")
                ));
            }
        }
        rendered
    }
}
