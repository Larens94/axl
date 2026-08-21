import json
from dataclasses import fields, is_dataclass
from typing import Any

from . import ir
from .ir import Program
from .validation import validate

IR_VERSION = "1.2"
SUPPORTED_IR_VERSIONS = {"1.0", "1.1", IR_VERSION}
MAX_IR_BYTES = 2_000_000

_NODE_TYPES = {
    cls.__name__: cls
    for cls in (
        ir.Literal,
        ir.Variable,
        ir.Recall,
        ir.ToolCall,
        ir.FunctionCall,
        ir.ListExpression,
        ir.Binary,
        ir.MemoryWrite,
        ir.Forget,
        ir.Let,
        ir.Return,
        ir.Emit,
        ir.If,
        ir.While,
        ir.Agent,
        ir.Workflow,
        ir.Run,
        ir.Parameter,
        ir.Function,
        ir.Program,
    )
}


def _encode(value: Any) -> Any:
    if is_dataclass(value):
        return {
            "type": value.__class__.__name__,
            **{
                field.name: _encode(getattr(value, field.name))
                for field in fields(value)
            },
        }
    if isinstance(value, tuple):
        return [_encode(item) for item in value]
    if isinstance(value, (str, int, bool)) or value is None:
        return value
    raise TypeError(f"unsupported IR value: {type(value).__name__}")


def _decode(value: Any) -> Any:
    if isinstance(value, list):
        return tuple(_decode(item) for item in value)
    if not isinstance(value, dict):
        return value
    node_type = value.get("type")
    if node_type not in _NODE_TYPES:
        raise ValueError(f"unknown IR node '{node_type}'")
    cls = _NODE_TYPES[node_type]
    expected = {field.name for field in fields(cls)}
    actual = set(value) - {"type"}
    if actual != expected:
        raise ValueError(f"invalid fields for {node_type}")
    return cls(**{name: _decode(value[name]) for name in expected})


def program_to_document(program: Program) -> dict[str, Any]:
    validate(program)
    if any(isinstance(item, ir.UiView | ir.Annotation) for item in program.instructions):
        raise ValueError("AX-UI experimental nodes are not available in AX-IR 1.2")
    return {"ir_version": IR_VERSION, "program": _encode(program)}


def program_to_json(program: Program, *, indent: int | None = 2) -> str:
    return json.dumps(program_to_document(program), ensure_ascii=False, indent=indent)


def program_from_document(document: dict[str, Any]) -> Program:
    if set(document) != {"ir_version", "program"}:
        raise ValueError("invalid IR envelope fields")
    version = document.get("ir_version")
    if not isinstance(version, str):
        raise ValueError("IR version must be a string")  # noqa: TRY004
    if version not in SUPPORTED_IR_VERSIONS:
        raise ValueError(f"unsupported IR version '{version}'")
    payload = document.get("program")
    try:
        _require_version_features(payload, version)
        if version == "1.0":
            payload = _upgrade_1_0(payload)
        program = _decode(payload)
    except RecursionError as error:
        raise ValueError("IR nesting is too deep") from error
    if not isinstance(program, Program):
        raise ValueError("IR root must be Program")  # noqa: TRY004
    validate(program)
    return program


def _require_version_features(payload: Any, version: str) -> None:
    if version == "1.2":
        return
    stack = [payload]
    while stack:
        value = stack.pop()
        if isinstance(value, list):
            stack.extend(value)
            continue
        if not isinstance(value, dict):
            continue
        if value.get("type") == "ListExpression":
            raise ValueError("ListExpression requires AX-IR 1.2")
        for key in ("type_name", "return_type"):
            type_name = value.get(key)
            if isinstance(type_name, str) and type_name.startswith("list<"):
                raise ValueError("list types require AX-IR 1.2")
        stack.extend(value.values())


def _upgrade_1_0(value: Any) -> Any:
    if isinstance(value, list):
        return [_upgrade_1_0(item) for item in value]
    if not isinstance(value, dict):
        return value
    upgraded = {key: _upgrade_1_0(item) for key, item in value.items()}
    if upgraded.get("type") == "Let" and "type_name" not in upgraded:
        upgraded["type_name"] = None
    return upgraded


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key '{key}'")
        result[key] = value
    return result


def program_from_json(payload: str) -> Program:
    try:
        payload_size = len(payload.encode("utf-8"))
    except UnicodeEncodeError as error:
        raise ValueError("invalid Unicode payload") from error
    if payload_size > MAX_IR_BYTES:
        raise ValueError(f"IR payload exceeds {MAX_IR_BYTES} bytes")
    try:
        document = json.loads(payload, object_pairs_hook=_unique_object)
    except RecursionError as error:
        raise ValueError("IR nesting is too deep") from error
    if not isinstance(document, dict):
        raise ValueError("IR document must be an object")  # noqa: TRY004
    return program_from_document(document)
