import json
import sqlite3
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Protocol

from .ir import Value

DEFAULT_SCOPE = "session:default"


@dataclass(frozen=True)
class MemoryRecord:
    key: str
    scope: str
    value: Value
    version: int
    updated_at: str
    confidence: int = 100
    source: str = "program"
    expires_at: str | None = None


class MemoryStore(Protocol):
    def get(self, key: str, scope: str = DEFAULT_SCOPE) -> Value | None: ...
    def set(
        self,
        key: str,
        value: Value,
        scope: str = DEFAULT_SCOPE,
        *,
        confidence: int = 100,
        ttl_seconds: int | None = None,
        source: str = "program",
    ) -> None: ...
    def delete(self, key: str, scope: str = DEFAULT_SCOPE) -> bool: ...
    def snapshot(self, scope: str = DEFAULT_SCOPE) -> dict[str, Value]: ...


class InMemoryStore:
    def __init__(self, initial: Mapping[str, Value] | None = None):
        self._values: dict[tuple[str, str], MemoryRecord] = {}
        for key, value in (initial or {}).items():
            self.set(key, value)

    def get(self, key: str, scope: str = DEFAULT_SCOPE) -> Value | None:
        record = self._values.get((scope, key))
        if record is not None and _expired(record.expires_at):
            self.delete(key, scope)
            return None
        return None if record is None else record.value

    def set(
        self,
        key: str,
        value: Value,
        scope: str = DEFAULT_SCOPE,
        *,
        confidence: int = 100,
        ttl_seconds: int | None = None,
        source: str = "program",
    ) -> None:
        previous = self._values.get((scope, key))
        self._values[(scope, key)] = MemoryRecord(
            key,
            scope,
            value,
            1 if previous is None else previous.version + 1,
            datetime.now(UTC).isoformat(),
            confidence,
            source,
            _expiry(ttl_seconds),
        )

    def delete(self, key: str, scope: str = DEFAULT_SCOPE) -> bool:
        return self._values.pop((scope, key), None) is not None

    def snapshot(self, scope: str = DEFAULT_SCOPE) -> dict[str, Value]:
        keys = [key for (record_scope, key) in self._values if record_scope == scope]
        return {
            key: value for key in keys if (value := self.get(key, scope)) is not None
        }

    def inspect(self, key: str, scope: str = DEFAULT_SCOPE) -> MemoryRecord | None:
        if self.get(key, scope) is None:
            return None
        return self._values.get((scope, key))


class SQLiteMemoryStore:
    def __init__(self, path: str | Path):
        self.connection = sqlite3.connect(str(path))
        self._initialize()

    def _initialize(self) -> None:
        exists = self.connection.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='memory'"
        ).fetchone()
        if exists:
            columns = {
                row[1] for row in self.connection.execute("PRAGMA table_info(memory)")
            }
            if "scope" not in columns:
                self.connection.execute("ALTER TABLE memory RENAME TO memory_legacy")
                self._create_table()
                self.connection.execute(
                    "INSERT INTO memory(scope,key,value_json,version,updated_at) "
                    "SELECT ?,key,value_json,1,? FROM memory_legacy",
                    (DEFAULT_SCOPE, datetime.now(UTC).isoformat()),
                )
                self.connection.execute("DROP TABLE memory_legacy")
        else:
            self._create_table()
        columns = {
            row[1] for row in self.connection.execute("PRAGMA table_info(memory)")
        }
        additions = {
            "confidence": "INTEGER NOT NULL DEFAULT 100",
            "source": "TEXT NOT NULL DEFAULT 'program'",
            "expires_at": "TEXT",
        }
        for name, definition in additions.items():
            if name not in columns:
                self.connection.execute(
                    f"ALTER TABLE memory ADD COLUMN {name} {definition}"
                )
        self.connection.commit()

    def _create_table(self) -> None:
        self.connection.execute(
            "CREATE TABLE memory ("
            "scope TEXT NOT NULL, key TEXT NOT NULL, value_json TEXT NOT NULL, "
            "version INTEGER NOT NULL, updated_at TEXT NOT NULL, "
            "confidence INTEGER NOT NULL DEFAULT 100, source TEXT NOT NULL DEFAULT 'program', "
            "expires_at TEXT, PRIMARY KEY(scope,key))"
        )

    def get(self, key: str, scope: str = DEFAULT_SCOPE) -> Value | None:
        row = self.connection.execute(
            "SELECT value_json,expires_at FROM memory WHERE scope=? AND key=?",
            (scope, key),
        ).fetchone()
        if row is not None and _expired(row[1]):
            self.delete(key, scope)
            return None
        return None if row is None else json.loads(row[0])

    def set(
        self,
        key: str,
        value: Value,
        scope: str = DEFAULT_SCOPE,
        *,
        confidence: int = 100,
        ttl_seconds: int | None = None,
        source: str = "program",
    ) -> None:
        payload = json.dumps(value, ensure_ascii=False)
        now = datetime.now(UTC).isoformat()
        self.connection.execute(
            "INSERT INTO memory(scope,key,value_json,version,updated_at,confidence,source,expires_at) "
            "VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(scope,key) DO UPDATE SET "
            "value_json=excluded.value_json, version=memory.version+1, "
            "updated_at=excluded.updated_at, confidence=excluded.confidence, "
            "source=excluded.source, expires_at=excluded.expires_at",
            (scope, key, payload, 1, now, confidence, source, _expiry(ttl_seconds)),
        )
        self.connection.commit()

    def delete(self, key: str, scope: str = DEFAULT_SCOPE) -> bool:
        cursor = self.connection.execute(
            "DELETE FROM memory WHERE scope=? AND key=?", (scope, key)
        )
        self.connection.commit()
        return cursor.rowcount > 0

    def snapshot(self, scope: str = DEFAULT_SCOPE) -> dict[str, Value]:
        rows = list(
            self.connection.execute(
                "SELECT key FROM memory WHERE scope=? ORDER BY key", (scope,)
            )
        )
        return {
            key: value for (key,) in rows if (value := self.get(key, scope)) is not None
        }

    def inspect(self, key: str, scope: str = DEFAULT_SCOPE) -> MemoryRecord | None:
        if self.get(key, scope) is None:
            return None
        row = self.connection.execute(
            "SELECT value_json,version,updated_at,confidence,source,expires_at "
            "FROM memory WHERE scope=? AND key=?",
            (scope, key),
        ).fetchone()
        return (
            None
            if row is None
            else MemoryRecord(
                key, scope, json.loads(row[0]), row[1], row[2], row[3], row[4], row[5]
            )
        )

    def close(self) -> None:
        self.connection.close()

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.close()


def _expiry(ttl_seconds: int | None) -> str | None:
    if ttl_seconds is None:
        return None
    return (datetime.now(UTC) + timedelta(seconds=ttl_seconds)).isoformat()


def _expired(expires_at: str | None) -> bool:
    return expires_at is not None and datetime.fromisoformat(
        expires_at
    ) <= datetime.now(UTC)
