import re
from dataclasses import fields, is_dataclass, replace
from pathlib import Path

from .ir import Function, FunctionCall, Program
from .parser import ParseError, parse


class CompileError(ValueError):
    """Raised when source files cannot be assembled into one AXL program."""


_IMPORT = re.compile(r'^\s*import\s+([A-Za-z_][A-Za-z0-9_]*)\s+from\s+"([^"\n]+)"\s*$')
_RESERVED = {
    "agent",
    "call",
    "else",
    "emit",
    "end",
    "false",
    "fn",
    "forget",
    "if",
    "import",
    "let",
    "memory",
    "meta",
    "recall",
    "return",
    "run",
    "true",
    "uses",
    "while",
    "workflow",
}


def compile_file(path: str | Path) -> Program:
    return _compile_file(Path(path).resolve(), ())


def _compile_file(path: Path, stack: tuple[Path, ...]) -> Program:
    if path in stack:
        chain = " -> ".join(item.name for item in (*stack, path))
        raise CompileError(f"cyclic module import: {chain}")
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as error:
        raise CompileError(f"cannot read module '{path}': {error}") from error

    imports: list[tuple[str, Path]] = []
    aliases: set[str] = set()
    local_lines: list[str] = []
    for number, line in enumerate(source.splitlines(), 1):
        if not line.strip().startswith("import "):
            local_lines.append(line)
            continue
        match = _IMPORT.fullmatch(line)
        if match is None:
            raise CompileError(f"{path}:{number}: invalid import declaration")
        alias, relative_path = match.groups()
        if alias in _RESERVED:
            raise CompileError(f"{path}:{number}: reserved import alias '{alias}'")
        if alias in aliases:
            raise CompileError(f"duplicate import alias '{alias}'")
        aliases.add(alias)
        imports.append((alias, (path.parent / relative_path).resolve()))
        local_lines.append("")

    try:
        local_program = parse("\n".join(local_lines))
    except ParseError as error:
        raise CompileError(f"{path}: {error}") from error

    imported_instructions = []
    for alias, imported_path in imports:
        imported = _compile_file(imported_path, (*stack, path))
        if not all(
            isinstance(instruction, Function) for instruction in imported.instructions
        ):
            raise CompileError(
                f"module '{imported_path}' may only export function declarations"
            )
        imported_instructions.extend(_namespace(imported, alias).instructions)

    return Program((*imported_instructions, *local_program.instructions))


def _namespace(program: Program, alias: str) -> Program:
    names = {
        instruction.name: f"{alias}.{instruction.name}"
        for instruction in program.instructions
        if isinstance(instruction, Function)
    }

    def transform(value):
        if isinstance(value, FunctionCall) and value.name in names:
            value = replace(value, name=names[value.name])
        if isinstance(value, Function) and value.name in names:
            value = replace(value, name=names[value.name])
        if is_dataclass(value):
            updates = {
                field.name: transform(getattr(value, field.name))
                for field in fields(value)
            }
            return replace(value, **updates)
        if isinstance(value, tuple):
            return tuple(transform(item) for item in value)
        return value

    return transform(program)
