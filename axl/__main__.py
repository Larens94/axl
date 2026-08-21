import argparse
import importlib
import sqlite3
import sys
from pathlib import Path

from .compact import program_to_compact
from .compiler import CompileError, compile_file
from .interpreter import Interpreter, RuntimeError, render_value
from .memory import SQLiteMemoryStore
from .parser import ParseError
from .policy import ApprovalRequired, Tool
from .render_web import WebRenderError, render_web
from .serialization import program_from_json, program_to_json
from .typechecker import typecheck
from .validation import validate


def _runtime_options(command: argparse.ArgumentParser) -> None:
    command.add_argument(
        "--memory", type=Path, help="persistent SQLite memory database"
    )
    command.add_argument(
        "--max-steps", type=int, default=10_000, help="instruction/expression budget"
    )
    command.add_argument("--max-output-bytes", type=int, default=1_000_000)
    command.add_argument("--max-value-bytes", type=int, default=1_000_000)
    command.add_argument("--max-value-nodes", type=int, default=100_000)
    command.add_argument("--max-value-depth", type=int, default=256)
    command.add_argument("--max-tool-calls", type=int, default=100)
    command.add_argument("--max-memory-ops", type=int, default=1_000)
    command.add_argument("--max-function-depth", type=int, default=256)
    command.add_argument("--scope", default="session:default", help="memory scope")
    command.add_argument(
        "--plugin", action="append", default=[], help="explicit Python tool plugin"
    )
    command.add_argument(
        "--approve-tool", action="append", default=[], help="pre-approved tool name"
    )


def _load_tools(plugins: list[str]) -> list[Tool]:
    tools: list[Tool] = []
    for specification in plugins:
        module_name, separator, factory_name = specification.partition(":")
        factory = getattr(
            importlib.import_module(module_name), factory_name if separator else "tools"
        )
        provided = factory()
        if not isinstance(provided, (list, tuple)) or not all(
            isinstance(tool, Tool) for tool in provided
        ):
            raise ValueError(f"plugin '{specification}' must return Tool objects")
        tools.extend(provided)
    names = [tool.name for tool in tools]
    if len(names) != len(set(names)):
        raise ValueError("duplicate tool name across plugins")
    return tools


def _execute(program, args) -> int:
    typecheck(program)
    store = SQLiteMemoryStore(args.memory) if args.memory else None
    try:
        approved = set(args.approve_tool)
        result = Interpreter(
            tools=_load_tools(args.plugin),
            memory_store=store,
            max_steps=args.max_steps,
            max_output_bytes=args.max_output_bytes,
            max_value_bytes=args.max_value_bytes,
            max_value_nodes=args.max_value_nodes,
            max_value_depth=args.max_value_depth,
            max_tool_calls=args.max_tool_calls,
            max_memory_ops=args.max_memory_ops,
            max_function_depth=args.max_function_depth,
            scope=args.scope,
            approve=lambda request: request.tool in approved,
        ).run(program)
        for line in result.output:
            print(render_value(line))
    finally:
        if store is not None:
            store.close()
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="axl")
    commands = parser.add_subparsers(dest="command", required=True)

    run = commands.add_parser("run", help="parse and execute AXL source")
    run.add_argument("file", type=Path)
    _runtime_options(run)

    compile_command = commands.add_parser(
        "compile", help="compile source to versioned JSON IR"
    )
    compile_command.add_argument("file", type=Path)
    compile_command.add_argument("-o", "--output", type=Path, required=True)

    pack = commands.add_parser("pack", help="normalize source to canonical compact AXL")
    pack.add_argument("file", type=Path)
    pack.add_argument("-o", "--output", type=Path, required=True)

    execute = commands.add_parser("exec", help="execute versioned JSON IR")
    execute.add_argument("file", type=Path)
    _runtime_options(execute)

    build = commands.add_parser("build", help="build an AXL application target")
    build.add_argument("file", type=Path)
    build.add_argument("--target", choices=("web",), required=True)
    build.add_argument("-o", "--output", type=Path, required=True)

    args = parser.parse_args(argv)
    try:
        if args.command == "compile":
            program = compile_file(args.file)
            validate(program)
            typecheck(program)
            args.output.write_text(program_to_json(program) + "\n", encoding="utf-8")
            return 0
        if args.command == "pack":
            program = compile_file(args.file)
            validate(program)
            typecheck(program)
            args.output.write_text(program_to_compact(program) + "\n", encoding="utf-8")
            return 0
        if args.command == "build":
            program = compile_file(args.file)
            validate(program)
            typecheck(program)
            render_web(program, args.output)
            return 0
        if args.command == "run":
            return _execute(compile_file(args.file), args)
        if args.command == "exec":
            return _execute(
                program_from_json(args.file.read_text(encoding="utf-8")), args
            )
    except (
        OSError,
        sqlite3.Error,
        ImportError,
        AttributeError,
        CompileError,
        ApprovalRequired,
        ParseError,
        RuntimeError,
        WebRenderError,
        ValueError,
    ) as error:
        print(f"axl: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
