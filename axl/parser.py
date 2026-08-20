import re

from .ir import Emit, MemoryRecall, MemoryWrite, Program


class ParseError(ValueError):
    pass


_MEMORY = re.compile(r'^memory\s+([A-Za-z_]\w*)\s*=\s*"([^"\\]*)"$')
_RECALL = re.compile(r"^let\s+([A-Za-z_]\w*)\s*=\s*recall\s+([A-Za-z_]\w*)$")
_EMIT = re.compile(r"^emit\s+([A-Za-z_]\w*)$")


def parse(source: str) -> Program:
    instructions = []
    for number, raw_line in enumerate(source.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if match := _MEMORY.fullmatch(line):
            instructions.append(MemoryWrite(*match.groups()))
        elif match := _RECALL.fullmatch(line):
            instructions.append(MemoryRecall(*match.groups()))
        elif match := _EMIT.fullmatch(line):
            instructions.append(Emit(*match.groups()))
        else:
            raise ParseError(f"line {number}: invalid instruction: {line}")
    return Program(tuple(instructions))
