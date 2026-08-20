import ast
import re
from dataclasses import dataclass

from .ir import Binary, Emit, If, Let, Literal, MemoryWrite, Program, Recall, ToolCall, Variable


class ParseError(ValueError):
    pass


_TOKEN = re.compile(
    r'(?P<STRING>"(?:\\.|[^"\\])*")|(?P<INT>\d+)|'
    r'(?P<BOOL>true|false)\b|(?P<IDENT>[A-Za-z_]\w*)|'
    r'(?P<OP>==|!=|>=|<=|[+\-*/(),<>])'
)
_PRECEDENCE = {"==": 1, "!=": 1, ">": 2, "<": 2, ">=": 2, "<=": 2, "+": 3, "-": 3, "*": 4, "/": 4}


@dataclass(frozen=True)
class Token:
    kind: str
    text: str


def _tokenize(text: str, line: int) -> list[Token]:
    tokens: list[Token] = []
    position = 0
    for match in _TOKEN.finditer(text):
        if text[position:match.start()].strip():
            raise ParseError(f"line {line}: invalid expression near '{text[position:]}'")
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
            raise ParseError(f"line {self.line}: unexpected token '{self.tokens[self.position].text}'")
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
            if self.position >= len(self.tokens) or self.tokens[self.position].text != ")":
                raise ParseError(f"line {self.line}: missing ')'")
            self.position += 1
            return expression
        if token.kind == "IDENT" and token.text == "recall":
            if self.position >= len(self.tokens) or self.tokens[self.position].kind != "IDENT":
                raise ParseError(f"line {self.line}: recall requires a memory key")
            key = self.tokens[self.position].text
            self.position += 1
            return Recall(key)
        if token.kind == "IDENT" and token.text == "call":
            return self._tool_call()
        if token.kind == "IDENT":
            return Variable(token.text)
        raise ParseError(f"line {self.line}: unexpected token '{token.text}'")

    def _tool_call(self):
        if self.position >= len(self.tokens) or self.tokens[self.position].kind != "IDENT":
            raise ParseError(f"line {self.line}: call requires a tool name")
        name = self.tokens[self.position].text
        self.position += 1
        if self.position >= len(self.tokens) or self.tokens[self.position].text != "(":
            raise ParseError(f"line {self.line}: tool arguments require '('")
        self.position += 1
        arguments = []
        if self.position < len(self.tokens) and self.tokens[self.position].text != ")":
            while True:
                arguments.append(self._binary(0))
                if self.position < len(self.tokens) and self.tokens[self.position].text == ",":
                    self.position += 1
                    continue
                break
        if self.position >= len(self.tokens) or self.tokens[self.position].text != ")":
            raise ParseError(f"line {self.line}: tool call missing ')'")
        self.position += 1
        return ToolCall(name, tuple(arguments))


def _expression(text: str, line: int):
    return ExpressionParser(text, line).parse()


def parse(source: str) -> Program:
    lines = [(number, raw.strip()) for number, raw in enumerate(source.splitlines(), 1)]
    instructions, position, terminator = _block(lines, 0, allow_else=False)
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
            instructions.append(MemoryWrite(key, _expression(right.strip(), number)))
        elif line.startswith("let ") and "=" in line:
            left, right = line[4:].split("=", 1)
            target = left.strip()
            _identifier(target, number)
            instructions.append(Let(target, _expression(right.strip(), number)))
        elif line.startswith("emit "):
            instructions.append(Emit(_expression(line[5:].strip(), number)))
        elif line.startswith("if "):
            body, position, terminator = _block(lines, position, allow_else=True)
            if terminator is None:
                raise ParseError(f"line {number}: missing end")
            else_body = []
            if terminator == "else":
                else_body, position, terminator = _block(lines, position, allow_else=False)
                if terminator != "end":
                    raise ParseError(f"line {number}: missing end")
            instructions.append(
                If(_expression(line[3:].strip(), number), tuple(body), tuple(else_body))
            )
        else:
            raise ParseError(f"line {number}: invalid instruction: {line}")
    return instructions, position, None


def _identifier(value: str, line: int) -> None:
    if not re.fullmatch(r"[A-Za-z_]\w*", value):
        raise ParseError(f"line {line}: invalid identifier '{value}'")
