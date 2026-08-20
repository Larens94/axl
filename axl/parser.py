import ast
import re
from dataclasses import dataclass

from .ir import (
    Agent,
    Binary,
    Emit,
    Forget,
    Function,
    FunctionCall,
    If,
    Let,
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


class ParseError(ValueError):
    pass


_TOKEN = re.compile(
    r'(?P<STRING>"(?:\\.|[^"\\])*")|(?P<INT>\d+)|'
    r"(?P<BOOL>true|false)\b|(?P<IDENT>[A-Za-z_][A-Za-z0-9_]*)|"
    r"(?P<OP>==|!=|>=|<=|[+\-*/(),.:<>])"
)
_PRECEDENCE = {
    "==": 1,
    "!=": 1,
    ">": 2,
    "<": 2,
    ">=": 2,
    "<=": 2,
    "+": 3,
    "-": 3,
    "*": 4,
    "/": 4,
}
_RESERVED = {
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
_MAX_SOURCE_BYTES = 1_000_000


@dataclass(frozen=True)
class Token:
    kind: str
    text: str


def _tokenize(text: str, line: int) -> list[Token]:
    tokens: list[Token] = []
    position = 0
    for match in _TOKEN.finditer(text):
        if text[position : match.start()].strip():
            raise ParseError(
                f"line {line}: invalid expression near '{text[position:]}'"
            )
        tokens.append(Token(match.lastgroup or "", match.group()))
        position = match.end()
    if text[position:].strip():
        raise ParseError(f"line {line}: invalid expression near '{text[position:]}'")
    if not tokens:
        raise ParseError(f"line {line}: expression required")
    return tokens


class ExpressionParser:
    def __init__(self, text: str, line: int):
        self.tokens = _tokenize(text, line)
        self.position = 0
        self.line = line

    def parse(self):
        expression = self._binary(0)
        if self.position != len(self.tokens):
            raise ParseError(
                f"line {self.line}: unexpected token '{self.tokens[self.position].text}'"
            )
        return expression

    def _binary(self, minimum: int):
        left = self._primary()
        while self.position < len(self.tokens):
            operator = self.tokens[self.position].text
            precedence = _PRECEDENCE.get(operator)
            if precedence is None or precedence < minimum:
                break
            self.position += 1
            right = self._binary(precedence + 1)
            left = Binary(left, operator, right)
        return left

    def _primary(self):
        if self.position >= len(self.tokens):
            raise ParseError(f"line {self.line}: incomplete expression")
        token = self.tokens[self.position]
        self.position += 1
        if token.kind == "STRING":
            return Literal(ast.literal_eval(token.text))
        if token.kind == "INT":
            return Literal(int(token.text))
        if token.kind == "BOOL":
            return Literal(token.text == "true")
        if token.text == "(":
            expression = self._binary(0)
            if (
                self.position >= len(self.tokens)
                or self.tokens[self.position].text != ")"
            ):
                raise ParseError(f"line {self.line}: missing ')'")
            self.position += 1
            return expression
        if token.kind == "IDENT" and token.text == "recall":
            if (
                self.position >= len(self.tokens)
                or self.tokens[self.position].kind != "IDENT"
            ):
                raise ParseError(f"line {self.line}: recall requires a memory key")
            key = self.tokens[self.position].text
            _identifier(key, self.line)
            self.position += 1
            return Recall(key)
        if token.kind == "IDENT" and token.text == "call":
            return self._tool_call()
        if token.kind == "IDENT":
            _identifier(token.text, self.line)
            if (
                self.position < len(self.tokens)
                and self.tokens[self.position].text == "."
            ):
                self.position += 1
                if (
                    self.position >= len(self.tokens)
                    or self.tokens[self.position].kind != "IDENT"
                ):
                    raise ParseError(f"line {self.line}: namespace member required")
                member = self.tokens[self.position].text
                _identifier(member, self.line)
                self.position += 1
                if (
                    self.position >= len(self.tokens)
                    or self.tokens[self.position].text != "("
                ):
                    raise ParseError(
                        f"line {self.line}: namespaced function call requires '('"
                    )
                return self._function_call(f"{token.text}.{member}")
            if (
                self.position < len(self.tokens)
                and self.tokens[self.position].text == "("
            ):
                return self._function_call(token.text)
            return Variable(token.text)
        raise ParseError(f"line {self.line}: unexpected token '{token.text}'")

    def _tool_call(self):
        if (
            self.position >= len(self.tokens)
            or self.tokens[self.position].kind != "IDENT"
        ):
            raise ParseError(f"line {self.line}: call requires a tool name")
        name = self.tokens[self.position].text
        _identifier(name, self.line)
        self.position += 1
        if self.position >= len(self.tokens) or self.tokens[self.position].text != "(":
            raise ParseError(f"line {self.line}: tool arguments require '('")
        self.position += 1
        arguments = []
        if self.position < len(self.tokens) and self.tokens[self.position].text != ")":
            while True:
                arguments.append(self._binary(0))
                if (
                    self.position < len(self.tokens)
                    and self.tokens[self.position].text == ","
                ):
                    self.position += 1
                    continue
                break
        if self.position >= len(self.tokens) or self.tokens[self.position].text != ")":
            raise ParseError(f"line {self.line}: tool call missing ')'")
        self.position += 1
        return ToolCall(name, tuple(arguments))

    def _function_call(self, name: str):
        self.position += 1
        arguments = []
        if self.position < len(self.tokens) and self.tokens[self.position].text != ")":
            while True:
                arguments.append(self._binary(0))
                if (
                    self.position < len(self.tokens)
                    and self.tokens[self.position].text == ","
                ):
                    self.position += 1
                    continue
                break
        if self.position >= len(self.tokens) or self.tokens[self.position].text != ")":
            raise ParseError(f"line {self.line}: function call missing ')'")
        self.position += 1
        return FunctionCall(name, tuple(arguments))


def _expression(text: str, line: int):
    return ExpressionParser(text, line).parse()


def parse(source: str) -> Program:
    try:
        source_size = len(source.encode("utf-8"))
    except UnicodeEncodeError as error:
        raise ParseError("invalid Unicode source") from error
    if source_size > _MAX_SOURCE_BYTES:
        raise ParseError(f"source exceeds {_MAX_SOURCE_BYTES} bytes")
    from .compact import CompactParseError, is_compact_source, parse_compact

    if is_compact_source(source):
        try:
            return parse_compact(source)
        except CompactParseError as error:
            raise ParseError(str(error)) from error
        except RecursionError as error:
            raise ParseError("source nesting is too deep") from error
    lines = [(number, raw.strip()) for number, raw in enumerate(source.splitlines(), 1)]
    try:
        instructions, position, terminator = _block(lines, 0, allow_else=False)
    except RecursionError as error:
        raise ParseError("source nesting is too deep") from error
    if terminator is not None:
        number, _ = lines[position - 1]
        raise ParseError(f"line {number}: unexpected {terminator}")
    return Program(tuple(instructions))


def _block(lines: list[tuple[int, str]], position: int, allow_else: bool):
    instructions = []
    while position < len(lines):
        number, line = lines[position]
        position += 1
        if not line or line.startswith("#"):
            continue
        if line == "end":
            return instructions, position, "end"
        if line == "else":
            if not allow_else:
                raise ParseError(f"line {number}: unexpected else")
            return instructions, position, "else"
        if line.startswith("memory ") and "=" in line:
            left, right = line[7:].split("=", 1)
            key = left.strip()
            _identifier(key, number)
            expression_text, metadata = _memory_metadata(right.strip(), number)
            instructions.append(
                MemoryWrite(key, _expression(expression_text, number), **metadata)
            )
        elif line.startswith("forget "):
            key = line[7:].strip()
            _identifier(key, number)
            instructions.append(Forget(key))
        elif line.startswith("let ") and "=" in line:
            left, right = line[4:].split("=", 1)
            target, separator, type_name = left.strip().partition(":")
            target = target.strip()
            _identifier(target, number)
            if separator:
                type_name = type_name.strip()
                _type_name(type_name, number)
            instructions.append(
                Let(target, _expression(right.strip(), number), type_name or None)
            )
        elif line.startswith("return "):
            instructions.append(Return(_expression(line[7:].strip(), number)))
        elif line.startswith("emit "):
            instructions.append(Emit(_expression(line[5:].strip(), number)))
        elif line.startswith("if "):
            body, position, terminator = _block(lines, position, allow_else=True)
            if terminator is None:
                raise ParseError(f"line {number}: missing end")
            else_body = []
            if terminator == "else":
                else_body, position, terminator = _block(
                    lines, position, allow_else=False
                )
                if terminator != "end":
                    raise ParseError(f"line {number}: missing end")
            instructions.append(
                If(_expression(line[3:].strip(), number), tuple(body), tuple(else_body))
            )
        elif line.startswith("while "):
            body, position, terminator = _block(lines, position, allow_else=False)
            if terminator != "end":
                raise ParseError(f"line {number}: missing end")
            instructions.append(
                While(_expression(line[6:].strip(), number), tuple(body))
            )
        elif line.startswith("agent "):
            declaration = line[6:].strip()
            name, separator, grants = declaration.partition(" uses ")
            _identifier(name, number)
            tools = (
                tuple(item.strip() for item in grants.split(",") if item.strip())
                if separator
                else ()
            )
            for tool in tools:
                _identifier(tool, number)
            body, position, terminator = _block(lines, position, allow_else=False)
            if terminator != "end":
                raise ParseError(f"line {number}: missing end")
            instructions.append(Agent(name, tools, tuple(body)))
        elif line.startswith("fn "):
            match = re.fullmatch(
                r"fn\s+([A-Za-z_][A-Za-z0-9_]*)\((.*)\)\s*->\s*([A-Za-z_][A-Za-z0-9_]*)",
                line,
            )
            if match is None:
                raise ParseError(f"line {number}: invalid function declaration")
            name, raw_parameters, return_type = match.groups()
            _identifier(name, number)
            _type_name(return_type, number)
            parameters = []
            if raw_parameters.strip():
                for raw_parameter in raw_parameters.split(","):
                    parameter_name, separator, parameter_type = raw_parameter.partition(
                        ":"
                    )
                    if not separator:
                        raise ParseError(
                            f"line {number}: function parameter requires a type"
                        )
                    parameter_name = parameter_name.strip()
                    parameter_type = parameter_type.strip()
                    _identifier(parameter_name, number)
                    _type_name(parameter_type, number)
                    parameters.append(Parameter(parameter_name, parameter_type))
            body, position, terminator = _block(lines, position, allow_else=False)
            if terminator != "end":
                raise ParseError(f"line {number}: missing end")
            instructions.append(
                Function(name, tuple(parameters), return_type, tuple(body))
            )
        elif line.startswith("workflow "):
            name = line[9:].strip()
            _identifier(name, number)
            body, position, terminator = _block(lines, position, allow_else=False)
            if terminator != "end":
                raise ParseError(f"line {number}: missing end")
            instructions.append(Workflow(name, tuple(body)))
        elif line.startswith("run "):
            name = line[4:].strip()
            _identifier(name, number)
            instructions.append(Run(name))
        else:
            raise ParseError(f"line {number}: invalid instruction: {line}")
    return instructions, position, None


def _split_meta(text: str) -> tuple[str, str | None]:
    quoted = False
    escaped = False
    depth = 0
    marker = " meta "
    for index, character in enumerate(text):
        if escaped:
            escaped = False
            continue
        if character == "\\" and quoted:
            escaped = True
            continue
        if character == '"':
            quoted = not quoted
            continue
        if quoted:
            continue
        if character == "(":
            depth += 1
        elif character == ")":
            depth -= 1
        elif depth == 0 and text.startswith(marker, index):
            return text[:index].strip(), text[index + len(marker) :].strip()
    return text, None


def _memory_metadata(text: str, line: int):
    expression, raw_metadata = _split_meta(text)
    metadata = {"confidence": 100, "ttl_seconds": None, "source": "program"}
    if raw_metadata is None:
        return expression, metadata
    seen: set[str] = set()
    for item in raw_metadata.split():
        key, equals, value = item.partition("=")
        if not equals or key not in {"confidence", "ttl", "source"}:
            raise ParseError(f"line {line}: invalid memory metadata '{item}'")
        normalized_key = "ttl_seconds" if key == "ttl" else key
        if normalized_key in seen:
            raise ParseError(f"line {line}: duplicate memory metadata '{key}'")
        seen.add(normalized_key)
        if key == "confidence":
            if not value.isdigit() or not 0 <= int(value) <= 100:
                raise ParseError(f"line {line}: confidence must be 0..100")
            metadata["confidence"] = int(value)
        elif key == "ttl":
            if not value.isdigit() or int(value) < 1:
                raise ParseError(f"line {line}: ttl must be a positive integer")
            metadata["ttl_seconds"] = int(value)
        else:
            _identifier(value, line)
            metadata["source"] = value
    return expression.strip(), metadata


def _identifier(value: str, line: int) -> None:
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", value):
        raise ParseError(f"line {line}: invalid identifier '{value}'")
    if value in _RESERVED:
        raise ParseError(f"line {line}: reserved identifier '{value}'")


def _type_name(value: str, line: int) -> None:
    if value not in {"int", "string", "bool"}:
        raise ParseError(f"line {line}: unknown type '{value}'")
