use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn main() {
    if let Err(error) = run() {
        eprintln!("axl-rs: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("build") => build_command(&arguments[1..]),
        Some("serve") => serve_command(&arguments[1..]),
        _ => Err(usage().into()),
    }
}

fn usage() -> &'static str {
    "usage:\n  axl-rs build <file.axl> --target web -o <directory>\n  axl-rs serve <file.axl> [--port <port>] [-o <directory>]"
}

fn build_command(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if arguments.len() != 5
        || arguments[1] != "--target"
        || arguments[2] != "web"
        || !matches!(arguments[3].as_str(), "-o" | "--output")
    {
        return Err(usage().into());
    }
    build(&PathBuf::from(&arguments[0]), &PathBuf::from(&arguments[4]))
}

fn serve_command(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let Some(source_arg) = arguments.first() else {
        return Err(usage().into());
    };
    let source_path = PathBuf::from(source_arg);
    let mut port = 8000_u16;
    let mut output = default_output(&source_path);
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--port" if index + 1 < arguments.len() => {
                port = arguments[index + 1].parse()?;
                index += 2;
            }
            "-o" | "--output" if index + 1 < arguments.len() => {
                output = PathBuf::from(&arguments[index + 1]);
                index += 2;
            }
            _ => return Err(usage().into()),
        }
    }

    build(&source_path, &output)?;
    let mut last_modified = modified(&source_path)?;
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("AXL Rust server: http://localhost:{port}");
    println!("source: {}", source_path.display());

    for incoming in listener.incoming() {
        let mut stream = incoming?;
        let current_modified = modified(&source_path)?;
        if current_modified > last_modified {
            match build(&source_path, &output) {
                Ok(()) => {
                    last_modified = current_modified;
                    println!("rebuilt: {}", source_path.display());
                }
                Err(error) => eprintln!("rebuild failed: {error}"),
            }
        }
        serve_file(&mut stream, &output)?;
    }
    Ok(())
}

fn build(source_path: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(source_path)?;
    let program = axl_core::parse(&source)?;
    axl_core::build_web(&program, output)?;
    Ok(())
}

fn default_output(source: &Path) -> PathBuf {
    let name = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("app");
    PathBuf::from("build").join(format!("{name}-rs"))
}

fn modified(path: &Path) -> Result<SystemTime, Box<dyn std::error::Error>> {
    Ok(fs::metadata(path)?.modified()?)
}

fn serve_file(stream: &mut TcpStream, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = [0_u8; 4096];
    let size = stream.read(&mut request)?;
    let first_line = String::from_utf8_lossy(&request[..size]);
    let target = first_line
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
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

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    Ok(())
}
