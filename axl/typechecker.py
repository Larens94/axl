from dataclasses import dataclass

from .ir import (
    Binary,
    Emit,
    Function,
    FunctionCall,
    If,
    Let,
    ListExpression,
    Literal,
    MemoryWrite,
    Program,
    Recall,
    Return,
    ToolCall,
    Variable,
    While,
)
from .type_names import split_type_name


class TypeCheckError(ValueError):
    """Raised when an AXL program violates its static type contract."""


TYPE_NAMES = {"int", "string", "bool"}
ANY = "any"


@dataclass(frozen=True)
class FunctionSignature:
    parameters: tuple[str, ...]
    return_type: str


def typecheck(program: Program) -> None:
    functions: dict[str, Function] = {}
    signatures: dict[str, FunctionSignature] = {}
    for instruction in program.instructions:
        if isinstance(instruction, Function):
            if instruction.name in functions:
                raise TypeCheckError(f"duplicate function '{instruction.name}'")
            for parameter in instruction.parameters:
                _require_known_type(parameter.type_name)
            _require_known_type(instruction.return_type)
            parameter_names = [parameter.name for parameter in instruction.parameters]
            if len(parameter_names) != len(set(parameter_names)):
                raise TypeCheckError(
                    f"duplicate parameter in function '{instruction.name}'"
                )
            functions[instruction.name] = instruction
            signatures[instruction.name] = FunctionSignature(
                tuple(parameter.type_name for parameter in instruction.parameters),
                instruction.return_type,
            )

    for function in functions.values():
        environment = {
            parameter.name: parameter.type_name for parameter in function.parameters
        }
        return_types: list[str] = []
        _check_block(
            function.body,
            environment,
            signatures,
            return_types,
            function.name,
        )
        for return_type in return_types:
            if not _compatible(function.return_type, return_type):
                raise TypeCheckError(
                    f"function '{function.name}' must return "
                    f"{function.return_type}, got {return_type}"
                )
        if not _always_returns(function.body):
            raise TypeCheckError(
                f"function '{function.name}' may complete without returning"
            )

    _check_block(program.instructions, {}, signatures, [], None)


def _check_block(
    instructions,
    environment: dict[str, str],
    signatures: dict[str, FunctionSignature],
    return_types: list[str],
    function_name: str | None,
) -> None:
    for instruction in instructions:
        if isinstance(instruction, Function):
            continue
        if isinstance(instruction, Let):
            if instruction.type_name is not None:
                _require_known_type(instruction.type_name)
            value_type = _expression_type(instruction.value, environment, signatures)
            if instruction.type_name and not _compatible(
                instruction.type_name, value_type
            ):
                raise TypeCheckError(
                    f"variable '{instruction.target}' must be "
                    f"{instruction.type_name}, got {value_type}"
                )
            environment[instruction.target] = instruction.type_name or value_type
        elif isinstance(instruction, Return):
            if function_name is None:
                raise TypeCheckError("return is only valid inside a function")
            return_types.append(
                _expression_type(instruction.value, environment, signatures)
            )
        elif isinstance(instruction, (Emit, MemoryWrite)):
            _expression_type(instruction.value, environment, signatures)
        elif isinstance(instruction, If):
            condition_type = _expression_type(
                instruction.condition, environment, signatures
            )
            if condition_type not in {"bool", ANY}:
                raise TypeCheckError("if condition must be bool")
            _check_block(
                instruction.body,
                dict(environment),
                signatures,
                return_types,
                function_name,
            )
            _check_block(
                instruction.else_body,
                dict(environment),
                signatures,
                return_types,
                function_name,
            )
        elif isinstance(instruction, While):
            condition_type = _expression_type(
                instruction.condition, environment, signatures
            )
            if condition_type not in {"bool", ANY}:
                raise TypeCheckError("while condition must be bool")
            _check_block(
                instruction.body,
                dict(environment),
                signatures,
                return_types,
                function_name,
            )
        elif hasattr(instruction, "body"):
            _check_block(
                instruction.body,
                {},
                signatures,
                return_types,
                function_name,
            )


