from dataclasses import dataclass

from .ir import Emit, MemoryRecall, MemoryWrite, Program


class RuntimeError(Exception):
    """Raised when a valid AXL program cannot be executed."""


@dataclass(frozen=True)
class ExecutionResult:
    output: list[str]
    memory: dict[str, str]


class Interpreter:
    def run(self, program: Program) -> ExecutionResult:
        memory: dict[str, str] = {}
        variables: dict[str, str] = {}
        output: list[str] = []

        for instruction in program.instructions:
            if isinstance(instruction, MemoryWrite):
                memory[instruction.key] = instruction.value
            elif isinstance(instruction, MemoryRecall):
                if instruction.key not in memory:
                    raise RuntimeError(f"unknown memory '{instruction.key}'")
                variables[instruction.target] = memory[instruction.key]
            elif isinstance(instruction, Emit):
                output.append(variables[instruction.variable])

        return ExecutionResult(output=output, memory=memory)
