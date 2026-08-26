use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};


fn main() {
    if let Err(error) = run() {
        eprintln!("axl: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") => run_command(&args[1..]),
        Some("check") => check_command(&args[1..]),
        Some("compile") => compile_command(&args[1..]),
        Some("exec") => exec_command(&args[1..]),
        Some("pack") => pack_command(&args[1..]),
        Some("fmt") => fmt_command(&args[1..]),
        Some("build") => build_command(&args[1..]),
        Some("dev") => dev_command(&args[1..]),
        Some("serve") => serve_command(&args[1..]),
        _ => {
            eprintln!("usage:");
            eprintln!("  axl run <file.axl>     [--memory <db>] [--max-steps <n>]");
            eprintln!("  axl check <file.axl>   # validate a full application");
            eprintln!("  axl compile <file.axl> -o <output.json>");
            eprintln!("  axl exec <file.json>   [--memory <db>] [--max-steps <n>]");
            eprintln!("  axl pack <file.axl>    -o <output.axl>");
            eprintln!("  axl fmt <file.axl>     [--width <n>] [--check] [-o <file>]");
            eprintln!("  axl build <file.axl>   [-o <dir>]             # full application");
            eprintln!("  axl dev <file.axl>     [-o <dir>]             # build and run full stack");
            eprintln!("  axl build <file.axl>   --target web -o <dir>  # compact AX-UI");
            eprintln!("  axl serve <file.axl>   [--port <port>] [-o <dir>]");
            Err("unknown command".into())
        }
    }
}

fn dev_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = None;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" if i + 1 < args.len() => { output = Some(PathBuf::from(&args[i + 1])); i += 2; }
            value if value.starts_with('-') => return Err(format!("unknown dev option '{value}'").into()),
            value => { file = Some(PathBuf::from(value)); i += 1; }
        }
    }
    let file = file.ok_or("usage: axl dev <file.axl> [-o <dir>]")?;
    let output = output.unwrap_or_else(|| {
        let name = file.file_stem().and_then(|value| value.to_str()).unwrap_or("app");
        PathBuf::from("build").join(name)
    });
    axl_compiler::compile_application(&file, &output)?;
    println!("AXL full stack generated; starting Rust + React...");
    let status = Command::new("sh").arg(output.join("dev.sh")).status()?;
    if !status.success() { return Err(format!("development stack exited with {status}").into()); }
    Ok(())
}

fn check_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let file = args.first().ok_or("usage: axl check <file.axl>")?;
    if args.len() != 1 {
        return Err("usage: axl check <file.axl>".into());
    }
    let app = axl_compiler::parser::parse_file(Path::new(file))?;
    let entity_count = app.entities.len();
    let api_count = app.apis.len();
    let ui_count = app.ui.len();
    axl_compiler::analyzer::analyze(app)?;
    println!("AXL application is valid: {entity_count} entities, {api_count} APIs, {ui_count} UI components");
    Ok(())
}

struct RuntimeArgs {
    memory_path: Option<PathBuf>,
    max_steps: usize,
    max_output_bytes: usize,
    max_value_bytes: usize,
    max_value_nodes: usize,
    max_value_depth: usize,
    max_tool_calls: usize,
    max_memory_ops: usize,
    max_function_depth: usize,
    scope: String,
}

fn parse_runtime_args(args: &[String]) -> Result<(RuntimeArgs, Vec<String>), Box<dyn std::error::Error>> {
    let mut rt = RuntimeArgs {
        memory_path: None, max_steps: 10_000, max_output_bytes: 1_000_000,
        max_value_bytes: 1_000_000, max_value_nodes: 100_000, max_value_depth: 256,
        max_tool_calls: 100, max_memory_ops: 1_000, max_function_depth: 256,
        scope: "session:default".into(),
    };
    let mut remaining = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--memory" if i + 1 < args.len() => { rt.memory_path = Some(PathBuf::from(&args[i+1])); i += 2; }
            "--max-steps" if i + 1 < args.len() => { rt.max_steps = args[i+1].parse()?; i += 2; }
            "--max-output-bytes" if i + 1 < args.len() => { rt.max_output_bytes = args[i+1].parse()?; i += 2; }
            "--max-value-bytes" if i + 1 < args.len() => { rt.max_value_bytes = args[i+1].parse()?; i += 2; }
            "--max-value-nodes" if i + 1 < args.len() => { rt.max_value_nodes = args[i+1].parse()?; i += 2; }
            "--max-value-depth" if i + 1 < args.len() => { rt.max_value_depth = args[i+1].parse()?; i += 2; }
            "--max-tool-calls" if i + 1 < args.len() => { rt.max_tool_calls = args[i+1].parse()?; i += 2; }
            "--max-memory-ops" if i + 1 < args.len() => { rt.max_memory_ops = args[i+1].parse()?; i += 2; }
            "--max-function-depth" if i + 1 < args.len() => { rt.max_function_depth = args[i+1].parse()?; i += 2; }
            "--scope" if i + 1 < args.len() => { rt.scope = args[i+1].clone(); i += 2; }
            _ => { remaining.push(args[i].clone()); i += 1; }
        }
    }
    Ok((rt, remaining))
}

