from dataclasses import dataclass

from .ir import (
    Binary,
    Emit,
    Function,
    FunctionCall,
    If,
    Let,
    Literal,
    MemoryWrite,
    Program,
    Recall,
    Return,
    ToolCall,
    Variable,
    While,
)


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


def _compatible(expected: str, actual: str) -> bool:
    return expected == actual or ANY in {expected, actual}


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
