import re

from .ir import (
    Agent,
    Annotation,
    Binary,
    Emit,
    Expression,
    Forget,
    Function,
    FunctionCall,
    If,
    Instruction,
    Let,
    ListExpression,
    Literal,
    MapExpression,
    MemoryWrite,
    Program,
    Recall,
    Return,
    Run,
    ToolCall,
    UiNode,
    UiView,
    Variable,
    While,
    Workflow,
)
from .type_names import validate_type_name
from .ui_registry import ANNOTATION_KINDS, COMPONENTS


class ValidationError(ValueError):
    pass


INSTRUCTION_TYPES = (
    MemoryWrite,
    Forget,
    Let,
    Return,
    Emit,
    If,
    While,
    Agent,
    Workflow,
    Run,
    Function,
    Annotation,
    UiView,
)
EXPRESSION_TYPES = (
    Literal,
    Variable,
    Recall,
    ToolCall,
    FunctionCall,
    ListExpression,
    MapExpression,
    Binary,
)
OPERATORS = {"+", "-", "*", "/", "==", "!=", ">", "<", ">=", "<="}
MAX_NESTING_DEPTH = 256
MAX_CALL_DEPTH = 256
RESERVED = {
    "agent",
    "call",
    "else",
    "emit",
    "end",
    "false",
    "forget",
    "fn",
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


def _identifier(value, label: str = "identifier") -> None:
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", value):
        raise ValidationError(f"invalid {label} identifier")
    if value in RESERVED:
        raise ValidationError(f"reserved {label} identifier '{value}'")


def _qualified_identifier(value, label: str) -> None:
    if not isinstance(value, str):
        raise ValidationError(f"invalid {label} identifier")
    for part in value.split("."):
        _identifier(part, label)


def _type_name(value) -> None:
    try:
        validate_type_name(value)
    except (TypeError, ValueError) as error:
        raise ValidationError(str(error)) from error


def _is_value(value) -> bool:
    return type(value) in (str, int, bool)


def validate(program: Program) -> None:
    if not isinstance(program, Program):
        raise ValidationError("IR root must be Program")
    if not isinstance(program.instructions, tuple):
        raise ValidationError("program instructions must be an array")
    _validate_nesting(program.instructions)

    names: set[str] = set()
    view_ids: set[int] = set()
    runnables: dict[str, Agent | Workflow] = {}
    for instruction in program.instructions:
        _require_instruction(instruction)
        if isinstance(instruction, (Agent, Workflow)):
            _identifier(instruction.name, "runnable")
            if instruction.name in names:
                raise ValidationError(f"duplicate runnable '{instruction.name}'")
            names.add(instruction.name)
            runnables[instruction.name] = instruction
        elif isinstance(instruction, Function):
            _qualified_identifier(instruction.name, "function")
        elif isinstance(instruction, UiView):
            if type(instruction.view_id) is not int or instruction.view_id < 1:
                raise ValidationError("UI view id must be a positive integer")
            if instruction.view_id in view_ids:
                raise ValidationError(f"duplicate UI view id '{instruction.view_id}'")
            view_ids.add(instruction.view_id)

    for instruction in program.instructions:
        _validate_instruction(instruction, names, top_level=True)
    _validate_call_graph(runnables)


def _validate_nesting(instructions) -> None:
    stack = [(instruction, 1) for instruction in instructions]
    while stack:
        node, depth = stack.pop()
        if depth > MAX_NESTING_DEPTH:
            raise ValidationError(f"program nesting depth exceeds {MAX_NESTING_DEPTH}")
        if isinstance(node, (Agent, Workflow, Function)):
            stack.extend((child, depth + 1) for child in node.body)
        elif isinstance(node, While):
            stack.append((node.condition, depth + 1))
            stack.extend((child, depth + 1) for child in node.body)
        elif isinstance(node, If):
            stack.append((node.condition, depth + 1))
            stack.extend((child, depth + 1) for child in node.body)
            stack.extend((child, depth + 1) for child in node.else_body)
        elif isinstance(node, (MemoryWrite, Let, Return, Emit)):
            stack.append((node.value, depth + 1))
        elif isinstance(node, Binary):
            stack.append((node.left, depth + 1))
            stack.append((node.right, depth + 1))
        elif isinstance(node, (ToolCall, FunctionCall)):
            stack.extend((argument, depth + 1) for argument in node.arguments)
        elif isinstance(node, ListExpression):
            stack.extend((item, depth + 1) for item in node.items)
        elif isinstance(node, MapExpression):
            stack.extend((item, depth + 1) for entry in node.entries for item in entry)
        elif isinstance(node, UiView):
            stack.append((node.root, depth + 1))
        elif isinstance(node, UiNode):
            stack.extend((child, depth + 1) for child in node.children)
            stack.extend((item.value, depth + 1) for item in node.properties)


def _require_instruction(instruction) -> None:
    if not isinstance(instruction, INSTRUCTION_TYPES):
        raise ValidationError(
            f"invalid instruction node '{type(instruction).__name__}'"
        )


def _validate_block(body, names: set[str]) -> None:
    if not isinstance(body, tuple):
        raise ValidationError("instruction body must be an array")
    for child in body:
        _require_instruction(child)
        _validate_instruction(child, names, top_level=False)


def _validate_instruction(
    instruction: Instruction, names: set[str], *, top_level: bool
) -> None:
    if isinstance(instruction, (Agent, Workflow, Function)):
        if not top_level:
            raise ValidationError("agent and workflow declarations must be top-level")
        if isinstance(instruction, Agent):
            if not isinstance(instruction.tools, tuple):
                raise ValidationError("agent tools must be an array of names")
            for tool in instruction.tools:
                _identifier(tool, "tool")
            if len(instruction.tools) != len(set(instruction.tools)):
                raise ValidationError(
                    f"duplicate tool grant in agent '{instruction.name}'"
                )
        if isinstance(instruction, Function):
            if not isinstance(instruction.parameters, tuple):
                raise ValidationError("function parameters must be an array")
            _type_name(instruction.return_type)
            for parameter in instruction.parameters:
                _identifier(parameter.name, "parameter")
                _type_name(parameter.type_name)
        _validate_block(instruction.body, names)
    elif isinstance(instruction, Run):
        _identifier(instruction.name, "runnable")
        if instruction.name not in names:
            raise ValidationError(f"unknown runnable '{instruction.name}'")
    elif isinstance(instruction, If):
        _validate_expression(instruction.condition)
        _validate_block(instruction.body, names)
        _validate_block(instruction.else_body, names)
    elif isinstance(instruction, While):
        _validate_expression(instruction.condition)
        _validate_block(instruction.body, names)
    elif isinstance(instruction, MemoryWrite):
        _identifier(instruction.key, "memory")
        _identifier(instruction.source, "source")
        _validate_expression(instruction.value)
        if (
            type(instruction.confidence) is not int
            or not 0 <= instruction.confidence <= 100
        ):
            raise ValidationError("memory confidence must be an integer from 0 to 100")
        if instruction.ttl_seconds is not None and (
            type(instruction.ttl_seconds) is not int or instruction.ttl_seconds < 1
        ):
            raise ValidationError("memory ttl must be a positive integer")
    elif isinstance(instruction, Let):
        _identifier(instruction.target, "variable")
        if instruction.type_name is not None:
            _type_name(instruction.type_name)
        _validate_expression(instruction.value)
    elif isinstance(instruction, (Return, Emit)):
        _validate_expression(instruction.value)
    elif isinstance(instruction, Forget):
        _identifier(instruction.key, "memory")
    elif isinstance(instruction, Annotation):
        if not top_level:
            raise ValidationError("annotations must be top-level")
        if instruction.kind not in ANNOTATION_KINDS:
            raise ValidationError(f"unknown annotation kind '{instruction.kind}'")
        if type(instruction.target) is not int or instruction.target < 1:
            raise ValidationError("annotation target must be a positive integer")
        if not isinstance(instruction.value, str) or not instruction.value:
            raise ValidationError("annotation value must be a non-empty string")
    elif isinstance(instruction, UiView):
        if not top_level:
            raise ValidationError("UI views must be top-level")
        _validate_ui_tree(instruction.root)


def _validate_ui_tree(root: UiNode) -> None:
    node_ids: set[int] = set()
    stack = [root]
    while stack:
        node = stack.pop()
        if not isinstance(node, UiNode):
            raise ValidationError("invalid UI node")
        if type(node.node_id) is not int or node.node_id < 1:
            raise ValidationError("UI node id must be a positive integer")
        if node.node_id in node_ids:
            raise ValidationError(f"duplicate UI node id '{node.node_id}'")
        node_ids.add(node.node_id)
        contract = COMPONENTS.get(node.component_id)
        if contract is None:
            raise ValidationError(f"unknown UI component '{node.component_id}'")
        if not all(isinstance(item, tuple) for item in (node.properties, node.events, node.children)):
            raise ValidationError("UI node collections must be arrays")
        property_ids = [item.property_id for item in node.properties]
        if len(property_ids) != len(set(property_ids)):
            raise ValidationError(f"duplicate UI property on node '{node.node_id}'")
        for item in node.properties:
            expected = contract.properties.get(item.property_id)
            if expected is None:
                raise ValidationError(
                    f"property '{item.property_id}' is not valid for component '{node.component_id}'"
                )
            if not isinstance(item.value, Literal):
                raise ValidationError("experimental UI properties must be literal")
            _validate_expression(item.value)
            actual = {str: "string", int: "int", bool: "bool"}[type(item.value.value)]
            if actual != expected:
                raise ValidationError(
                    f"property '{item.property_id}' requires {expected}, got {actual}"
                )
        event_ids = [item.event_id for item in node.events]
        if len(event_ids) != len(set(event_ids)):
            raise ValidationError(f"duplicate UI event on node '{node.node_id}'")
        for item in node.events:
            if item.event_id not in contract.events:
                raise ValidationError(
                    f"event '{item.event_id}' is not valid for component '{node.component_id}'"
                )
            if type(item.action_id) is not int or item.action_id < 1:
                raise ValidationError("UI action id must be a positive integer")
        if node.children and not contract.children:
            raise ValidationError(f"component '{node.component_id}' cannot have children")
        stack.extend(node.children)


def _validate_expression(expression: Expression) -> None:
    if not isinstance(expression, EXPRESSION_TYPES):
        raise ValidationError(f"invalid expression node '{type(expression).__name__}'")
    if isinstance(expression, Literal):
        if not _is_value(expression.value):
            raise ValidationError("literal must be string, integer, or boolean")
        if type(expression.value) is str:
            try:
                expression.value.encode("utf-8")
            except UnicodeEncodeError as error:
                raise ValidationError("invalid Unicode string") from error
    elif isinstance(expression, Binary):
        if expression.operator not in OPERATORS:
            raise ValidationError(f"unknown operator '{expression.operator}'")
        _validate_expression(expression.left)
        _validate_expression(expression.right)
    elif isinstance(expression, ToolCall):
        _identifier(expression.name, "tool")
        if not isinstance(expression.arguments, tuple):
            raise ValidationError("tool arguments must be an array")
        for argument in expression.arguments:
            _validate_expression(argument)
    elif isinstance(expression, FunctionCall):
        _qualified_identifier(expression.name, "function")
        if not isinstance(expression.arguments, tuple):
            raise ValidationError("function arguments must be an array")
        for argument in expression.arguments:
            _validate_expression(argument)
    elif isinstance(expression, ListExpression):
        if not isinstance(expression.items, tuple):
            raise ValidationError("list items must be an array")
        for item in expression.items:
            _validate_expression(item)
    elif isinstance(expression, MapExpression):
        if not isinstance(expression.entries, tuple):
            raise ValidationError("map entries must be an array")
        for entry in expression.entries:
            if not isinstance(entry, tuple) or len(entry) != 2:
                raise ValidationError("map entry must contain key and value")
            _validate_expression(entry[0])
            _validate_expression(entry[1])
    elif isinstance(expression, Variable):
        _identifier(expression.name, "variable")
    elif isinstance(expression, Recall):
        _identifier(expression.key, "memory")


def _validate_call_graph(runnables: dict[str, Agent | Workflow]) -> None:
    graph = {name: _runs(runnable.body) for name, runnable in runnables.items()}
    state: dict[str, int] = {name: 0 for name in graph}

    for root, dependencies in graph.items():
        if state[root] != 0:
            continue
        state[root] = 1
        stack: list[tuple[str, object]] = [(root, iter(dependencies))]
        while stack:
            name, dependencies = stack[-1]
            try:
                dependency = next(dependencies)
            except StopIteration:
                state[name] = 2
                stack.pop()
                continue
            if state[dependency] == 1:
                raise ValidationError(f"workflow cycle detected at '{dependency}'")
            if state[dependency] == 0:
                if len(stack) >= MAX_CALL_DEPTH:
                    raise ValidationError(
                        f"workflow call depth exceeds {MAX_CALL_DEPTH}"
                    )
                state[dependency] = 1
                stack.append((dependency, iter(graph[dependency])))


def _runs(body) -> set[str]:
    result: set[str] = set()
    for instruction in body:
        if isinstance(instruction, Run):
            result.add(instruction.name)
        elif isinstance(instruction, If):
            result |= _runs(instruction.body) | _runs(instruction.else_body)
        elif isinstance(instruction, While):
            result |= _runs(instruction.body)
    return result
