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


if __name__ == "__main__":
    unittest.main()
