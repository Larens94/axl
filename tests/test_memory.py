import tempfile
import unittest
from pathlib import Path

from axl import Interpreter, SQLiteMemoryStore, parse


class PersistentMemoryTest(unittest.TestCase):
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
