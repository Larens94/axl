pub mod ir;
pub mod type_names;
pub mod ui_registry;
pub mod compact;
pub mod keyword;
pub mod validation;
pub mod typechecker;
pub mod memory;
pub mod policy;
pub mod interpreter;
pub mod serialization;
pub mod render_web;
pub mod llm;
pub mod primitives;
pub mod web;
pub mod compiler;
pub mod mimo;
pub mod server;

pub use compact::{parse_compact, is_compact_source, program_to_compact, split_compact_frames};
pub use validation::validate;
pub use typechecker::typecheck;
pub use interpreter::{run_program, render_value, InterpreterConfig, ExecutionResult, RuntimeError};
pub use serialization::{program_to_json, program_from_json, program_to_document};
pub use render_web::build_web;
pub use memory::{InMemoryStore, SQLiteMemoryStore, MemoryStore};
pub use policy::Tool;
pub use ir::{Program, Value, Expression, Instruction};
pub use llm::{LlmBackend, LlmError, MockBackend, reason, classify, extract, generate, generate_json, embed, similarity};

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

    #[test]
    fn llm_mock_reason() {
        let backend = MockBackend::new(vec!["Step 1: analyze\nStep 2: conclude\nAnswer: 42".into()]);
        let result = llm::reason(&backend, "solve this", "what is 6*7").unwrap();
        assert!(result.contains("42"));
    }

    #[test]
    fn llm_mock_classify() {
        let backend = MockBackend::new(vec!["news".into()]);
        let labels = vec!["news".into(), "opinion".into()];
        let result = llm::classify(&backend, "classify this", "breaking: something happened", &labels).unwrap();
        assert_eq!(result, "news");
    }

    #[test]
    fn llm_mock_extract() {
        let backend = MockBackend::new(vec!["Alice\nAcme Corp\nNew York".into()]);
        let result = llm::extract(&backend, "person, organization, location", "Alice from Acme Corp in New York").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Alice");
    }

    #[test]
    fn llm_mock_embed() {
        let backend = MockBackend::with_default();
        let embedding = backend.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 128);
    }

    #[test]
    fn llm_similarity() {
        let a = vec![1000i64, 0, 0, 1000];
        let b = vec![1000i64, 0, 0, 1000];
        let c = vec![0i64, 1000, 1000, 0];
        assert!((llm::similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((llm::similarity(&a, &c)).abs() < 0.001);
    }

    #[test]
    fn value_json_roundtrip_embedding() {
        let val = Value::Embedding(vec![100, 200, 300]);
        let json = val.to_json_value();
        let restored = Value::from_json_value(&json).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn value_json_roundtrip_agent_ref() {
        let val = Value::AgentRef("search_agent".into());
        let json = val.to_json_value();
        let restored = Value::from_json_value(&json).unwrap();
        assert_eq!(val, restored);
    }

    #[test]
    fn value_json_roundtrip_null() {
        let val = Value::Null;
        let json = val.to_json_value();
        assert_eq!(json, serde_json::Value::Null);
        let restored = Value::from_json_value(&json).unwrap();
        assert_eq!(val, restored);
    }

    // ========================================================================
    // Primitive tests
    // ========================================================================

    #[test]
    fn primitive_text_operations() {
        use primitives::call_primitive;
        assert_eq!(call_primitive("text_upper", &[Value::String("hello".into())]).unwrap(), Value::String("HELLO".into()));
        assert_eq!(call_primitive("text_lower", &[Value::String("HELLO".into())]).unwrap(), Value::String("hello".into()));
        assert_eq!(call_primitive("text_trim", &[Value::String("  hi  ".into())]).unwrap(), Value::String("hi".into()));
        assert_eq!(call_primitive("text_length", &[Value::String("abc".into())]).unwrap(), Value::Int(3));
        assert_eq!(call_primitive("text_contains", &[Value::String("hello world".into()), Value::String("world".into())]).unwrap(), Value::Bool(true));
    }

    #[test]
    fn primitive_list_operations() {
        use primitives::call_primitive;
        let list = Value::List(vec![Value::Int(3), Value::Int(1), Value::Int(2)]);
        let sorted = call_primitive("list_sort", &[list]).unwrap();
        assert_eq!(sorted, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));

        let list2 = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(call_primitive("list_length", &[list2.clone()]).unwrap(), Value::Int(3));
        assert_eq!(call_primitive("list_sum", &[list2]).unwrap(), Value::Int(6));
    }

    #[test]
    fn primitive_map_operations() {
        use primitives::call_primitive;
        let map = Value::Map(vec![
            (Value::String("a".into()), Value::Int(1)),
            (Value::String("b".into()), Value::Int(2)),
        ]);
        assert_eq!(call_primitive("map_get", &[map.clone(), Value::String("a".into())]).unwrap(), Value::Int(1));
        assert_eq!(call_primitive("map_keys", &[map]).unwrap(), Value::List(vec![Value::String("a".into()), Value::String("b".into())]));
    }

    #[test]
    fn primitive_math_operations() {
        use primitives::call_primitive;
        assert_eq!(call_primitive("math_add", &[Value::Int(2), Value::Int(3)]).unwrap(), Value::Int(5));
        assert_eq!(call_primitive("math_mul", &[Value::Int(4), Value::Int(5)]).unwrap(), Value::Int(20));
        assert_eq!(call_primitive("math_max", &[Value::Int(3), Value::Int(7)]).unwrap(), Value::Int(7));
    }

    #[test]
    fn primitive_crypto_operations() {
        use primitives::call_primitive;
        let hash = call_primitive("hash_sha256", &[Value::String("hello".into())]).unwrap();
        assert!(matches!(hash, Value::String(_)));

        let b64 = call_primitive("encode_base64", &[Value::String("hello".into())]).unwrap();
        assert_eq!(b64, Value::String("aGVsbG8=".into()));

        let decoded = call_primitive("decode_base64", &[Value::String("aGVsbG8=".into())]).unwrap();
        assert_eq!(decoded, Value::String("hello".into()));
    }

    #[test]
    fn primitive_json_operations() {
        use primitives::call_primitive;
        let parsed = call_primitive("json_parse", &[Value::String(r#"{"a":1,"b":"hello"}"#.into())]).unwrap();
        assert!(matches!(parsed, Value::Map(_)));

        let stringified = call_primitive("json_stringify", &[Value::Map(vec![
            (Value::String("x".into()), Value::Int(42)),
        ])]).unwrap();
        assert!(matches!(stringified, Value::String(_)));

        assert_eq!(call_primitive("json_validate", &[Value::String(r#"{"valid":true}"#.into())]).unwrap(), Value::Bool(true));
        assert_eq!(call_primitive("json_validate", &[Value::String("not json".into())]).unwrap(), Value::Bool(false));
    }

    #[test]
    fn primitive_file_operations() {
        use primitives::call_primitive;
        let test_path = "/tmp/axl_test_file.txt";
        let _ = std::fs::remove_file(test_path);

        assert_eq!(call_primitive("file_exists", &[Value::String(test_path.into())]).unwrap(), Value::Bool(false));
        assert_eq!(call_primitive("file_write", &[Value::String(test_path.into()), Value::String("test content".into())]).unwrap(), Value::Bool(true));
        assert_eq!(call_primitive("file_exists", &[Value::String(test_path.into())]).unwrap(), Value::Bool(true));

        let content = call_primitive("file_read", &[Value::String(test_path.into())]).unwrap();
        assert_eq!(content, Value::String("test content".into()));

        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn native_primitive_called_from_axl() {
        // AXL source: 2;10|result|"hello world",!text_upper/1|s;12|$result
        // This calls the native text_upper primitive
        let program = parse_compact("2;10|result|\"hello world\",!text_upper/1|s;12|$result").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0], Value::String("HELLO WORLD".into()));
    }

    #[test]
    fn native_primitive_json_from_axl() {
        // AXL source: 2;10|data|"{\"x\":42}",!json_parse/1|s;12|$data
        // This calls json_parse native primitive
        let program = parse_compact("2;10|data|\"{\\\"x\\\":42}\",!json_parse/1|s;12|$data").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output.len(), 1);
        // The output should be a map with x=42
        match &result.output[0] {
            Value::Map(entries) => {
                assert!(entries.iter().any(|(k, v)| matches!(k, Value::String(s) if s == "x") && matches!(v, Value::Int(42))));
            }
            other => panic!("expected map, got {:?}", other),
        }
    }

    #[test]
    fn native_primitive_hash_from_axl() {
        // AXL source: 2;10|hash|"test",!hash_sha256/1|s;12|$hash
        let program = parse_compact("2;10|hash|\"test\",!hash_sha256/1|s;12|$hash").unwrap();
        let result = run(&program).unwrap();
        assert_eq!(result.output.len(), 1);
        match &result.output[0] {
            Value::String(s) => assert_eq!(s.len(), 64), // SHA256 hex is 64 chars
            other => panic!("expected string hash, got {:?}", other),
        }
    }
}
