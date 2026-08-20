import json
import sys
from collections.abc import Callable
from dataclasses import dataclass

from .ir import (
    Agent,
    Binary,
    Emit,
    Expression,
    Forget,
    Function,
    FunctionCall,
    If,
    Let,
    ListExpression,
    Literal,
    MapExpression,
    MapValue,
    MemoryWrite,
    Program,
    Recall,
    Return,
    Run,
    ToolCall,
    Value,
    Variable,
    While,
    Workflow,
)
from .memory import InMemoryStore, MemoryStore
from .policy import ApprovalRequest, ApprovalRequired, AuditEvent, Tool, validate_tool
from .validation import validate

MAX_VALUE_DEPTH = 256


class RuntimeError(Exception):
    """Raised when a valid AXL program cannot be executed."""


def render_value(value: Value) -> str:
    if type(value) is bool:
        return str(value).lower()
    if type(value) is tuple or isinstance(value, MapValue):
        try:
            return json.dumps(
                _json_value(value), ensure_ascii=False, separators=(",", ":")
            )
        except (UnicodeEncodeError, ValueError) as error:
            raise RuntimeError("output value cannot be rendered") from error
    if type(value) is int:
        digit_limit = sys.get_int_max_str_digits()
        estimated_digits = (value.bit_length() * 30103) // 100000 + 1
        if digit_limit and estimated_digits >= digit_limit:
            raise RuntimeError("integer output is too large")
    try:
        return str(value)
    except ValueError as error:
        raise RuntimeError("integer output is too large") from error


def _json_value(value: Value):
    if type(value) is tuple:
        return [_json_value(item) for item in value]
    if isinstance(value, MapValue):
        entries = sorted(
            value.entries, key=lambda entry: (type(entry[0]).__name__, entry[0])
        )
        return {
            "$ax.map": [[_json_value(key), _json_value(item)] for key, item in entries]
        }
    return value


def _unify_value_shapes(left, right, context: str):
    if left == "any":
        return right
    if right == "any" or left == right:
        return left
    if (
        isinstance(left, tuple)
        and isinstance(right, tuple)
        and left[0] == right[0]
        and len(left) == len(right)
    ):
        return (left[0],) + tuple(
            _unify_value_shapes(a, b, context)
            for a, b in zip(left[1:], right[1:], strict=True)
        )
    raise RuntimeError(f"{context} must have one type")


class _FunctionReturn(Exception):
    def __init__(self, value: Value):
        self.value = value


@dataclass(frozen=True)
class ExecutionResult:
    output: list[Value]
    memory: dict[str, Value]
    audit: list[AuditEvent]