def _expression_type(expression, environment, signatures) -> str:
    if isinstance(expression, Literal):
        return {str: "string", int: "int", bool: "bool"}[type(expression.value)]
    if isinstance(expression, ListExpression):
        item_types = [
            _expression_type(item, environment, signatures) for item in expression.items
        ]
        item_type = _unify_types(item_types) if item_types else ANY
        result = f"list<{item_type}>"
        try:
            split_type_name(result)
        except ValueError as error:
            raise TypeCheckError(str(error)) from error
        return result
    if isinstance(expression, Variable):
        if expression.name not in environment:
            raise TypeCheckError(f"unknown variable '{expression.name}'")
        return environment[expression.name]
    if isinstance(expression, Recall | ToolCall):
        return ANY
    if isinstance(expression, FunctionCall):
        signature = signatures.get(expression.name)
        if signature is None:
            raise TypeCheckError(f"unknown function '{expression.name}'")
        if len(expression.arguments) != len(signature.parameters):
            raise TypeCheckError(
                f"function '{expression.name}' expects {len(signature.parameters)} "
                f"arguments, got {len(expression.arguments)}"
            )
        for index, (argument, expected) in enumerate(
            zip(expression.arguments, signature.parameters, strict=True), 1
        ):
            actual = _expression_type(argument, environment, signatures)
            if not _compatible(expected, actual):
                raise TypeCheckError(
                    f"argument {index} of '{expression.name}' must be "
                    f"{expected}, got {actual}"
                )
        return signature.return_type
    if isinstance(expression, Binary):
        left = _expression_type(expression.left, environment, signatures)
        right = _expression_type(expression.right, environment, signatures)
        if ANY in {left, right}:
            return ANY
        if expression.operator in {"==", "!="}:
            if left != right:
                raise TypeCheckError(
                    f"operator '{expression.operator}' requires matching types"
                )
            return "bool"
        if expression.operator in {">", "<", ">=", "<="}:
            if left != "int" or right != "int":
                raise TypeCheckError(
                    f"operator '{expression.operator}' requires int operands"
                )
            return "bool"
        if expression.operator == "+" and left == right == "string":
            return "string"
        if left != "int" or right != "int":
            raise TypeCheckError(
                f"operator '{expression.operator}' requires int operands"
            )
        return "int"
    raise TypeCheckError("unsupported expression")


def _unify_types(type_names: list[str]) -> str:
    try:
        parsed = [split_type_name(type_name) for type_name in type_names]
    except ValueError as error:
        raise TypeCheckError(str(error)) from error
    depths = {depth for depth, _ in parsed}
    concrete = {base for _, base in parsed if base != ANY}
    if len(depths) != 1 or len(concrete) > 1:
        raise TypeCheckError("list items must have one type")
    result = next(iter(concrete), ANY)
    for _ in range(next(iter(depths))):
        result = f"list<{result}>"
    return result


def _compatible(expected: str, actual: str) -> bool:
    if expected == actual or ANY in {expected, actual}:
        return True
    try:
        expected_depth, expected_base = split_type_name(expected)
        actual_depth, actual_base = split_type_name(actual)
    except (TypeError, ValueError):
        return False
    return expected_depth == actual_depth and (
        expected_base == actual_base or ANY in {expected_base, actual_base}
    )


def _require_known_type(type_name: str) -> None:
    if not _is_known_type(type_name):
        raise TypeCheckError(f"unknown type '{type_name}'")


def _is_known_type(type_name: str) -> bool:
    try:
        _, base = split_type_name(type_name)
    except (TypeError, ValueError):
        return False
    return base in TYPE_NAMES


def _always_returns(instructions) -> bool:
    for instruction in instructions:
        if isinstance(instruction, Return):
            return True
        if (
            isinstance(instruction, If)
            and instruction.else_body
            and _always_returns(instruction.body)
            and _always_returns(instruction.else_body)
        ):
            return True
    return False
