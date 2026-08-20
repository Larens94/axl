from axl import Tool


def search(query):
    return f"research:{query}"


def publish(content):
    return f"published:{content}"


def tools():
    return [
        Tool("search", search, effect="read"),
        Tool("publish", publish, effect="write", approval=True),
    ]
