from dataclasses import dataclass


@dataclass(frozen=True)
class MemoryWrite:
    key: str
    value: str


@dataclass(frozen=True)
class MemoryRecall:
    target: str
    key: str


@dataclass(frozen=True)
class Emit:
    variable: str


Instruction = MemoryWrite | MemoryRecall | Emit


@dataclass(frozen=True)
class Program:
    instructions: tuple[Instruction, ...]
