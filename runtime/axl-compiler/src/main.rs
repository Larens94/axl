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
        "check" | "ir" | "pack" | "fmt" | "blocks" | "eval" | "serve" | "experiment" | "unpack" => {
            run(&args[1..])
        }
        _ => {
            usage();
            bail!("invalid command")
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    let command = args.first().map(String::as_str).unwrap_or_default();
    let Some(input) = args.get(1) else {
        usage();
        bail!("{command} requires an input file")
    };
    if command == "unpack" {
        let packed = std::fs::read_to_string(input)
            .with_context(|| format!("cannot read packed IR '{input}'"))?;
        let graph = next::packed::decode(&packed)?;
        println!("{}", serde_json::to_string_pretty(&graph)?);
        return Ok(());
    }

    let compilation = match next::compile_file(Path::new(input))? {
        Ok(compilation) => compilation,
        Err(diagnostics) => {
            if args.iter().any(|argument| argument == "--json") {
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
        "check" => {
            if args.iter().any(|argument| argument == "--json") {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "schema": compilation.graph.schema,
                        "app": compilation.graph.app,
                        "nodes": compilation.graph.nodes.len(),
                        "edges": compilation.graph.edges.len(),
                    })
                );
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
        "eval" => {
            let flow = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("eval requires a flow name"))?;
            let input_path = args
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("eval requires a JSON input file"))?;
            let input = std::fs::read_to_string(input_path)
                .with_context(|| format!("cannot read eval input '{input_path}'"))?;
            let input: serde_json::Value = serde_json::from_str(&input)
                .with_context(|| format!("invalid JSON input '{input_path}'"))?;
            let result = next::runtime::evaluate_flow(&compilation.graph, flow, input)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "serve" => {
            let address = args.get(2).map(String::as_str).unwrap_or("127.0.0.1:8080");
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            runtime.block_on(next::http::serve(compilation.graph, address))?;
        }
        "experiment" => {
            let Some(output) = args.get(2) else {
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
        "Usage:\n  axl-compiler check <input.axl> [--json]\n  axl-compiler ir <input.axl>\n  axl-compiler pack <input.axl> [--matrix]\n  axl-compiler fmt <input.axl>\n  axl-compiler blocks <input.axl>\n  axl-compiler eval <input.axl> <flow> <input.json>\n  axl-compiler serve <input.axl> [address]\n  axl-compiler experiment <input.axl> <output-dir>\n  axl-compiler unpack <packed.axl>"
    );
}
