use std::path::Path;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub fix_safety: FixSafety,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repairs: Vec<Repair>,
}

/// Machine-readable output of `axl-compiler check --json` / `diagnose`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckReport {
    pub protocol: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub const PROTOCOL: &'static str = "axl-check/1";

    pub fn success(
        path: Option<&Path>,
        app: impl Into<String>,
        schema: impl Into<String>,
        nodes: usize,
        edges: usize,
    ) -> Self {
        Self {
            protocol: Self::PROTOCOL.into(),
            ok: true,
            path: path.map(|value| value.display().to_string()),
            app: Some(app.into()),
            schema: Some(schema.into()),
            nodes: Some(nodes),
            edges: Some(edges),
            diagnostics: Vec::new(),
        }
    }

    pub fn failure(path: Option<&Path>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            protocol: Self::PROTOCOL.into(),
            ok: false,
            path: path.map(|value| value.display().to_string()),
            app: None,
            schema: None,
            nodes: None,
            edges: None,
            diagnostics,
        }
    }
}

pub fn tag_diagnostics(diagnostics: &mut [Diagnostic], path: Option<&Path>) {
    let Some(path) = path else {
        return;
    };
    let path = path.display().to_string();
    for diagnostic in diagnostics {
        if diagnostic.path.is_none() {
            diagnostic.path = Some(path.clone());
        }
    }
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
            path: None,
            span,
            expected: None,
            found: None,
            hint: None,
            fix_safety: FixSafety::Manual,
            repairs: Vec::new(),
        }
    }

    pub fn at_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().display().to_string());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn expected(mut self, expected: impl Into<String>, found: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self.found = Some(found.into());
        self
    }

    pub fn repair(mut self, safety: FixSafety, repair: Repair) -> Self {
        self.fix_safety = safety;
        if self.hint.is_none() {
            self.hint = Some(repair_summary(&repair));
        }
        self.repairs.push(repair);
        self
    }

    pub fn human(&self) -> String {
        let location = self
            .path
            .as_deref()
            .map(|path| format!("{path}:"))
            .unwrap_or_default();
        let mut rendered = format!(
            "error[{}] {}{}:{}:{}: {}",
            self.code, location, self.phase, self.span.line, self.span.column, self.message
        );
        if let (Some(expected), Some(found)) = (&self.expected, &self.found) {
            rendered.push_str(&format!("\n  expected: {expected}\n  found: {found}"));
        }
        if let Some(hint) = &self.hint {
            rendered.push_str(&format!("\n  hint: {hint}"));
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

fn repair_summary(repair: &Repair) -> String {
    if repair.candidates.is_empty() {
        if let Some(replacement) = &repair.replacement {
            return format!("{} {} -> {}", repair.kind, repair.target, replacement);
        }
        return format!("{} {}", repair.kind, repair.target);
    }
    format!(
        "{} {} using [{}]",
        repair.kind,
        repair.target,
        repair.candidates.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_report_failure_envelope_is_stable() {
        let report = CheckReport::failure(
            Some(Path::new("examples/invalid/flow-calls.axl")),
            vec![
                Diagnostic::error(
                    "AXL-X817",
                    "execution",
                    "call receives the wrong argument type",
                    SourceSpan {
                        line: 39,
                        column: 1,
                        length: 35,
                    },
                )
                .at_path("examples/invalid/flow-calls.axl")
                .expected("uuid", "Movement"),
            ],
        );
        assert_eq!(report.protocol, "axl-check/1");
        assert!(!report.ok);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code, "AXL-X817");
    }
}
