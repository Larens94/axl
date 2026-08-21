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
    MapExpression,
    MemoryWrite,
    Program,
    Recall,
    Return,
    ToolCall,
    UiView,
    Variable,
    While,
)
from .type_names import is_known_type_name, split_map_type, validate_type_name


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
        if isinstance(instruction, (Function, UiView)):
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
            validate_type_name(result)
        except ValueError as error:
            raise TypeCheckError(str(error)) from error
        return result
    if isinstance(expression, MapExpression):
        key_types = [
            _expression_type(key, environment, signatures)
            for key, _ in expression.entries
        ]
        value_types = [
            _expression_type(value, environment, signatures)
            for _, value in expression.entries
        ]
        key_type = _unify_types(key_types, "map keys") if key_types else ANY
        value_type = _unify_types(value_types, "map values") if value_types else ANY
        if key_type not in {"int", "string", "bool", ANY}:
            raise TypeCheckError("map keys must be scalar")
        result = f"map<{key_type},{value_type}>"
        try:
            validate_type_name(result)
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


def _unify_types(type_names: list[str], context: str = "list items") -> str:
    result = ANY
    for type_name in type_names:
        result = _unify_type_pair(result, type_name, context)
    return result


def _unify_type_pair(left: str, right: str, context: str) -> str:
    if left == ANY:
        return right
    if right == ANY or left == right:
        return left
    left_list = _list_item_type(left)
    right_list = _list_item_type(right)
    if left_list is not None and right_list is not None:
        return f"list<{_unify_type_pair(left_list, right_list, context)}>"
    left_map = split_map_type(left)
    right_map = split_map_type(right)
    if left_map is not None and right_map is not None:
        key = _unify_type_pair(left_map[0], right_map[0], context)
        value = _unify_type_pair(left_map[1], right_map[1], context)
        return f"map<{key},{value}>"
    raise TypeCheckError(f"{context} must have one type")


def _compatible(expected: str, actual: str) -> bool:
    if expected == actual or ANY in {expected, actual}:
        return True
    expected_map = split_map_type(expected)
    actual_map = split_map_type(actual)
    if expected_map is not None or actual_map is not None:
        return (
            expected_map is not None
            and actual_map is not None
            and _compatible(expected_map[0], actual_map[0])
            and _compatible(expected_map[1], actual_map[1])
        )
    expected_list = _list_item_type(expected)
    actual_list = _list_item_type(actual)
    return (
        expected_list is not None
        and actual_list is not None
        and _compatible(expected_list, actual_list)
    )


def _list_item_type(type_name: str) -> str | None:
    if type_name.startswith("list<") and type_name.endswith(">"):
        return type_name[5:-1]
    return None


def _require_known_type(type_name: str) -> None:
    if not _is_known_type(type_name):
        raise TypeCheckError(f"unknown type '{type_name}'")


def _is_known_type(type_name: str) -> bool:
    return is_known_type_name(type_name)


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
