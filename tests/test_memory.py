import sqlite3
import tempfile
import unittest
from datetime import UTC, datetime
from pathlib import Path
from unittest.mock import patch

from axl import Interpreter, RuntimeError, SQLiteMemoryStore, parse


class PersistentMemoryTest(unittest.TestCase):
    def test_sqlite_object_is_rejected_before_json_materialization(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "object.sqlite")
            payload = '{"x":' * 600 + "0" + "}" * 600
            store.connection.execute(
                "INSERT INTO memory(scope,key,value_json,version,updated_at) "
                "VALUES(?,?,?,?,?)",
                (
                    "session:default",
                    "object",
                    payload,
                    1,
                    datetime.now(UTC).isoformat(),
                ),
            )
            store.connection.commit()

            try:
                with (
                    patch("axl.memory.json.loads") as loads,
                    self.assertRaisesRegex(RuntimeError, "memory 'object' is invalid"),
                ):
                    Interpreter(
                        memory_store=store, max_value_bytes=10_000, max_value_nodes=2
                    ).run(parse("emit recall object"))
                loads.assert_not_called()
            finally:
                store.close()

    def test_sqlite_blob_is_a_controlled_memory_error(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "blob.sqlite")
            store.connection.execute(
                "INSERT INTO memory(scope,key,value_json,version,updated_at) "
                "VALUES(?,?,?,?,?)",
                (
                    "session:default",
                    "blob",
                    sqlite3.Binary(b"[1]"),
                    1,
                    datetime.now(UTC).isoformat(),
                ),
            )
            store.connection.commit()

            try:
                with self.assertRaisesRegex(RuntimeError, "memory 'blob' is invalid"):
                    Interpreter(memory_store=store).run(parse("emit recall blob"))
            finally:
                store.close()

    def test_memory_values_round_trip_at_semantic_byte_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "boundary.sqlite")
            try:
                string_result = Interpreter(memory_store=store, max_value_bytes=1).run(
                    parse('memory value = "a"')
                )
                list_result = Interpreter(memory_store=store, max_value_bytes=2).run(
                    parse("2;20|value|#0,~1")
                )
            finally:
                store.close()

        self.assertEqual(string_result.memory["value"], "a")
        self.assertEqual(list_result.memory["value"], (0,))

    def test_wide_sqlite_value_is_rejected_before_json_materialization(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "wide.sqlite")
            payload = "[" + ",".join("0" for _ in range(200_000)) + "]"
            store.connection.execute(
                "INSERT INTO memory(scope,key,value_json,version,updated_at) "
                "VALUES(?,?,?,?,?)",
                ("session:default", "wide", payload, 1, datetime.now(UTC).isoformat()),
            )
            store.connection.commit()
            interpreter = Interpreter(
                memory_store=store, max_value_bytes=2, max_value_nodes=2
            )

            with (
                patch("axl.memory.json.loads") as loads,
                self.assertRaisesRegex(RuntimeError, "memory 'wide' is invalid"),
            ):
                interpreter.run(parse("emit recall wide"))
            loads.assert_not_called()
            store.close()

    def test_deep_sqlite_value_is_a_controlled_runtime_error(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "deep.sqlite")
            payload = "[" * 1_200 + "0" + "]" * 1_200
            store.connection.execute(
                "INSERT INTO memory(scope,key,value_json,version,updated_at) "
                "VALUES(?,?,?,?,?)",
                ("session:default", "deep", payload, 1, datetime.now(UTC).isoformat()),
            )
            store.connection.commit()

            try:
                with self.assertRaisesRegex(RuntimeError, "memory 'deep' is invalid"):
                    Interpreter(memory_store=store).run(parse("emit recall deep"))
            finally:
                store.close()

    def test_sqlite_memory_survives_interpreter_instances(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "agent-memory.sqlite"
            first_store = SQLiteMemoryStore(path)
            Interpreter(memory_store=first_store).run(
                parse('memory preference = "concise"')
            )
            first_store.close()

            second_store = SQLiteMemoryStore(path)
            result = Interpreter(memory_store=second_store).run(
                parse("let style = recall preference\nemit style")
            )
            second_store.close()

        self.assertEqual(result.output, ["concise"])
        self.assertEqual(result.memory, {"preference": "concise"})

    def test_sqlite_memory_round_trips_list_values(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lists.sqlite"
            first_store = SQLiteMemoryStore(path)
            Interpreter(memory_store=first_store).run(parse("2;20|xs|#1,#2,#3,~3"))
            first_store.close()

            second_store = SQLiteMemoryStore(path)
            result = Interpreter(memory_store=second_store).run(parse("2;12|@xs"))
            second_store.close()

        self.assertEqual(result.output, [(1, 2, 3)])

    def test_sqlite_memory_preserves_value_types(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "typed.sqlite")
            Interpreter(memory_store=store).run(
                parse("memory retries = 3\nmemory enabled = true")
            )
            result = Interpreter(memory_store=store).run(
                parse("emit recall retries\nemit recall enabled")
            )
            store.close()

        self.assertEqual(result.output, [3, True])


if __name__ == "__main__":
    unittest.main()
