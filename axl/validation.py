import re

from .ir import (
    Agent,
    Binary,
    Emit,
    Expression,
    Forget,
    Function,
    FunctionCall,
    If,
    Instruction,
    Let,
    Literal,
    MemoryWrite,
    Program,
    Recall,
    Return,
    Run,
    ToolCall,
    Variable,
    While,
    Workflow,
)


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
)
EXPRESSION_TYPES = (Literal, Variable, Recall, ToolCall, FunctionCall, Binary)
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


def _is_value(value) -> bool:
    return type(value) in (str, int, bool)


def validate(program: Program) -> None:
    if not isinstance(program, Program):
        raise ValidationError("IR root must be Program")
    if not isinstance(program.instructions, tuple):
        raise ValidationError("program instructions must be an array")
    _validate_nesting(program.instructions)

    names: set[str] = set()
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
            _identifier(instruction.return_type, "type")
            for parameter in instruction.parameters:
                _identifier(parameter.name, "parameter")
                _identifier(parameter.type_name, "type")
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
            _identifier(instruction.type_name, "type")
        _validate_expression(instruction.value)
    elif isinstance(instruction, (Return, Emit)):
        _validate_expression(instruction.value)
    elif isinstance(instruction, Forget):
        _identifier(instruction.key, "memory")


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
