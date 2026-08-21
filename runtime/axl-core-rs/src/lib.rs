pub mod ir;
pub mod type_names;
pub mod ui_registry;
pub mod compact;
pub mod validation;
pub mod typechecker;
pub mod memory;
pub mod policy;
pub mod interpreter;
pub mod serialization;
pub mod render_web;

pub use compact::{parse_compact, is_compact_source, program_to_compact, split_compact_frames};
pub use validation::validate;
pub use typechecker::typecheck;
pub use interpreter::{run_program, render_value, InterpreterConfig, ExecutionResult, RuntimeError};
pub use serialization::{program_to_json, program_from_json, program_to_document};
pub use render_web::build_web;
pub use memory::{InMemoryStore, SQLiteMemoryStore, MemoryStore};
pub use policy::Tool;
pub use ir::{Program, Value, Expression, Instruction};

use std::sync::{Arc, Mutex};

/// Run a compiled program with default settings.
pub fn run(program: &Program) -> Result<ExecutionResult, RuntimeError> {
    let memory: Arc<Mutex<dyn MemoryStore>> = Arc::new(Mutex::new(InMemoryStore::new()));
    run_program(program, vec![], memory, InterpreterConfig::default(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_numeric_ui() {
        let program = parse_compact("3;80|1|1|\"demo\";60|1;61|1|1;62|1|\"AX\";61|2|3;62|1|\"Row\";61|3|4;62|1|\"Film\";62|2|\"New\";62|3|#1;62|4|#1;63|1|3;99;99;99").unwrap();
        assert_eq!(program.instructions.len(), 2);
    }

    #[test]
    fn rejects_wrong_property_type() {
        let program = parse_compact("3;60|1;61|1|1;62|1|#1;99").unwrap();
        let err = validate(&program).unwrap_err();
        assert!(err.0.contains("requires string"));
    }

    #[test]
    fn basic_emit_string() {
        let program = parse_compact("2;12|\"hello world\"").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0], Value::String("hello world".into()));
    }

    #[test]
    fn arithmetic_rpn() {
        let program = parse_compact("2;10|x|#2,#3,#4,*,+|i;12|$x").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output[0], Value::Int(14));
    }

    #[test]
    fn if_else() {
        let program = parse_compact("2;10|x|#5|i;30|$x,#3,>;12|\"big\";31;12|\"small\";99").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output[0], Value::String("big".into()));
    }

    #[test]
    fn memory_roundtrip() {
        let program = parse_compact("2;20|foo|#42;21|foo").unwrap();
        let result = run(&program).unwrap();
        assert!(result.memory.is_empty());
    }

    #[test]
    fn compact_to_json_roundtrip() {
        let program = parse_compact("2;10|x|#7|i;12|$x").unwrap();
        let json = program_to_json(&program).unwrap();
        let restored = program_from_json(&json).unwrap();
        assert_eq!(program.instructions.len(), restored.instructions.len());
    }

    #[test]
    fn json_roundtrip() {
        let program = parse_compact("2;12|\"test\"").unwrap();
        let json = program_to_json(&program).unwrap();
        let restored = program_from_json(&json).unwrap();
        let result = run(&restored).unwrap();
        assert_eq!(result.output[0], Value::String("test".into()));
    }
}
