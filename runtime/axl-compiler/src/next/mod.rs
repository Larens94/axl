pub mod analyzer;
pub mod ast;
pub mod diagnostic;
pub mod expression;
pub mod formatter;
pub mod http;
pub mod ir;
pub mod packed;
pub mod parser;
pub mod resolver;
pub mod runtime;
pub mod targets;
pub mod ui;

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use diagnostic::{Diagnostic, tag_diagnostics};
use ir::GraphIr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compilation {
    pub source: String,
    pub graph: GraphIr,
    pub packed: String,
    pub matrix: String,
}

pub fn compile_source(source: &str) -> std::result::Result<Compilation, Vec<Diagnostic>> {
    compile_source_at(source, None)
}

pub fn compile_source_at(
    source: &str,
    base_file: Option<&Path>,
) -> std::result::Result<Compilation, Vec<Diagnostic>> {
    let program = match parser::parse(source) {
        Ok(program) => program,
        Err(mut diagnostics) => {
            tag_diagnostics(&mut diagnostics, base_file);
            return Err(diagnostics);
        }
    };
    compile_program(&program, base_file)
}

pub fn compile_file(path: &Path) -> Result<std::result::Result<Compilation, Vec<Diagnostic>>> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read AXL source '{}'", path.display()))?;
    Ok(compile_source_at(&source, Some(path)))
}

fn compile_program(
    program: &ast::Program,
    base_file: Option<&Path>,
) -> std::result::Result<Compilation, Vec<Diagnostic>> {
    let merged = if program.imports.is_empty() {
        program.clone()
    } else {
        let Some(base_file) = base_file else {
            return Err(vec![resolver::imports_require_file_base(program).expect(
                "imports present but imports_require_file_base returned None",
            )]);
        };
        resolver::resolve_imports(program, base_file)?
    };
    let graph = match analyzer::analyze(&merged) {
        Ok(graph) => graph,
        Err(mut diagnostics) => {
            tag_diagnostics(&mut diagnostics, base_file);
            return Err(diagnostics);
        }
    };
    let packed = match packed::encode(&graph) {
        Ok(packed) => packed,
        Err(error) => {
            let mut diagnostics = vec![Diagnostic::error(
                "AXL-C001",
                "compact",
                error.to_string(),
                diagnostic::SourceSpan {
                    line: 1,
                    column: 1,
                    length: 1,
                },
            )];
            tag_diagnostics(&mut diagnostics, base_file);
            return Err(diagnostics);
        }
    };
    let matrix = match packed::matrix(&packed, 100) {
        Ok(matrix) => matrix,
        Err(error) => {
            let mut diagnostics = vec![Diagnostic::error(
                "AXL-C002",
                "compact",
                error.to_string(),
                diagnostic::SourceSpan {
                    line: 1,
                    column: 1,
                    length: 1,
                },
            )];
            tag_diagnostics(&mut diagnostics, base_file);
            return Err(diagnostics);
        }
    };
    Ok(Compilation {
        source: formatter::format(program),
        graph,
        packed,
        matrix,
    })
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
