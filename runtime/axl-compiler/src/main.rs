use std::path::PathBuf;
use anyhow::Result;
use axl_compiler::compile_application;

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
    
    compile_application(&input, &output)?;
    tracing::info!("Compilation complete!");
    Ok(())
}
