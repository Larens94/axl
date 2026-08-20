import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class CliTest(unittest.TestCase):
    def test_run_file_prints_program_output(self):
        with tempfile.TemporaryDirectory() as directory:
            program = Path(directory) / "hello.axl"
            program.write_text('memory greeting = "hello agent"\nlet x = recall greeting\nemit x\n')

            completed = subprocess.run(
                [sys.executable, "-m", "axl", "run", str(program)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stdout, "hello agent\n")

    def test_cli_persists_memory_between_runs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            database = root / "memory.sqlite"
            remember = root / "remember.axl"
            recall = root / "recall.axl"
            remember.write_text('memory name = "AXL"\n')
            recall.write_text("emit recall name\n")
            command = [sys.executable, "-m", "axl", "run"]

            first = subprocess.run(
                [*command, "--memory", str(database), str(remember)],
                cwd=Path(__file__).parents[1], capture_output=True, text=True,
            )
            second = subprocess.run(
                [*command, "--memory", str(database), str(recall)],
                cwd=Path(__file__).parents[1], capture_output=True, text=True,
            )

        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(second.stdout, "AXL\n")


if __name__ == "__main__":
    unittest.main()
