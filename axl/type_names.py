MAX_TYPE_DEPTH = 16
_SCALARS = ("int", "string", "bool")


def split_type_name(type_name: str) -> tuple[int, str]:
    if not isinstance(type_name, str):
        raise TypeError("invalid type name")
    depth = 0
    current = type_name
    while current.startswith("list<") and current.endswith(">"):
        depth += 1
        if depth > MAX_TYPE_DEPTH:
            raise ValueError(f"type nesting is too deep ({MAX_TYPE_DEPTH})")
        current = current[5:-1]
    return depth, current


def validate_type_name(type_name: str) -> None:
    if not isinstance(type_name, str):
        raise TypeError("invalid type name")
    position = _parse_type(type_name, 0, 0, allow_unknown=True)
    if position != len(type_name):
        raise ValueError(f"invalid type '{type_name}'")


def is_known_type_name(type_name: str) -> bool:
    if not isinstance(type_name, str):
        return False
    try:
        position = _parse_type(type_name, 0, 0, allow_unknown=False)
    except (TypeError, ValueError):
        return False
    return position == len(type_name)


def split_map_type(type_name: str) -> tuple[str, str] | None:
    if not isinstance(type_name, str) or not type_name.startswith("map<"):
        return None
    depth = 0
    for position in range(4, len(type_name) - 1):
        character = type_name[position]
        if character == "<":
            depth += 1
        elif character == ">":
            depth -= 1
        elif character == "," and depth == 0 and type_name.endswith(">"):
            return type_name[4:position], type_name[position + 1 : -1]
    return None


def _parse_type(source: str, position: int, depth: int, *, allow_unknown: bool) -> int:
    for scalar in _SCALARS:
        if source.startswith(scalar, position):
            return position + len(scalar)
    if depth >= MAX_TYPE_DEPTH:
        raise ValueError(f"type nesting is too deep ({MAX_TYPE_DEPTH})")
    if source.startswith("list<", position):
        position = _parse_type(
            source, position + 5, depth + 1, allow_unknown=allow_unknown
        )
        if position >= len(source) or source[position] != ">":
            raise ValueError(f"invalid type '{source}'")
        return position + 1
    if source.startswith("map<", position):
        position = _parse_type(
            source, position + 4, depth + 1, allow_unknown=allow_unknown
        )
        if position >= len(source) or source[position] != ",":
            raise ValueError(f"invalid type '{source}'")
        position = _parse_type(
            source, position + 1, depth + 1, allow_unknown=allow_unknown
        )
        if position >= len(source) or source[position] != ">":
            raise ValueError(f"invalid type '{source}'")
        return position + 1
    if allow_unknown and position < len(source):
        end = position
        while end < len(source) and (source[end].isalnum() or source[end] == "_"):
            end += 1
        if end > position and (source[position].isalpha() or source[position] == "_"):
            return end
    raise ValueError(f"invalid type '{source}'")
