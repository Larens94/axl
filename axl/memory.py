import json
import sqlite3
from collections.abc import Mapping
from pathlib import Path
from typing import Protocol

from .ir import Value


class MemoryStore(Protocol):
    def get(self, key: str) -> Value | None: ...

    def set(self, key: str, value: Value) -> None: ...

    def snapshot(self) -> dict[str, Value]: ...


class InMemoryStore:
    def __init__(self, initial: Mapping[str, Value] | None = None):
        self._values = dict(initial or {})

    def get(self, key: str) -> Value | None:
        return self._values.get(key)

    def set(self, key: str, value: Value) -> None:
        self._values[key] = value

    def snapshot(self) -> dict[str, Value]:
        return dict(self._values)


class SQLiteMemoryStore:
    def __init__(self, path: str | Path):
        self.connection = sqlite3.connect(Path(path))
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS memory (key TEXT PRIMARY KEY, value_json TEXT NOT NULL)"
        )
        self.connection.commit()

    def get(self, key: str) -> Value | None:
        row = self.connection.execute(
            "SELECT value_json FROM memory WHERE key = ?", (key,)
        ).fetchone()
        return None if row is None else json.loads(row[0])

    def set(self, key: str, value: Value) -> None:
        payload = json.dumps(value, ensure_ascii=False)
        self.connection.execute(
            "INSERT INTO memory(key, value_json) VALUES(?, ?) "
            "ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            (key, payload),
        )
        self.connection.commit()

    def snapshot(self) -> dict[str, Value]:
        rows = self.connection.execute("SELECT key, value_json FROM memory ORDER BY key")
        return {key: json.loads(value_json) for key, value_json in rows}

    def close(self) -> None:
        self.connection.close()

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.close()
