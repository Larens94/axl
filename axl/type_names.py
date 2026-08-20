MAX_TYPE_DEPTH = 16


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
