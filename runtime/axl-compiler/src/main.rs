use anyhow::{Context, Result, bail};
use axl_compiler::next;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "check" | "diagnose" | "ir" | "pack" | "fmt" | "blocks" | "eval" | "tick" | "serve"
        | "ui" | "render" | "experiment" | "unpack" => run(&args[1..]),
        _ => {
            usage();
            bail!("invalid command")
        }
    }
}

fn positional_args(args: &[String]) -> Vec<&String> {
    args.iter()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .collect()
}

fn read_json_input(token: &str) -> Result<serde_json::Value> {
    if token == "null" {
        return Ok(serde_json::Value::Null);
    }
    let input = std::fs::read_to_string(token)
        .with_context(|| format!("cannot read JSON input '{token}'"))?;
    serde_json::from_str(&input).with_context(|| format!("invalid JSON input '{token}'"))
}

fn run(args: &[String]) -> Result<()> {
    let command = args.first().map(String::as_str).unwrap_or_default();
    let json_output = args.iter().any(|argument| argument == "--json");
    let positional = positional_args(args);
    let Some(input) = positional.first() else {
        usage();
        bail!("{command} requires an input file")
    };
    let input_path = Path::new(input.as_str());
    if command == "unpack" {
        let packed = std::fs::read_to_string(input)
            .with_context(|| format!("cannot read packed IR '{input}'"))?;
        let graph = next::packed::decode(&packed)?;
        println!("{}", serde_json::to_string_pretty(&graph)?);
        return Ok(());
    }

    let compilation = match next::compile_file(input_path)? {
        Ok(compilation) => compilation,
        Err(diagnostics) => {
            if command == "check" || command == "diagnose" {
                if json_output {
                    let report =
                        next::diagnostic::CheckReport::failure(Some(input_path), diagnostics);
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    for diagnostic in diagnostics {
                        eprintln!("{}", diagnostic.human());
                    }
                }
            } else if json_output {
                eprintln!("{}", serde_json::to_string_pretty(&diagnostics)?);
            } else {
                for diagnostic in diagnostics {
                    eprintln!("{}", diagnostic.human());
                }
            }
            std::process::exit(1);
        }
    };

    match command {
        "check" | "diagnose" => {
            if json_output {
                let report = next::diagnostic::CheckReport::success(
                    Some(input_path),
                    &compilation.graph.app,
                    &compilation.graph.schema,
                    compilation.graph.nodes.len(),
                    compilation.graph.edges.len(),
                );
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "AXL 4 OK: {} ({} nodes, {} edges)",
                    compilation.graph.app,
                    compilation.graph.nodes.len(),
                    compilation.graph.edges.len()
                );
            }
        }
        "ir" => println!("{}", serde_json::to_string_pretty(&compilation.graph)?),
        "pack" => {
            if args.iter().any(|argument| argument == "--matrix") {
                println!("{}", compilation.matrix);
            } else {
                println!("{}", compilation.packed);
            }
        }
        "fmt" => print!("{}", compilation.source),
        "blocks" => println!(
            "{}",
            serde_json::to_string_pretty(&next::targets::open_block_manifest(&compilation.graph))?
        ),
        "ui" => println!(
            "{}",
            serde_json::to_string_pretty(&next::ui::ui_manifest(&compilation.graph))?
        ),
        "eval" => {
            let flow = positional
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("eval requires a flow name"))?;
            let input_path = positional
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("eval requires a JSON input file or null"))?;
            let input = read_json_input(input_path)?;
            let result = next::runtime::evaluate_flow(&compilation.graph, flow, input)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "tick" => {
            let mut runtime = next::runtime::BuiltinRuntime::new()?;
            let executed = next::runtime::run_due_jobs(&compilation.graph, &mut runtime)?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "executed": executed,
                })
            );
        }
        "serve" => {
            let address = positional
                .get(1)
                .map(|address| address.as_str())
                .unwrap_or("127.0.0.1:8080");
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            runtime.block_on(next::http::serve(compilation.graph, address))?;
        }
        "render" => {
            let page = positional
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("render requires a page path"))?;
            let input_path = positional
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("render requires a JSON input file or null"))?;
            let input = read_json_input(input_path)?;
            let mut headers = std::collections::BTreeMap::new();
            if let Some(index) = args.iter().position(|argument| argument == "--cookie")
                && let Some(cookie) = args.get(index + 1)
            {
                headers.insert("cookie".into(), cookie.clone());
            }
            let mut runtime = next::runtime::BuiltinRuntime::new().map_err(|error| {
                anyhow::anyhow!("provider_runtime_initialization_failed: {error}")
            })?;
            let rendered = next::ui::render_page_with_runtime(
                &compilation.graph,
                &mut runtime,
                page,
                input,
                &headers,
            )
            .map_err(|error| anyhow::anyhow!(error))?;
            if args.iter().any(|argument| argument == "--json") {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "path": rendered.path,
                        "flow": rendered.flow,
                        "output": rendered.output_type,
                        "data": rendered.data,
                    })
                );
            } else {
                print!("{}", rendered.html);
            }
        }
        "experiment" => {
            let Some(output) = positional.get(1) else {
                bail!("experiment requires an output directory")
            };
            let output = Path::new(output);
            std::fs::create_dir_all(output)?;
            std::fs::write(output.join("app.axl"), compilation.source)?;
            std::fs::write(
                output.join("app.axir.json"),
                serde_json::to_string_pretty(&compilation.graph)?,
            )?;
            std::fs::write(output.join("app.packed.axl"), compilation.matrix)?;
            next::targets::generate(&compilation.graph, &output.join("targets"))?;
            println!("AXL 4 experiment written to {}", output.display());
        }
        _ => bail!("unknown AXL 4 command '{command}'"),
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "Usage:\n  axl-compiler check|diagnose <input.axl> [--json]\n  axl-compiler check|diagnose [--json] <input.axl>\n  axl-compiler ir <input.axl>\n  axl-compiler pack <input.axl> [--matrix]\n  axl-compiler fmt <input.axl>\n  axl-compiler blocks <input.axl>\n  axl-compiler ui <input.axl>\n  axl-compiler eval <input.axl> <flow> <input.json|null>\n  axl-compiler render <input.axl> <page-path> <input.json|null> [--cookie 'sid=...'] [--json]\n  axl-compiler tick <input.axl>\n  axl-compiler serve <input.axl> [address]\n  axl-compiler experiment <input.axl> <output-dir>\n  axl-compiler unpack <packed.axl>\n\nFlags such as --json may appear before or after positional arguments."
    );
}
