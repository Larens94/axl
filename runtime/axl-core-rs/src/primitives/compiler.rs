use crate::ir::Value;
use super::PrimitiveError;
use std::path::Path;

/// axl_compile_frontend(source_path: string, output_dir: string) -> bool
/// Compiles an AXL source file into a React frontend (Refine + MUI + Vite).
pub fn axl_compile_frontend(args: &[Value]) -> Result<Value, PrimitiveError> {
    let source = args.first()
        .and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("axl_compile_frontend requires source_path:string".into()))?;
    let output = args.get(1)
        .and_then(|v| match v { Value::String(s) => Some(s.as_str()), _ => None })
        .ok_or_else(|| PrimitiveError("axl_compile_frontend requires output_dir:string".into()))?;

    let source_path = Path::new(source);
    let output_path = Path::new(output);

    // Parse AXL source
    let ast = axl_compiler::parser::parse_file(source_path)
        .map_err(|e| PrimitiveError(format!("axl_compile_frontend parse: {e}")))?;

    // Analyze
    let analyzed = axl_compiler::analyzer::analyze(ast)
        .map_err(|e| PrimitiveError(format!("axl_compile_frontend analyze: {e}")))?;

    // Generate React frontend
    axl_compiler::codegen::react::generate(&analyzed, None, output_path)
        .map_err(|e| PrimitiveError(format!("axl_compile_frontend codegen: {e}")))?;

    Ok(Value::Bool(true))
}
