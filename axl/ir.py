from dataclasses import dataclass
from typing import TypeAlias


@dataclass(frozen=True, eq=False)
class MapValue:
    entries: tuple[tuple["Value", "Value"], ...]

    def __eq__(self, other):
        if not isinstance(other, MapValue):
            return NotImplemented
        return _map_items(self.entries) == _map_items(other.entries)

    def __hash__(self):
        return hash(frozenset(_map_items(self.entries)))


def _map_items(entries):
    return frozenset(((type(key), key), value) for key, value in entries)


Value: TypeAlias = str | int | bool | tuple["Value", ...] | MapValue


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
class FunctionCall:
    name: str
    arguments: tuple["Expression", ...]


@dataclass(frozen=True)
class ListExpression:
    items: tuple["Expression", ...]


@dataclass(frozen=True)
class MapExpression:
    entries: tuple[tuple["Expression", "Expression"], ...]


@dataclass(frozen=True)
class Binary:
    left: "Expression"
    operator: str
    right: "Expression"


Expression: TypeAlias = (
    Literal
    | Variable
    | Recall
    | ToolCall
    | FunctionCall
    | ListExpression
    | MapExpression
    | Binary
)


@dataclass(frozen=True)
class MemoryWrite:
    key: str
    value: Expression
    confidence: int = 100
    ttl_seconds: int | None = None
    source: str = "program"


@dataclass(frozen=True)
class Forget:
    key: str


@dataclass(frozen=True)
class Let:
    target: str
    value: Expression
    type_name: str | None = None


@dataclass(frozen=True)
class Return:
    value: Expression


@dataclass(frozen=True)
class Emit:
    value: Expression


@dataclass(frozen=True)
class If:
    condition: Expression
    body: tuple["Instruction", ...]
    else_body: tuple["Instruction", ...] = ()


@dataclass(frozen=True)
class While:
    condition: Expression
    body: tuple["Instruction", ...]


@dataclass(frozen=True)
class Agent:
    name: str
    tools: tuple[str, ...]
    body: tuple["Instruction", ...]


@dataclass(frozen=True)
class Workflow:
    name: str
    body: tuple["Instruction", ...]


@dataclass(frozen=True)
class Run:
    name: str


@dataclass(frozen=True)
class Parameter:
    name: str
    type_name: str


@dataclass(frozen=True)
class Function:
    name: str
    parameters: tuple[Parameter, ...]
    return_type: str
    body: tuple["Instruction", ...]


Instruction: TypeAlias = (
    MemoryWrite
    | Forget
    | Let
    | Return
    | Emit
    | If
    | While
    | Agent
    | Workflow
    | Run
    | Function
)


@dataclass(frozen=True)
class Program:
    instructions: tuple[Instruction, ...]
