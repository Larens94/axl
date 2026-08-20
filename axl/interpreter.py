from collections.abc import Callable
from dataclasses import dataclass

from .ir import Binary, Emit, Expression, If, Let, Literal, MemoryWrite, Program, Recall, ToolCall, Value, Variable, While
from .memory import InMemoryStore, MemoryStore


class RuntimeError(Exception):
    """Raised when a valid AXL program cannot be executed."""


@dataclass(frozen=True)
class ExecutionResult:
    output: list[Value]
    memory: dict[str, Value]


class Interpreter:
    def __init__(
        self,
        tools: dict[str, Callable[..., Value]] | None = None,
        max_steps: int = 10_000,
        memory_store: MemoryStore | None = None,
    ):
        if max_steps < 1:
            raise ValueError("max_steps must be positive")
        self.tools = dict(tools or {})
        self.max_steps = max_steps
        self.memory_store = memory_store or InMemoryStore()

    def run(self, program: Program) -> ExecutionResult:
        self.variables: dict[str, Value] = {}
        self.output: list[Value] = []
        self.steps = 0
        self._execute(program.instructions)
        return ExecutionResult(output=self.output, memory=self.memory_store.snapshot())

    def _execute(self, instructions) -> None:
        for instruction in instructions:
            self._step()
            if isinstance(instruction, MemoryWrite):
                self.memory_store.set(instruction.key, self._evaluate(instruction.value))
            elif isinstance(instruction, Let):
                self.variables[instruction.target] = self._evaluate(instruction.value)
            elif isinstance(instruction, Emit):
                self.output.append(self._evaluate(instruction.value))
            elif isinstance(instruction, If):
                condition = self._evaluate(instruction.condition)
                if not isinstance(condition, bool):
                    raise RuntimeError("if condition must be boolean")
                if condition:
                    self._execute(instruction.body)
                else:
                    self._execute(instruction.else_body)
            elif isinstance(instruction, While):
                while True:
                    condition = self._evaluate(instruction.condition)
                    if not isinstance(condition, bool):
                        raise RuntimeError("while condition must be boolean")
                    if not condition:
                        break
                    self._step()
                    self._execute(instruction.body)

    def _step(self) -> None:
        self.steps += 1
        if self.steps > self.max_steps:
            raise RuntimeError(f"execution budget exceeded ({self.max_steps} steps)")

    def _evaluate(self, expression: Expression) -> Value:
        if isinstance(expression, Literal):
            return expression.value
        if isinstance(expression, Variable):
            if expression.name not in self.variables:
                raise RuntimeError(f"unknown variable '{expression.name}'")
            return self.variables[expression.name]
        if isinstance(expression, Recall):
            value = self.memory_store.get(expression.key)
            if value is None:
                raise RuntimeError(f"unknown memory '{expression.key}'")
            return value
        if isinstance(expression, ToolCall):
            if expression.name not in self.tools:
                raise RuntimeError(f"tool '{expression.name}' is not allowed")
            arguments = [self._evaluate(argument) for argument in expression.arguments]
            try:
                return self.tools[expression.name](*arguments)
            except Exception as error:
                raise RuntimeError(f"tool '{expression.name}' failed: {error}") from error
        if isinstance(expression, Binary):
            return self._binary(
                self._evaluate(expression.left),
                expression.operator,
                self._evaluate(expression.right),
            )
        raise RuntimeError("unsupported expression")

    def _binary(self, left: Value, operator: str, right: Value) -> Value:
        try:
            if operator == "+":
                return left + right  # type: ignore[operator]
            if operator == "-":
                return left - right  # type: ignore[operator]
            if operator == "*":
                return left * right  # type: ignore[operator]
            if operator == "/":
                if right == 0:
                    raise RuntimeError("division by zero")
                result = left / right  # type: ignore[operator]
                return int(result) if isinstance(result, float) and result.is_integer() else result  # type: ignore[return-value]
            if operator == "==":
                return left == right
            if operator == "!=":
                return left != right
            if operator == ">":
                return left > right  # type: ignore[operator]
            if operator == "<":
                return left < right  # type: ignore[operator]
            if operator == ">=":
                return left >= right  # type: ignore[operator]
            if operator == "<=":
                return left <= right  # type: ignore[operator]
        except TypeError as error:
            raise RuntimeError(f"invalid operands for '{operator}'") from error
        raise RuntimeError(f"unknown operator '{operator}'")
