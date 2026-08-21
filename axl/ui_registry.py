"""AX-UI Registry 1: stable numeric contracts for the first web slice."""

from dataclasses import dataclass


@dataclass(frozen=True)
class ComponentContract:
    name: str
    properties: dict[int, str]
    events: frozenset[int] = frozenset()
    children: bool = False


COMPONENTS = {
    1: ComponentContract("app", {1: "string"}, children=True),
    2: ComponentContract(
        "hero",
        {1: "string", 2: "string", 3: "string", 4: "string", 5: "string"},
        frozenset({1, 2}),
    ),
    3: ComponentContract("shelf", {1: "string"}, children=True),
    4: ComponentContract(
        "media-card",
        {1: "string", 2: "string", 3: "int", 4: "int"},
        frozenset({1}),
    ),
}

ANNOTATION_KINDS = {1: "purpose", 2: "export", 3: "rule"}
