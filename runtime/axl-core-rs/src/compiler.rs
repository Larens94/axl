use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::compact::{parse_compact, split_compact_frames, CompactParseError};
use crate::ir::*;
use crate::validation;

#[derive(Debug)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompileError {}

const MAX_IMPORT_DEPTH: usize = 32;
const MAX_MODULES: usize = 256;
const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;

struct CompileContext {
    root: PathBuf,
    modules: usize,
    source_bytes: usize,
    imported: HashSet<PathBuf>,
}

/// Compile an AXL file with module support
pub fn compile_file(path: &Path) -> Result<Program, CompileError> {
    let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut ctx = CompileContext {
        root, modules: 0, source_bytes: 0,
        imported: HashSet::new(),
    };
    compile_file_inner(path, &mut ctx)
}

fn compile_file_inner(path: &Path, ctx: &mut CompileContext) -> Result<Program, CompileError> {
    if ctx.modules > MAX_MODULES {
        return Err(CompileError(format!("module count exceeds {MAX_MODULES}")));
    }
    if ctx.imported.contains(path) {
        return Err(CompileError(format!("cyclic import: {}", path.display())));
    }
    ctx.imported.insert(path.to_path_buf());
    ctx.modules += 1;

    let source = std::fs::read_to_string(path)
        .map_err(|e| CompileError(format!("cannot read '{}': {e}", path.display())))?;

    if source.len() > MAX_SOURCE_BYTES {
        return Err(CompileError("source exceeds 4MB".into()));
    }
    ctx.source_bytes += source.len();

    // Parse imports
    let (imports, local_source) = extract_imports(&source, path, &ctx.root)?;

    // Parse local program
    let mut program = parse_compact(&local_source)
        .map_err(|e| CompileError(format!("{}: {e}", path.display())))?;

    // Process imports
    for (alias, import_path) in &imports {
        let imported = compile_file_inner(import_path, ctx)?;

        // Namespace imported functions
        for instruction in imported.instructions {
            match instruction {
                Instruction::Function(func) => {
                    let namespaced = Function {
                        name: format!("{alias}.{}", func.name),
                        ..func
                    };
                    program.instructions.insert(0, Instruction::Function(namespaced));
                }
                _ => {}
            }
        }
    }

    Ok(program)
}

fn extract_imports(
    source: &str,
    path: &Path,
    root: &Path,
) -> Result<(Vec<(String, PathBuf)>, String), CompileError> {
    let mut imports = Vec::new();
    let mut aliases = HashSet::new();
    let mut local_frames = Vec::new();

    let frames = split_compact_frames(source)
        .map_err(|e| CompileError(format!("{}: {e}", path.display())))?;

    if frames.is_empty() || (frames[0] != "2" && frames[0] != "3") {
        return Err(CompileError("compact source requires version header '2' or '3'".into()));
    }
    local_frames.push(frames[0].clone());

    let mut depth = 0;
    for (i, frame) in frames[1..].iter().enumerate() {
        let fields: Vec<&str> = frame.split('|').collect();
        let opcode = fields.first().copied().unwrap_or("");

        if opcode == "1" && fields.len() == 3 {
            if depth > 0 {
                return Err(CompileError(format!("{}: frame {}: import must be top-level", path.display(), i + 1)));
            }
            let alias = fields[1];
            let relative_path = fields[2];

            if aliases.contains(alias) {
                return Err(CompileError(format!("{}: frame {}: duplicate import alias '{alias}'", path.display(), i + 1)));
            }

            let import_path = path.parent().unwrap_or(Path::new(".")).join(relative_path);
            let resolved = import_path.canonicalize()
                .map_err(|e| CompileError(format!("{}: frame {}: cannot resolve '{relative_path}': {e}", path.display(), i + 1)))?;

            if !resolved.starts_with(root) {
                return Err(CompileError(format!("{}: frame {}: import path escapes module root", path.display(), i + 1)));
            }

            aliases.insert(alias.to_string());
            imports.push((alias.to_string(), resolved));
        } else {
            local_frames.push(frame.clone());
            if matches!(opcode, "30" | "32" | "40" | "50" | "51") {
                depth += 1;
            } else if opcode == "99" && depth > 0 {
                depth -= 1;
            }
        }
    }

    Ok((imports, local_frames.join(";")))
}

/// List all functions in a program
pub fn list_functions(program: &Program) -> Vec<String> {
    program.instructions.iter().filter_map(|i| {
        if let Instruction::Function(f) = i {
            Some(f.name.clone())
        } else {
            None
        }
    }).collect()
}

/// List all agents in a program
pub fn list_agents(program: &Program) -> Vec<String> {
    program.instructions.iter().filter_map(|i| {
        if let Instruction::Agent(a) = i {
            Some(a.name.clone())
        } else {
            None
        }
    }).collect()
}

/// List all workflows in a program
pub fn list_workflows(program: &Program) -> Vec<String> {
    program.instructions.iter().filter_map(|i| {
        if let Instruction::Workflow(w) = i {
            Some(w.name.clone())
        } else {
            None
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_simple_file() {
        let dir = std::env::temp_dir().join("axl_compile_test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.axl");
        std::fs::write(&file, "2;12|\"hello\"").unwrap();

        let program = compile_file(&file).unwrap();
        assert_eq!(program.instructions.len(), 1);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn list_functions_works() {
        let program = Program { instructions: vec![
            Instruction::Function(Function {
                name: "test".into(), parameters: vec![], return_type: "int".into(), body: vec![],
            }),
            Instruction::Emit(Expression::Literal(Value::Int(1))),
        ] };
        let funcs = list_functions(&program);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0], "test");
    }

    #[test]
    fn list_agents_works() {
        let program = Program { instructions: vec![
            Instruction::Agent(Agent {
                name: "research".into(), tools: vec![], body: vec![],
                goal: None, tool_defs: vec![], memory_defs: vec![], handlers: vec![],
            }),
        ] };
        let agents = list_agents(&program);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0], "research");
    }
}
