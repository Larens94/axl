use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::ast::{Declaration, Import, Program};
use super::diagnostic::Diagnostic;
use super::parser;

pub fn resolve_imports(program: &Program, base_file: &Path) -> Result<Program, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let base_dir = base_file.parent().unwrap_or_else(|| Path::new("."));
    let mut visiting = BTreeSet::new();
    let mut done = BTreeSet::new();
    let mut merged = Vec::new();

    for import in &program.imports {
        merge_import(
            import,
            base_dir,
            base_file,
            &mut visiting,
            &mut done,
            &mut merged,
            &mut diagnostics,
        );
    }

    merged.extend(program.declarations.clone());

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(Program {
        version: program.version,
        name: program.name.clone(),
        imports: Vec::new(),
        declarations: merged,
    })
}

fn merge_import(
    import: &Import,
    base_dir: &Path,
    owner_file: &Path,
    visiting: &mut BTreeSet<PathBuf>,
    done: &mut BTreeSet<PathBuf>,
    merged: &mut Vec<Declaration>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let owner = owner_file.display().to_string();
    let Some(resolved) = resolve_import_path(&import.path, base_dir) else {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P930",
                "parse",
                "import path must stay within the project directory",
                import.span.clone(),
            )
            .at_path(&owner)
            .expected("a relative path such as './module.axl'", &import.path),
        );
        return;
    };

    let canonical = match resolved.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P931",
                    "imports",
                    format!("import path '{}' was not found", import.path),
                    import.span.clone(),
                )
                .at_path(&owner),
            );
            return;
        }
    };

    // Diamond imports: already fully merged → no-op (do not re-extend declarations).
    if done.contains(&canonical) {
        return;
    }

    if !visiting.insert(canonical.clone()) {
        diagnostics.push(
            Diagnostic::error(
                "AXL-P932",
                "imports",
                format!("circular import through '{}'", import.path),
                import.span.clone(),
            )
            .at_path(&owner),
        );
        return;
    }

    let import_path = canonical.display().to_string();
    let source = match std::fs::read_to_string(&canonical) {
        Ok(source) => source,
        Err(_) => {
            diagnostics.push(
                Diagnostic::error(
                    "AXL-P931",
                    "imports",
                    format!("import path '{}' was not found", import.path),
                    import.span.clone(),
                )
                .at_path(&owner),
            );
            visiting.remove(&canonical);
            return;
        }
    };

    let imported = match parser::parse(&source) {
        Ok(program) => program,
        Err(mut parse_errors) => {
            for diagnostic in &mut parse_errors {
                if diagnostic.path.is_none() {
                    diagnostic.path = Some(import_path.clone());
                }
            }
            diagnostics.extend(parse_errors);
            visiting.remove(&canonical);
            return;
        }
    };

    let import_dir = canonical.parent().unwrap_or(base_dir);
    for nested in &imported.imports {
        merge_import(
            nested,
            import_dir,
            &canonical,
            visiting,
            done,
            merged,
            diagnostics,
        );
    }
    merged.extend(imported.declarations);
    visiting.remove(&canonical);
    done.insert(canonical);
}

fn resolve_import_path(path: &str, base_dir: &Path) -> Option<PathBuf> {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') {
        return None;
    }
    Some(base_dir.join(trimmed))
}

pub fn imports_require_file_base(program: &Program) -> Option<Diagnostic> {
    program.imports.first().map(|import| {
        Diagnostic::error(
            "AXL-P933",
            "imports",
            "file imports require compiling from a source file path",
            import.span.clone(),
        )
        .expected("compile_file(path)", "compile_source without a base path")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/apps")
            .join(name)
    }

    #[test]
    fn resolves_relative_import_into_merged_declarations() {
        let source = std::fs::read_to_string(fixture("import-demo.axl")).unwrap();
        let program = parser::parse(&source).unwrap();
        let merged =
            resolve_imports(&program, &fixture("import-demo.axl")).expect("imports resolve");
        assert!(merged.imports.is_empty());
        assert!(
            merged
                .declarations
                .iter()
                .any(|declaration| declaration.name() == "BalanceInput")
        );
        assert!(
            merged
                .declarations
                .iter()
                .any(|declaration| declaration.name() == "CalculateBalance")
        );
        assert!(
            merged
                .declarations
                .iter()
                .any(|declaration| declaration.name() == "DemoBalance")
        );
    }

    #[test]
    fn missing_import_reports_stable_code() {
        let source = r#"axl 4
app MissingImport
import "./missing-module.axl"
"#;
        let program = parser::parse(source).unwrap();
        let diagnostics =
            resolve_imports(&program, &fixture("import-demo.axl")).expect_err("missing import");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "AXL-P931")
        );
    }

    #[test]
    fn diamond_imports_merge_shared_module_once() {
        let path = fixture("import-diamond-demo.axl");
        let source = std::fs::read_to_string(&path).unwrap();
        let program = parser::parse(&source).unwrap();
        let merged = resolve_imports(&program, &path).expect("diamond imports resolve");
        let email_count = merged
            .declarations
            .iter()
            .filter(|declaration| declaration.name() == "EmailMessage")
            .count();
        assert_eq!(email_count, 1, "shared email module must merge once");
        assert!(
            merged
                .declarations
                .iter()
                .any(|declaration| declaration.name() == "LeftUsesEmail")
        );
        assert!(
            merged
                .declarations
                .iter()
                .any(|declaration| declaration.name() == "RightUsesEmail")
        );
    }
}
