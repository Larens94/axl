use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("axl-rs: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.len() != 6
        || arguments[0] != "build"
        || arguments[2] != "--target"
        || arguments[3] != "web"
        || !matches!(arguments[4].as_str(), "-o" | "--output")
    {
        return Err("usage: axl-rs build <file.axl> --target web -o <directory>".into());
    }
    let source = fs::read_to_string(&arguments[1])?;
    let program = axl_core::parse(&source)?;
    axl_core::build_web(&program, &PathBuf::from(&arguments[5]))?;
    Ok(())
}
