import re
from collections.abc import Callable
from dataclasses import dataclass
from datetime import UTC, datetime

from .ir import Value

Effect = str


@dataclass(frozen=True)
class Tool:
    name: str
    handler: Callable[..., Value]
    effect: Effect = "read"
    approval: bool = False


@dataclass(frozen=True)
class ApprovalRequest:
    tool: str
    arguments: tuple[Value, ...]
    effect: Effect


@dataclass(frozen=True)
class AuditEvent:
    timestamp: str
    tool: str
    arguments: tuple[Value, ...]
    effect: Effect
    decision: str

    @classmethod
    def create(cls, request: ApprovalRequest, decision: str):
        return cls(
            timestamp=datetime.now(UTC).isoformat(),
            tool=request.tool,
            arguments=request.arguments,
            effect=request.effect,
            decision=decision,
        )


class ApprovalRequired(Exception):
    pass


_RESERVED = {
    "agent",
    "call",
    "else",
    "emit",
    "end",
    "false",
    "forget",
    "if",
    "let",
    "memory",
    "meta",
    "recall",
    "run",
    "true",
    "uses",
    "while",
    "workflow",
}


def validate_tool(tool: Tool) -> None:
    if not isinstance(tool.name, str) or not re.fullmatch(
        r"[A-Za-z_][A-Za-z0-9_]*", tool.name
    ):
        raise ValueError("invalid tool name")
    if tool.name in _RESERVED:
        raise ValueError(f"reserved tool name '{tool.name}'")
    if not callable(tool.handler):
        raise ValueError(f"tool '{tool.name}' handler must be callable")  # noqa: TRY004
    if not isinstance(tool.effect, str) or not tool.effect:
        raise ValueError(f"tool '{tool.name}' effect must be a non-empty string")
    if type(tool.approval) is not bool:
        raise ValueError(f"tool '{tool.name}' approval must be boolean")
