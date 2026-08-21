import os
import re
from dataclasses import dataclass, fields, is_dataclass, replace
from pathlib import Path

from .compact import CompactParseError, is_compact_source, split_compact_frames
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
_MAX_IMPORT_DEPTH = 256
_MAX_MODULES = 1_024
_MAX_TOTAL_SOURCE_BYTES = 4 * 1024 * 1024


@dataclass
class _CompileContext:
    root: Path
    modules: int = 0
    source_bytes: int = 0


def compile_file(path: str | Path, *, module_root: str | Path | None = None) -> Program:
    entry = Path(path).resolve()
    root = Path(module_root).resolve() if module_root is not None else entry.parent
    return _compile_file(entry, (), _CompileContext(root))


def _compile_file(
    path: Path, stack: tuple[Path, ...], context: _CompileContext
) -> Program:
    if len(stack) > _MAX_IMPORT_DEPTH:
        raise CompileError(f"import depth exceeds {_MAX_IMPORT_DEPTH}")
    if not path.is_relative_to(context.root):
        raise CompileError(f"module '{path.name}' is outside module root")
    if path.suffix != ".axl":
        raise CompileError("modules must use the .axl extension")
    if path in stack:
        chain = " -> ".join(item.name for item in (*stack, path))
        raise CompileError(f"cyclic module import: {chain}")
    context.modules += 1
    if context.modules > _MAX_MODULES:
        raise CompileError(f"module count exceeds {_MAX_MODULES}")
    remaining = _MAX_TOTAL_SOURCE_BYTES - context.source_bytes
    try:
        with path.open("rb") as stream:
            size = os.fstat(stream.fileno()).st_size
            if size > remaining:
                raise CompileError(
                    f"aggregate module source exceeds {_MAX_TOTAL_SOURCE_BYTES} bytes"
                )
            data = stream.read(remaining + 1)
    except CompileError:
        raise
    except OSError as error:
        raise CompileError(f"cannot read module '{path}': {error}") from error
    if len(data) > remaining:
        raise CompileError(
            f"aggregate module source exceeds {_MAX_TOTAL_SOURCE_BYTES} bytes"
        )
    try:
        source = data.decode("utf-8")
    except UnicodeError as error:
        raise CompileError(f"cannot read module '{path}': {error}") from error
    context.source_bytes += len(data)

    imports: list[tuple[str, Path]] = []
    aliases: set[str] = set()
    try:
        if is_compact_source(source):
            local_source = _compact_imports(
                source, path, imports, aliases, context.root
            )
        else:
            local_source = _legacy_imports(source, path, imports, aliases, context.root)
        local_program = parse(local_source)
    except (CompactParseError, ParseError) as error:
        raise CompileError(f"{path}: {error}") from error

    imported_instructions = []
    for alias, imported_path in imports:
        imported = _compile_file(imported_path, (*stack, path), context)
        if not all(
            isinstance(instruction, Function) for instruction in imported.instructions
        ):
            raise CompileError(
                f"module '{imported_path}' may only export function declarations"
            )
        imported_instructions.extend(_namespace(imported, alias).instructions)

    return Program((*imported_instructions, *local_program.instructions))


def _compact_imports(
    source: str,
    path: Path,
    imports: list[tuple[str, Path]],
    aliases: set[str],
    module_root: Path,
) -> str:
    frames = split_compact_frames(source)
    local_frames = [frames[0]]
    depth = 0
    for index, frame in enumerate(frames[1:], 1):
        opcode = frame.split("|", 1)[0].strip()
        if opcode == "1":
            if depth:
                raise CompileError(
                    f"{path}:frame {index}: compact import must be top-level"
                )
            fields = [field.strip() for field in frame.split("|")]
            if len(fields) != 3:
                raise CompileError(f"{path}:frame {index}: invalid compact import")
            alias, relative_path = fields[1:]
            _add_import(
                alias, relative_path, path, index, imports, aliases, module_root
            )
            continue
        local_frames.append(frame)
        if opcode in {"30", "32", "40", "50", "51", "61"}:
            depth += 1
        elif opcode == "99" and depth:
            depth -= 1
    return ";".join(local_frames)


def _legacy_imports(
    source: str,
    path: Path,
    imports: list[tuple[str, Path]],
    aliases: set[str],
    module_root: Path,
) -> str:
    local_lines: list[str] = []
    depth = 0
    for number, line in enumerate(source.splitlines(), 1):
        stripped = line.strip()
        if not stripped.startswith("import "):
            local_lines.append(line)
            if stripped == "end" and depth:
                depth -= 1
            elif stripped.startswith(("if ", "while ", "agent ", "fn ", "workflow ")):
                depth += 1
            continue
        if depth:
            raise CompileError(f"{path}:{number}: import must be top-level")
        match = _IMPORT.fullmatch(line)
        if match is None:
            raise CompileError(f"{path}:{number}: invalid import declaration")
        alias, relative_path = match.groups()
        _add_import(alias, relative_path, path, number, imports, aliases, module_root)
        local_lines.append("")
    return "\n".join(local_lines)


def _add_import(
    alias: str,
    relative_path: str,
    path: Path,
    position: int,
    imports: list[tuple[str, Path]],
    aliases: set[str],
    module_root: Path,
) -> None:
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", alias) or alias in _RESERVED:
        raise CompileError(f"{path}:{position}: invalid import alias '{alias}'")
    if alias in aliases:
        raise CompileError(f"duplicate import alias '{alias}'")
    if not relative_path or "|" in relative_path or ";" in relative_path:
        raise CompileError(f"{path}:{position}: invalid import path")
    candidate = Path(relative_path)
    if candidate.is_absolute():
        raise CompileError(f"{path}:{position}: import path must be relative")
    if ".." in candidate.parts:
        raise CompileError(f"{path}:{position}: import path escapes module root")
    imported_path = (path.parent / candidate).resolve()
    if not imported_path.is_relative_to(module_root):
        raise CompileError(f"{path}:{position}: import path escapes module root")
    if imported_path.suffix != ".axl":
        raise CompileError(f"{path}:{position}: imported module must use .axl")
    aliases.add(alias)
    imports.append((alias, imported_path))


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
