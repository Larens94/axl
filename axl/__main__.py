import argparse
from pathlib import Path

from .interpreter import Interpreter
from .parser import parse


def main() -> int:
    parser = argparse.ArgumentParser(prog="axl")
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run", help="execute an AXL source file")
    run.add_argument("file", type=Path)
    args = parser.parse_args()

    if args.command == "run":
        result = Interpreter().run(parse(args.file.read_text(encoding="utf-8")))
        for line in result.output:
            print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
