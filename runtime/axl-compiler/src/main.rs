use std::path::PathBuf;
use anyhow::Result;

mod parser;
mod analyzer;
mod codegen;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: axl-compiler <input.axl> <output-dir>");
        std::process::exit(1);
    }
    
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    
    tracing::info!("Compiling {:?} to {:?}", input, output);
    
    // Parse AXL source
    let ast = parser::parse_file(&input)?;
    tracing::info!("Parsed {} entities, {} APIs", ast.entities.len(), ast.apis.len());
    
    // Analyze
    let analyzed = analyzer::analyze(ast)?;
    tracing::info!("Analysis complete");
    
    // Generate Rust code
    codegen::rust::generate(&analyzed, &output.join("backend"))?;
    tracing::info!("Generated Rust backend");
    
    // Generate React code
    codegen::react::generate(&analyzed, &output.join("frontend"))?;
    tracing::info!("Generated React frontend");
    
    // Generate SQL migrations
    codegen::sql::generate(&analyzed, &output.join("backend/migrations"))?;
    tracing::info!("Generated SQL migrations");
    
    tracing::info!("Compilation complete!");
    Ok(())
}
