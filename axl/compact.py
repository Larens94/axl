import json
import re

from .ir import (
    Agent,
    Binary,
    Emit,
    Forget,
    Function,
    FunctionCall,
    If,
    Let,
    ListExpression,
    Literal,
    MemoryWrite,
    Parameter,
    Program,
    Recall,
    Return,
    Run,
    ToolCall,
    Variable,
    While,
    Workflow,
)


class CompactParseError(ValueError):
    pass


_OPERATORS = {"+", "-", "*", "/", "=", "!", ">", "<", "G", "L"}
_OPERATOR_NAMES = {"=": "==", "!": "!=", "G": ">=", "L": "<="}
_TYPE_NAMES = {"i": "int", "s": "string", "b": "bool"}
_TYPE_CODES = {value: key for key, value in _TYPE_NAMES.items()}
_CALL = re.compile(r"(?P<kind>[!^])(?P<name>[A-Za-z_][A-Za-z0-9_.]*)/(?P<arity>\d+)")
_LIST = re.compile(r"~(?P<arity>\d+)")
_OPERATOR_CODES = {value: key for key, value in _OPERATOR_NAMES.items()}
_MAX_CALL_ARITY = 65_535
_MAX_SOURCE_BYTES = 1_000_000


def is_compact_source(source: str) -> bool:
    return re.match(r"^\s*2\s*(?:;|$)", source) is not None


def program_to_compact(program: Program) -> str:
    from .validation import ValidationError, validate

    try:
        validate(program)
    except ValidationError as error:
        raise CompactParseError(str(error)) from error
    frames = ["2"]
    for instruction in program.instructions:
        frames.extend(_instruction_frames(instruction))
    source = ";".join(frames)
    if len(source.encode("utf-8")) > _MAX_SOURCE_BYTES:
        raise CompactParseError(f"source exceeds {_MAX_SOURCE_BYTES} bytes")
    return source


def _type_code(type_name: str) -> str:
    if type_name.startswith("list<") and type_name.endswith(">"):
        return "l" + _type_code(type_name[5:-1])
    try:
        return _TYPE_CODES[type_name]
    except KeyError as error:
        raise CompactParseError(f"cannot encode type '{type_name}'") from error


def _type_name(type_code: str) -> str:
    if type_code.startswith("l"):
        if len(type_code) == 1:
            raise CompactParseError("invalid list type")
        return f"list<{_type_name(type_code[1:])}>"
    try:
        return _TYPE_NAMES[type_code]
    except KeyError as error:
        raise CompactParseError(f"invalid type '{type_code}'") from error


def _instruction_frames(instruction) -> list[str]:
    if isinstance(instruction, Let):
        frame = f"10|{instruction.target}|{_expression_source(instruction.value)}"
        if instruction.type_name:
            frame += f"|{_type_code(instruction.type_name)}"
        return [frame]
    if isinstance(instruction, Return):
        return [f"11|{_expression_source(instruction.value)}"]
    if isinstance(instruction, Emit):
        return [f"12|{_expression_source(instruction.value)}"]
    if isinstance(instruction, MemoryWrite):
        frame = f"20|{instruction.key}|{_expression_source(instruction.value)}"
        if (instruction.confidence, instruction.ttl_seconds, instruction.source) != (
            100,
            None,
            "program",
        ):
            ttl = (
                "-" if instruction.ttl_seconds is None else str(instruction.ttl_seconds)
            )
            frame += f"|{instruction.confidence}|{ttl}|{instruction.source}"
        return [frame]
    if isinstance(instruction, Forget):
        return [f"21|{instruction.key}"]
    if isinstance(instruction, If):
        frames = [f"30|{_expression_source(instruction.condition)}"]
        for child in instruction.body:
            frames.extend(_instruction_frames(child))
        if instruction.else_body:
            frames.append("31")
            for child in instruction.else_body:
                frames.extend(_instruction_frames(child))
        return [*frames, "99"]
    if isinstance(instruction, While):
        frames = [f"32|{_expression_source(instruction.condition)}"]
        for child in instruction.body:
            frames.extend(_instruction_frames(child))
        return [*frames, "99"]
    if isinstance(instruction, Function):
        parameters = ",".join(
            f"{item.name}:{_type_code(item.type_name)}"
            for item in instruction.parameters
        )
        frames = [
            f"40|{instruction.name}|{parameters}|{_type_code(instruction.return_type)}"
        ]
        for child in instruction.body:
            frames.extend(_instruction_frames(child))
        return [*frames, "99"]
    if isinstance(instruction, Agent):
        frame = f"50|{instruction.name}"
        if instruction.tools:
            frame += "|" + ",".join(instruction.tools)
        frames = [frame]
        for child in instruction.body:
            frames.extend(_instruction_frames(child))
        return [*frames, "99"]
    if isinstance(instruction, Workflow):
        frames = [f"51|{instruction.name}"]
        for child in instruction.body:
            frames.extend(_instruction_frames(child))
        return [*frames, "99"]
    if isinstance(instruction, Run):
        return [f"52|{instruction.name}"]
    raise CompactParseError(f"cannot encode instruction '{type(instruction).__name__}'")


