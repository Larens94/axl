import tempfile
import unittest
from pathlib import Path

from axl import Interpreter, RuntimeError, SQLiteMemoryStore, parse


class ScopedMemoryTest(unittest.TestCase):
    def test_scopes_isolate_same_memory_key(self):
        with tempfile.TemporaryDirectory() as directory:
            store = SQLiteMemoryStore(Path(directory) / "scoped.sqlite")
            Interpreter(memory_store=store, scope="user:1").run(
                parse('memory style = "short"')
            )
            Interpreter(memory_store=store, scope="user:2").run(
                parse('memory style = "detailed"')
            )

            one = Interpreter(memory_store=store, scope="user:1").run(
                parse("emit recall style")
            )
            two = Interpreter(memory_store=store, scope="user:2").run(
                parse("emit recall style")
            )
            store.close()

        self.assertEqual(one.output, ["short"])
        self.assertEqual(two.output, ["detailed"])

    def test_forget_removes_memory_in_current_scope(self):
        program = parse(
            'memory secret = "temporary"\nforget secret\nemit recall secret'
        )

        with self.assertRaisesRegex(RuntimeError, "unknown memory 'secret'"):
            Interpreter(scope="session:test").run(program)

    def test_metadata_is_recorded(self):
        store = SQLiteMemoryStore(":memory:")
        Interpreter(memory_store=store, scope="agent:neo").run(
            parse("memory confidence = 99 meta confidence=87 ttl=3600 source=user")
        )

        record = store.inspect("confidence", "agent:neo")
        store.close()

        self.assertEqual(record.scope, "agent:neo")
        self.assertEqual(record.value, 99)
        self.assertEqual(record.version, 1)
        self.assertEqual(record.confidence, 87)
        self.assertEqual(record.source, "user")
        self.assertIsNotNone(record.expires_at)
        self.assertIsNotNone(record.updated_at)

    def test_expired_memory_is_forgotten(self):
        store = SQLiteMemoryStore(":memory:")
        store.set("temporary", "gone", "session:x", ttl_seconds=-1)

        self.assertIsNone(store.get("temporary", "session:x"))
        self.assertIsNone(store.inspect("temporary", "session:x"))
        store.close()


if __name__ == "__main__":
    unittest.main()