class Interpreter:
    def __init__(
        self,
        tools: dict[str, Callable[..., Value]] | list[Tool] | None = None,
        max_steps: int = 10_000,
        memory_store: MemoryStore | None = None,
        approve: Callable[[ApprovalRequest], bool] | None = None,
        scope: str = "session:default",
        max_output_bytes: int = 1_000_000,
        max_value_bytes: int = 1_000_000,
        max_value_nodes: int = 100_000,
        max_value_depth: int = 256,
        max_tool_calls: int = 100,
        max_memory_ops: int = 1_000,
        max_function_depth: int = 256,
    ):
        if (
            min(
                max_steps,
                max_output_bytes,
                max_value_bytes,
                max_value_nodes,
                max_value_depth,
                max_tool_calls,
                max_memory_ops,
                max_function_depth,
            )
            < 1
        ):
            raise ValueError("runtime budgets must be positive")
        if max_value_depth > MAX_VALUE_DEPTH:
            raise ValueError(f"max_value_depth cannot exceed {MAX_VALUE_DEPTH}")
        if isinstance(tools, dict):
            tool_list = [Tool(name, handler) for name, handler in tools.items()]
        else:
            tool_list = list(tools or [])
        for tool in tool_list:
            validate_tool(tool)
        names = [tool.name for tool in tool_list]
        if len(names) != len(set(names)):
            raise ValueError("duplicate tool name")
        self.tools = {tool.name: tool for tool in tool_list}
        self.max_steps = max_steps
        self.memory_store = memory_store or InMemoryStore()
        configure_limits = getattr(self.memory_store, "configure_limits", None)
        if configure_limits is not None:
            configure_limits(
                max_bytes=max_value_bytes,
                max_nodes=max_value_nodes,
                max_depth=max_value_depth,
            )
        self.approve = approve
        self.scope = scope
        self.max_output_bytes = max_output_bytes
        self.max_value_bytes = max_value_bytes
        self.max_value_nodes = max_value_nodes
        self.max_value_depth = max_value_depth
        self.max_tool_calls = max_tool_calls
        self.max_memory_ops = max_memory_ops
        self.max_function_depth = max_function_depth
        self.audit: list[AuditEvent] = []
        self.runnables: dict[str, Agent | Workflow] = {}
        self.current_agent: Agent | None = None

    def run(self, program: Program) -> ExecutionResult:
        validate(program)
        self.audit = []
        self.variables: dict[str, Value] = {}
        self.output: list[Value] = []
        self.steps = 0
        self.output_bytes = 0
        self.tool_calls = 0
        self.memory_ops = 0
        self.function_depth = 0
        self.runnables = {
            instruction.name: instruction
            for instruction in program.instructions
            if isinstance(instruction, (Agent, Workflow))
        }
        self.functions = {
            instruction.name: instruction
            for instruction in program.instructions
            if isinstance(instruction, Function)
        }
        self._execute(program.instructions)
        try:
            snapshot = self.memory_store.snapshot(self.scope)
        except (RecursionError, TypeError, ValueError) as error:
            raise RuntimeError("memory snapshot is invalid") from error
        memory = {
            key: self._bounded_value(value, f"memory '{key}'")
            for key, value in snapshot.items()
        }
        return ExecutionResult(
            output=self.output,
            memory=memory,
            audit=list(self.audit),
        )

    def _render_output(self, value: Value) -> str:
        return render_value(value)

    def _execute(self, instructions) -> None:
        for instruction in instructions:
            self._step()
            if isinstance(instruction, MemoryWrite):
                self._memory_op()
                self.memory_store.set(
                    instruction.key,
                    self._evaluate(instruction.value),
                    self.scope,
                    confidence=instruction.confidence,
                    ttl_seconds=instruction.ttl_seconds,
                    source=instruction.source,
                )
            elif isinstance(instruction, Forget):
                self._memory_op()
                self.memory_store.delete(instruction.key, self.scope)
            elif isinstance(instruction, Let):
                self.variables[instruction.target] = self._evaluate(instruction.value)
            elif isinstance(instruction, Return):
                raise _FunctionReturn(self._evaluate(instruction.value))
            elif isinstance(instruction, Emit):
                value = self._evaluate(instruction.value)
                rendered = self._render_output(value)
                size = len(rendered.encode("utf-8"))
                if self.output_bytes + size > self.max_output_bytes:
                    raise RuntimeError(
                        f"output budget exceeded ({self.max_output_bytes} bytes)"
                    )
                self.output_bytes += size
                self.output.append(value)
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
            elif isinstance(instruction, (Agent, Workflow, Function)):
                continue
            elif isinstance(instruction, Run):
                runnable = self.runnables.get(instruction.name)
                if runnable is None:
                    raise RuntimeError(
                        f"unknown agent or workflow '{instruction.name}'"
                    )
                previous_agent = self.current_agent
                previous_variables = self.variables
                if isinstance(runnable, Agent):
                    self.current_agent = runnable
                    self.variables = {}
                try:
                    self._execute(runnable.body)
                finally:
                    self.current_agent = previous_agent
                    self.variables = previous_variables

    def _step(self) -> None:
        self.steps += 1
        if self.steps > self.max_steps:
            raise RuntimeError(f"execution budget exceeded ({self.max_steps} steps)")

    def _memory_op(self) -> None:
        self.memory_ops += 1
        if self.memory_ops > self.max_memory_ops:
            raise RuntimeError(
                f"memory operation budget exceeded ({self.max_memory_ops})"
            )

    def _tool_call(self) -> None:
        self.tool_calls += 1
        if self.tool_calls > self.max_tool_calls:
            raise RuntimeError(f"tool call budget exceeded ({self.max_tool_calls})")

    def _bounded_value(self, value, context: str = "value") -> Value:
        size = 0
        nodes = 0
        stack = [(value, 0)]
        while stack:
            item, depth = stack.pop()
            nodes += 1
            if nodes > self.max_value_nodes:
                raise RuntimeError(
                    f"value node budget exceeded ({self.max_value_nodes})"
                )
            if type(item) is tuple:
                size += 1
                if depth >= self.max_value_depth:
                    raise RuntimeError(f"value nesting exceeds {self.max_value_depth}")
                stack.extend((child, depth + 1) for child in item)
            elif isinstance(item, MapValue):
                size += 1
                if depth >= self.max_value_depth:
                    raise RuntimeError(f"value nesting exceeds {self.max_value_depth}")
                if type(item.entries) is not tuple or any(
                    type(entry) is not tuple or len(entry) != 2
                    for entry in item.entries
                ):
                    raise RuntimeError(f"{context} contains invalid map")
                stack.extend(
                    (child, depth + 1) for entry in item.entries for child in entry
                )
            elif type(item) is str:
                try:
                    size += len(item.encode("utf-8"))
                except UnicodeEncodeError as error:
                    raise RuntimeError(f"{context} contains invalid Unicode") from error
            elif type(item) is int:
                size += max(1, (item.bit_length() + 7) // 8)
            elif type(item) is bool:
                size += 1
            else:
                raise RuntimeError(
                    f"{context} contains invalid value '{type(item).__name__}'"
                )
            if size > self.max_value_bytes:
                raise RuntimeError(
                    f"value budget exceeded ({self.max_value_bytes} bytes)"
                )
        if type(value) is tuple or isinstance(value, MapValue):
            self._value_shape(value)
        return value

    def _value_shape(self, value: Value):
        work = [(value, 0, False)]
        shapes = []
        while work:
            item, depth, visited = work.pop()
            if type(item) is tuple:
                if depth >= self.max_value_depth:
                    raise RuntimeError(f"value nesting exceeds {self.max_value_depth}")
                if not visited:
                    work.append((item, depth, True))
                    work.extend((child, depth + 1, False) for child in reversed(item))
                    continue
                children = shapes[-len(item) :] if item else []
                if item:
                    del shapes[-len(item) :]
                item_shape = "any"
                for child in children:
                    item_shape = _unify_value_shapes(item_shape, child, "list items")
                shapes.append(("list", item_shape))
            elif isinstance(item, MapValue):
                if depth >= self.max_value_depth:
                    raise RuntimeError(f"value nesting exceeds {self.max_value_depth}")
                if not visited:
                    work.append((item, depth, True))
                    work.extend(
                        (child, depth + 1, False)
                        for entry in reversed(item.entries)
                        for child in reversed(entry)
                    )
                    continue
                width = len(item.entries) * 2
                children = shapes[-width:] if width else []
                if width:
                    del shapes[-width:]
                key_shapes = children[::2]
                value_shapes = children[1::2]
                keys = [key for key, _ in item.entries]
                if any(type(key) not in (str, int, bool) for key in keys):
                    raise RuntimeError("map keys must be scalar")
                identities = [(type(key), key) for key in keys]
                if len(identities) != len(set(identities)):
                    raise RuntimeError("map keys must be unique")
                key_shape = "any"
                for child in key_shapes:
                    key_shape = _unify_value_shapes(key_shape, child, "map keys")
                value_shape = "any"
                for child in value_shapes:
                    value_shape = _unify_value_shapes(value_shape, child, "map values")
                shapes.append(("map", key_shape, value_shape))
            elif type(item) is bool:
                shapes.append("bool")
            elif type(item) is int:
                shapes.append("int")
            elif type(item) is str:
                shapes.append("string")
            else:
                raise RuntimeError(
                    f"value contains invalid value '{type(item).__name__}'"
                )
        return shapes[0]

    def _evaluate(self, expression: Expression) -> Value:
        self._step()
        if isinstance(expression, Literal):
            return self._bounded_value(expression.value)
        if isinstance(expression, ListExpression):
            return self._bounded_value(
                tuple(self._evaluate(item) for item in expression.items)
            )
        if isinstance(expression, MapExpression):
            entries = tuple(
                (self._evaluate(key), self._evaluate(value))
                for key, value in expression.entries
            )
            return self._bounded_value(MapValue(entries))
        if isinstance(expression, Variable):
            if expression.name not in self.variables:
                raise RuntimeError(f"unknown variable '{expression.name}'")
            return self._bounded_value(self.variables[expression.name])
        if isinstance(expression, Recall):
            self._memory_op()
            try:
                value = self.memory_store.get(expression.key, self.scope)
            except (RecursionError, TypeError, ValueError) as error:
                raise RuntimeError(f"memory '{expression.key}' is invalid") from error
            if value is None:
                raise RuntimeError(f"unknown memory '{expression.key}'")
            return self._bounded_value(value, f"memory '{expression.key}'")
        if isinstance(expression, ToolCall):
            self._tool_call()
            if expression.name not in self.tools:
                request = ApprovalRequest(expression.name, (), "unknown")
                self.audit.append(AuditEvent.create(request, "denied"))
                raise RuntimeError(f"tool '{expression.name}' is not allowed")
            tool = self.tools[expression.name]
            if (
                self.current_agent is not None
                and tool.name not in self.current_agent.tools
            ):
                request = ApprovalRequest(tool.name, (), tool.effect)
                self.audit.append(AuditEvent.create(request, "denied"))
                raise RuntimeError(
                    f"tool '{tool.name}' not granted to agent '{self.current_agent.name}'"
                )
            arguments = tuple(
                self._evaluate(argument) for argument in expression.arguments
            )
            request = ApprovalRequest(tool.name, arguments, tool.effect)
            if tool.approval:
                if self.approve is None:
                    self.audit.append(AuditEvent.create(request, "approval_required"))
                    raise ApprovalRequired(f"tool '{tool.name}' requires approval")
                try:
                    approved = self.approve(request)
                except Exception as error:
                    self.audit.append(AuditEvent.create(request, "denied"))
                    raise ApprovalRequired(
                        f"tool '{tool.name}' approval provider failed"
                    ) from error
                if approved is not True:
                    self.audit.append(AuditEvent.create(request, "denied"))
                    raise ApprovalRequired(f"tool '{tool.name}' denied")
                self.audit.append(AuditEvent.create(request, "approved"))
            try:
                result = tool.handler(*arguments)
            except Exception as error:
                self.audit.append(AuditEvent.create(request, "failed"))
                raise RuntimeError(
                    f"tool '{expression.name}' failed: {error}"
                ) from error
            if type(result) not in (str, int, bool, tuple) and not isinstance(
                result, MapValue
            ):
                self.audit.append(AuditEvent.create(request, "failed"))
                raise RuntimeError(
                    f"tool '{tool.name}' returned invalid value '{type(result).__name__}'"
                )
            try:
                result = self._bounded_value(result, f"tool '{tool.name}'")
            except RuntimeError:
                self.audit.append(AuditEvent.create(request, "failed"))
                raise
            self.audit.append(AuditEvent.create(request, "executed"))
            return result
        if isinstance(expression, FunctionCall):
            function = self.functions.get(expression.name)
            if function is None:
                raise RuntimeError(f"unknown function '{expression.name}'")
            if self.function_depth >= self.max_function_depth:
                raise RuntimeError(
                    f"function call depth exceeded ({self.max_function_depth})"
                )
            arguments = tuple(
                self._evaluate(argument) for argument in expression.arguments
            )
            previous_variables = self.variables
            self.variables = {
                parameter.name: value
                for parameter, value in zip(function.parameters, arguments, strict=True)
            }
            self.function_depth += 1
            try:
                self._execute(function.body)
            except _FunctionReturn as returned:
                return self._bounded_value(returned.value)
            finally:
                self.function_depth -= 1
                self.variables = previous_variables
            raise RuntimeError(f"function '{function.name}' completed without return")
        if isinstance(expression, Binary):
            return self._bounded_value(
                self._binary(
                    self._evaluate(expression.left),
                    expression.operator,
                    self._evaluate(expression.right),
                )
            )
        raise RuntimeError("unsupported expression")

    def _binary(self, left: Value, operator: str, right: Value) -> Value:
        if operator in {"==", "!="}:
            if type(left) is not type(right):
                raise RuntimeError(f"invalid operands for '{operator}'")
            return left == right if operator == "==" else left != right

        if operator == "+" and type(left) is str and type(right) is str:
            return left + right

        if type(left) is not int or type(right) is not int:
            raise RuntimeError(f"invalid operands for '{operator}'")
        if operator == "+":
            return left + right
        if operator == "-":
            return left - right
        if operator == "*":
            return left * right
        if operator == "/":
            if right == 0:
                raise RuntimeError("division by zero")
            if left % right != 0:
                raise RuntimeError("non-integer division is not supported")
            return left // right
        if operator == ">":
            return left > right
        if operator == "<":
            return left < right
        if operator == ">=":
            return left >= right
        if operator == "<=":
            return left <= right
        raise RuntimeError(f"unknown operator '{operator}'")