def _expression_source(expression) -> str:
    if isinstance(expression, Literal):
        if type(expression.value) is bool:
            return "?1" if expression.value else "?0"
        if type(expression.value) is int:
            return f"#{expression.value}"
        if type(expression.value) is str:
            _require_unicode_scalar_string(expression.value)
            return json.dumps(
                expression.value, ensure_ascii=False, separators=(",", ":")
            )
        raise CompactParseError("cannot encode literal")
    if isinstance(expression, Variable):
        return f"${expression.name}"
    if isinstance(expression, Recall):
        return f"@{expression.key}"
    if isinstance(expression, Binary):
        operator = _OPERATOR_CODES.get(expression.operator, expression.operator)
        return f"{_expression_source(expression.left)},{_expression_source(expression.right)},{operator}"
    if isinstance(expression, ToolCall):
        _require_call_arity(len(expression.arguments))
        arguments = [_expression_source(item) for item in expression.arguments]
        return ",".join([*arguments, f"!{expression.name}/{len(arguments)}"])
    if isinstance(expression, FunctionCall):
        _require_call_arity(len(expression.arguments))
        arguments = [_expression_source(item) for item in expression.arguments]
        return ",".join([*arguments, f"^{expression.name}/{len(arguments)}"])
    if isinstance(expression, ListExpression):
        _require_call_arity(len(expression.items))
        items = [_expression_source(item) for item in expression.items]
        return ",".join([*items, f"~{len(items)}"])
    raise CompactParseError(f"cannot encode expression '{type(expression).__name__}'")


def parse_compact(source: str) -> Program:
    frames = split_compact_frames(source)
    if not frames or frames[0] != "2":
        raise CompactParseError("compact source requires version header '2'")
    instructions, position, terminator = _block(frames, 1, False)
    if terminator is not None:
        raise CompactParseError(f"frame {position}: unexpected opcode '{terminator}'")
    return Program(tuple(instructions))


def split_compact_frames(source: str) -> list[str]:
    return _split(_remove_unquoted_whitespace(source), ";")


def _require_call_arity(arity: int) -> None:
    if arity > _MAX_CALL_ARITY:
        raise CompactParseError("invalid call arity")


def _require_unicode_scalar_string(value: str) -> None:
    try:
        value.encode("utf-8")
    except UnicodeEncodeError as error:
        raise CompactParseError("invalid Unicode string") from error


