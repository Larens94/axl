import argparse
from pathlib import Path

from .interpreter import Interpreter
from .memory import SQLiteMemoryStore
from .parser import parse


def main() -> int:
    parser = argparse.ArgumentParser(prog="axl")
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run", help="execute an AXL source file")
    run.add_argument("file", type=Path)
    run.add_argument("--memory", type=Path, help="persistent SQLite memory database")
    run.add_argument("--max-steps", type=int, default=10_000, help="execution budget")
    args = parser.parse_args()

    if args.command == "run":
        store = SQLiteMemoryStore(args.memory) if args.memory else None
        try:
            interpreter = Interpreter(memory_store=store, max_steps=args.max_steps)
            result = interpreter.run(parse(args.file.read_text(encoding="utf-8")))
            for line in result.output:
                print(str(line).lower() if isinstance(line, bool) else line)
        finally:
            if store is not None:
                store.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