fn load_program_from_source(path: &Path) -> Result<axl_core::Program, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)?;
    if axl_core::is_compact_source(&source) {
        Ok(axl_core::compiler::compile_file(path)?)
    } else {
        Err(format!("only compact source is supported in Rust runtime; use 'axl pack' to convert").into())
    }
}

fn execute_program(program: &axl_core::Program, rt: &RuntimeArgs) -> Result<i32, Box<dyn std::error::Error>> {
    axl_core::typecheck(program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let memory: Arc<Mutex<dyn axl_core::MemoryStore>> = if let Some(ref path) = rt.memory_path {
        Arc::new(Mutex::new(axl_core::SQLiteMemoryStore::open(path)?))
    } else {
        Arc::new(Mutex::new(axl_core::InMemoryStore::new()))
    };
    let config = axl_core::InterpreterConfig {
        max_steps: rt.max_steps,
        max_output_bytes: rt.max_output_bytes,
        max_value_bytes: rt.max_value_bytes,
        max_value_nodes: rt.max_value_nodes,
        max_value_depth: rt.max_value_depth,
        max_tool_calls: rt.max_tool_calls,
        max_memory_ops: rt.max_memory_ops,
        max_function_depth: rt.max_function_depth,
        scope: rt.scope.clone(),
    };
    let result = axl_core::run_program(program, vec![], memory, config, None)?;
    for line in &result.output {
        println!("{}", axl_core::render_value(line)?);
    }
    Ok(0)
}

fn run_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let (rt, remaining) = parse_runtime_args(args)?;
    let file = remaining.first().ok_or("usage: axl run <file.axl>")?;
    let program = load_program_from_source(Path::new(file))?;
    let code = execute_program(&program, &rt)?;
    std::process::exit(code);
}

fn compile_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = None;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" if i + 1 < args.len() => { output = Some(PathBuf::from(&args[i+1])); i += 2; }
            _ => { file = Some(PathBuf::from(&args[i])); i += 1; }
        }
    }
    let file = file.ok_or("usage: axl compile <file.axl> -o <output.json>")?;
    let output = output.ok_or("usage: axl compile <file.axl> -o <output.json>")?;
    let program = load_program_from_source(&file)?;
    axl_core::validate(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::typecheck(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let json = axl_core::program_to_json(&program)?;
    fs::write(&output, format!("{json}\n"))?;
    Ok(())
}

fn exec_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let (rt, remaining) = parse_runtime_args(args)?;
    let file = remaining.first().ok_or("usage: axl exec <file.json>")?;
    let payload = fs::read_to_string(file)?;
    let program = axl_core::program_from_json(&payload)?;
    let code = execute_program(&program, &rt)?;
    std::process::exit(code);
}

fn pack_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = None;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" if i + 1 < args.len() => { output = Some(PathBuf::from(&args[i+1])); i += 2; }
            _ => { file = Some(PathBuf::from(&args[i])); i += 1; }
        }
    }
    let file = file.ok_or("usage: axl pack <file.axl> -o <output.axl>")?;
    let output = output.ok_or("usage: axl pack <file.axl> -o <output.axl>")?;
    let program = load_program_from_source(&file)?;
    axl_core::validate(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::typecheck(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let compact = axl_core::format_compact(&program, 100)?;
    fs::write(&output, format!("{compact}\n"))?;
    Ok(())
}

fn fmt_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = None;
    let mut output = None;
    let mut width = 100usize;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--width" if i + 1 < args.len() => {
                width = args[i + 1].parse()?;
                i += 2;
            }
            "--check" => {
                check = true;
                i += 1;
            }
            "-o" | "--output" if i + 1 < args.len() => {
                output = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            value if value.starts_with('-') => return Err(format!("unknown fmt option '{value}'").into()),
            value => {
                if file.is_some() {
                    return Err("usage: axl fmt <file.axl> [--width <n>] [--check] [-o <file>]".into());
                }
                file = Some(PathBuf::from(value));
                i += 1;
            }
        }
    }

    let file = file.ok_or("usage: axl fmt <file.axl> [--width <n>] [--check] [-o <file>]")?;
    let source = fs::read_to_string(&file)?;
    if !axl_core::is_compact_source(&source) {
        return Err("axl fmt currently formats Compact Source files".into());
    }
    let program = axl_core::parse_compact(&source)?;
    let formatted = format!("{}\n", axl_core::format_compact(&program, width)?);

    if check {
        if source == formatted {
            println!("{} is formatted", file.display());
            return Ok(());
        }
        return Err(format!("{} is not formatted", file.display()).into());
    }

    let destination = output.unwrap_or(file);
    fs::write(&destination, formatted)?;
    println!("formatted {}", destination.display());
    Ok(())
}