def _block(frames: list[str], position: int, allow_else: bool):
    instructions = []
    while position < len(frames):
        index = position
        frame = frames[position]
        position += 1
        if not frame:
            continue
        fields = _split(frame, "|")
        opcode = fields[0]
        if opcode == "99":
            return instructions, position, opcode
        if opcode == "31":
            if not allow_else:
                raise CompactParseError(f"frame {index}: unexpected else opcode")
            return instructions, position, opcode
        if opcode == "10" and len(fields) in {3, 4}:
            type_name = None
            if len(fields) == 4:
                try:
                    type_name = _type_name(fields[3])
                except CompactParseError as error:
                    raise CompactParseError(
                        f"frame {index}: invalid binding type '{fields[3]}'"
                    ) from error
            instructions.append(
                Let(fields[1], _expression(fields[2], index), type_name)
            )
        elif opcode == "12" and len(fields) == 2:
            instructions.append(Emit(_expression(fields[1], index)))
        elif opcode == "20" and len(fields) in {3, 6}:
            metadata = {}
            if len(fields) == 6:
                try:
                    metadata = {
                        "confidence": int(fields[3]),
                        "ttl_seconds": None if fields[4] == "-" else int(fields[4]),
                        "source": fields[5],
                    }
                except ValueError as error:
                    raise CompactParseError(
                        f"frame {index}: invalid memory metadata"
                    ) from error
            instructions.append(
                MemoryWrite(fields[1], _expression(fields[2], index), **metadata)
            )
        elif opcode == "21" and len(fields) == 2:
            instructions.append(Forget(fields[1]))
        elif opcode == "30" and len(fields) == 2:
            body, position, terminator = _block(frames, position, True)
            if terminator is None:
                raise CompactParseError(f"frame {index}: if missing end opcode 99")
            else_body = []
            if terminator == "31":
                else_body, position, terminator = _block(frames, position, False)
                if terminator != "99":
                    raise CompactParseError(f"frame {index}: if missing end opcode 99")
            instructions.append(
                If(_expression(fields[1], index), tuple(body), tuple(else_body))
            )
        elif opcode == "32" and len(fields) == 2:
            body, position, terminator = _block(frames, position, False)
            if terminator != "99":
                raise CompactParseError(f"frame {index}: while missing end opcode 99")
            instructions.append(While(_expression(fields[1], index), tuple(body)))
        elif opcode == "40" and len(fields) == 4:
            parameters = []
            if fields[2]:
                for raw in fields[2].split(","):
                    name, separator, type_code = raw.partition(":")
                    if not separator:
                        raise CompactParseError(
                            f"frame {index}: invalid function parameter '{raw}'"
                        )
                    try:
                        parameter_type = _type_name(type_code)
                    except CompactParseError as error:
                        raise CompactParseError(
                            f"frame {index}: invalid function parameter '{raw}'"
                        ) from error
                    parameters.append(Parameter(name, parameter_type))
            try:
                return_type = _type_name(fields[3])
            except CompactParseError as error:
                raise CompactParseError(
                    f"frame {index}: invalid return type '{fields[3]}'"
                ) from error
            body, position, terminator = _block(frames, position, False)
            if terminator != "99":
                raise CompactParseError(
                    f"frame {index}: function missing end opcode 99"
                )
            instructions.append(
                Function(fields[1], tuple(parameters), return_type, tuple(body))
            )
        elif opcode == "11" and len(fields) == 2:
            instructions.append(Return(_expression(fields[1], index)))
        elif opcode == "50" and len(fields) in {2, 3}:
            grants = (
                tuple(item for item in fields[2].split(",") if item)
                if len(fields) == 3
                else ()
            )
            body, position, terminator = _block(frames, position, False)
            if terminator != "99":
                raise CompactParseError(f"frame {index}: agent missing end opcode 99")
            instructions.append(Agent(fields[1], grants, tuple(body)))
        elif opcode == "51" and len(fields) == 2:
            body, position, terminator = _block(frames, position, False)
            if terminator != "99":
                raise CompactParseError(
                    f"frame {index}: workflow missing end opcode 99"
                )
            instructions.append(Workflow(fields[1], tuple(body)))
        elif opcode == "52" and len(fields) == 2:
            instructions.append(Run(fields[1]))
        else:
            raise CompactParseError(
                f"frame {index}: invalid opcode or arity '{opcode}'"
            )
    return instructions, position, None


