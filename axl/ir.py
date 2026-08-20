from dataclasses import dataclass
from typing import TypeAlias

Value: TypeAlias = str | int | bool


@dataclass(frozen=True)
class Literal:
    value: Value


@dataclass(frozen=True)
class Variable:
    name: str


@dataclass(frozen=True)
class Recall:
    key: str


@dataclass(frozen=True)
class ToolCall:
    name: str
    arguments: tuple["Expression", ...]


@dataclass(frozen=True)
class Binary:
    left: "Expression"
    operator: str
    right: "Expression"


Expression: TypeAlias = Literal | Variable | Recall | ToolCall | Binary


@dataclass(frozen=True)
class MemoryWrite:
    key: str
    value: Expression


@dataclass(frozen=True)
class Let:
    target: str
    value: Expression


@dataclass(frozen=True)
class Emit:
    value: Expression


@dataclass(frozen=True)
class If:
    condition: Expression
    body: tuple["Instruction", ...]
    else_body: tuple["Instruction", ...] = ()


Instruction: TypeAlias = MemoryWrite | Let | Emit | If


@dataclass(frozen=True)
class Program:
    instructions: tuple[Instruction, ...]