fn build_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = None;
    let mut file = None;
    let mut target = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" if i + 1 < args.len() => { target = Some(args[i+1].clone()); i += 2; }
            "-o" | "--output" if i + 1 < args.len() => { output = Some(PathBuf::from(&args[i+1])); i += 2; }
            _ => { file = Some(PathBuf::from(&args[i])); i += 1; }
        }
    }
    let file = file.ok_or("usage: axl build <file.axl> [-o <dir>] [--target app|web]")?;
    let output = output.unwrap_or_else(|| {
        let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
        PathBuf::from("build").join(name)
    });

    if target.as_deref().is_none() || target.as_deref() == Some("app") {
        axl_compiler::compile_application(&file, &output)?;
        println!("AXL application built in {}", output.display());
        println!("  backend:  {}", output.join("backend").display());
        println!("  frontend: {}", output.join("frontend").display());
        return Ok(());
    }
    if target.as_deref() != Some("web") {
        return Err("supported targets are 'app' and 'web'".into());
    }
    let program = load_program_from_source(&file)?;
    axl_core::validate(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::typecheck(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::build_web(&program, &output)?;
    Ok(())
}

fn serve_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut port: u16 = 8000;
    let mut output = None;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" if i + 1 < args.len() => { port = args[i+1].parse()?; i += 2; }
            "-o" | "--output" if i + 1 < args.len() => { output = Some(PathBuf::from(&args[i+1])); i += 2; }
            _ => { file = Some(PathBuf::from(&args[i])); i += 1; }
        }
    }
    let file = file.ok_or("usage: axl serve <file.axl> [--port <port>] [-o <dir>]")?;
    let out_dir = output.unwrap_or_else(|| {
        let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
        PathBuf::from("build").join(format!("{name}-rs"))
    });

    // Initial build
    let source = fs::read_to_string(&file)?;
    let program = if axl_core::is_compact_source(&source) {
        axl_core::parse_compact(&source)?
    } else {
        return Err("only compact source is supported".into());
    };
    axl_core::validate(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::typecheck(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::build_web(&program, &out_dir)?;

    let mut last_modified = fs::metadata(&file)?.modified()?;
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("axl serve: http://localhost:{port}");
    println!("source: {}", file.display());

    for incoming in listener.incoming() {
        let mut stream = incoming?;
        let current_modified = fs::metadata(&file)?.modified()?;
        if current_modified > last_modified {
            match serve_rebuild(&file, &out_dir) {
                Ok(()) => { last_modified = current_modified; println!("rebuilt: {}", file.display()); }
                Err(e) => eprintln!("rebuild failed: {e}"),
            }
        }
        serve_file(&mut stream, &out_dir)?;
    }
    Ok(())
}

fn serve_rebuild(file: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file)?;
    let program = axl_core::parse_compact(&source)?;
    axl_core::validate(&program).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    axl_core::build_web(&program, output)?;
    Ok(())
}

fn serve_file(stream: &mut TcpStream, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = [0_u8; 4096];
    let size = stream.read(&mut request)?;
    let first_line = String::from_utf8_lossy(&request[..size]);
    let target = first_line.lines().next().and_then(|l| l.split_whitespace().nth(1)).unwrap_or("/");
    let name = match target.split('?').next().unwrap_or("/") {
        "/" | "/index.html" => "index.html",
        "/ax-ui.css" => "ax-ui.css",
        "/ax-ui.js" => "ax-ui.js",
        _ => return respond(stream, "404 Not Found", "text/plain", b"Not found"),
    };
    let body = fs::read(root.join(name))?;
    let content_type = match name.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        _ => "text/html; charset=utf-8",
    };
    respond(stream, "200 OK", content_type, &body)
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len())?;
    stream.write_all(body)?;
    Ok(())
}