def _expression(source: str, frame: int):
    stack = []
    for token in _split(source, ","):
        call = _CALL.fullmatch(token)
        list_constructor = _LIST.fullmatch(token)
        if list_constructor:
            try:
                arity = int(list_constructor.group("arity"))
            except ValueError as error:
                raise CompactParseError(f"frame {frame}: invalid list arity") from error
            if arity > _MAX_CALL_ARITY or len(stack) < arity:
                raise CompactParseError(f"frame {frame}: invalid list arity")
            items = tuple(stack[-arity:]) if arity else ()
            if arity:
                del stack[-arity:]
            stack.append(ListExpression(items))
        elif call:
            try:
                arity = int(call.group("arity"))
            except ValueError as error:
                raise CompactParseError(f"frame {frame}: invalid call arity") from error
            if arity > _MAX_CALL_ARITY:
                raise CompactParseError(f"frame {frame}: invalid call arity")
            if len(stack) < arity:
                raise CompactParseError(
                    f"frame {frame}: call '{token}' needs {arity} values"
                )
            arguments = tuple(stack[-arity:]) if arity else ()
            if arity:
                del stack[-arity:]
            node = ToolCall if call.group("kind") == "!" else FunctionCall
            stack.append(node(call.group("name"), arguments))
        elif token in _OPERATORS:
            if len(stack) < 2:
                raise CompactParseError(
                    f"frame {frame}: operator '{token}' needs two values"
                )
            right = stack.pop()
            left = stack.pop()
            stack.append(Binary(left, _OPERATOR_NAMES.get(token, token), right))
        elif token.startswith("#"):
            try:
                stack.append(Literal(int(token[1:])))
            except ValueError as error:
                raise CompactParseError(
                    f"frame {frame}: invalid integer '{token}'"
                ) from error
        elif token.startswith("$") and len(token) > 1:
            stack.append(Variable(token[1:]))
        elif token.startswith("@") and len(token) > 1:
            stack.append(Recall(token[1:]))
        elif token.startswith('"'):
            try:
                value = json.loads(token)
            except json.JSONDecodeError as error:
                raise CompactParseError(f"frame {frame}: invalid string") from error
            if not isinstance(value, str):
                raise CompactParseError(f"frame {frame}: string required")
            _require_unicode_scalar_string(value)
            stack.append(Literal(value))
        elif token == "?1":
            stack.append(Literal(True))
        elif token == "?0":
            stack.append(Literal(False))
        else:
            raise CompactParseError(
                f"frame {frame}: invalid expression token '{token}'"
            )
    if len(stack) != 1:
        raise CompactParseError(f"frame {frame}: expression leaves {len(stack)} values")
    return stack[0]


def _split(source: str, delimiter: str) -> list[str]:
    parts = []
    start = 0
    quoted = False
    escaped = False
    for index, character in enumerate(source):
        if escaped:
            escaped = False
        elif quoted and character == "\\":
            escaped = True
        elif character == '"':
            quoted = not quoted
        elif character == delimiter and not quoted:
            parts.append(source[start:index].strip())
            start = index + 1
    if quoted:
        raise CompactParseError("unterminated string")
    parts.append(source[start:].strip())
    return parts


def _remove_unquoted_whitespace(source: str) -> str:
    result = []
    quoted = False
    escaped = False
    for character in source:
        if escaped:
            result.append(character)
            escaped = False
        elif quoted and character == "\\":
            result.append(character)
            escaped = True
        elif character == '"':
            result.append(character)
            quoted = not quoted
        elif quoted or not character.isspace():
            result.append(character)
    if quoted:
        raise CompactParseError("unterminated string")
    return "".join(result)
