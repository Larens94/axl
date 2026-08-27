pub mod analyzer;
pub mod ast;
pub mod diagnostic;
pub mod expression;
pub mod formatter;
pub mod http;
pub mod ir;
pub mod packed;
pub mod parser;
pub mod runtime;
pub mod targets;

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use diagnostic::Diagnostic;
use ir::GraphIr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compilation {
    pub source: String,
    pub graph: GraphIr,
    pub packed: String,
    pub matrix: String,
}

pub fn compile_source(source: &str) -> std::result::Result<Compilation, Vec<Diagnostic>> {
    let program = parser::parse(source)?;
    let graph = analyzer::analyze(&program)?;
    let packed = packed::encode(&graph).map_err(|error| {
        vec![Diagnostic::error(
            "AXL-C001",
            "compact",
            error.to_string(),
            diagnostic::SourceSpan {
                line: 1,
                column: 1,
                length: 1,
            },
        )]
    })?;
    let matrix = packed::matrix(&packed, 100).map_err(|error| {
        vec![Diagnostic::error(
            "AXL-C002",
            "compact",
            error.to_string(),
            diagnostic::SourceSpan {
                line: 1,
                column: 1,
                length: 1,
            },
        )]
    })?;
    Ok(Compilation {
        source: formatter::format(&program),
        graph,
        packed,
        matrix,
    })
}

pub fn compile_file(path: &Path) -> Result<std::result::Result<Compilation, Vec<Diagnostic>>> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read AXL source '{}'", path.display()))?;
    Ok(compile_source(&source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compilation_exposes_all_three_representations() {
        let source = r#"axl 4
app Demo
entity Customer
  id: uuid key
capacity CustomerStore
  op save Customer -> Result<Customer>
skill SqliteCustomers provides CustomerStore
  native rust crm::sqlite
blueprint CRM
  in store: CustomerStore
  use store = SqliteCustomers
"#;
        let compiled = compile_source(source).unwrap();
        assert!(compiled.source.starts_with("axl 4\napp Demo"));
        assert_eq!(compiled.graph.schema, "ax-ir/4.0");
        assert!(compiled.packed.starts_with("4;"));
        assert_eq!(packed::decode(&compiled.matrix).unwrap(), compiled.graph);
    }
}
