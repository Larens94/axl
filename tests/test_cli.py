import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class CliTest(unittest.TestCase):
    def test_run_file_prints_program_output(self):
        with tempfile.TemporaryDirectory() as directory:
            program = Path(directory) / "hello.axl"
            program.write_text(
                'memory greeting = "hello agent"\nlet x = recall greeting\nemit x\n'
            )

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
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )
            second = subprocess.run(
                [*command, "--memory", str(database), str(recall)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(second.stdout, "AXL\n")

    def test_compile_then_execute_ir(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "program.axl"
            target = root / "program.axlir.json"
            source.write_text('emit "compiled"\n')
            base = [sys.executable, "-m", "axl"]

            compiled = subprocess.run(
                [*base, "compile", str(source), "-o", str(target)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )
            executed = subprocess.run(
                [*base, "exec", str(target)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(compiled.returncode, 0, compiled.stderr)
        self.assertEqual(executed.returncode, 0, executed.stderr)
        self.assertEqual(executed.stdout, "compiled\n")

    def test_cli_compiles_and_runs_imported_module(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            module = root / "math.axl"
            app = root / "app.axl"
            target = root / "app.json"
            module.write_text(
                "fn add(a: int, b: int) -> int\n  return a + b\nend\n",
                encoding="utf-8",
            )
            app.write_text(
                'import math from "math.axl"\nemit math.add(2, 5)\n',
                encoding="utf-8",
            )
            base = [sys.executable, "-m", "axl"]

            compiled = subprocess.run(
                [*base, "compile", str(app), "-o", str(target)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )
            executed = subprocess.run(
                [*base, "exec", str(target)],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(compiled.returncode, 0, compiled.stderr)
        self.assertEqual(executed.returncode, 0, executed.stderr)
        self.assertEqual(executed.stdout, "7\n")

    def test_cli_loads_explicit_tool_plugin_and_approval(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            plugin = root / "demo_plugin.py"
            program = root / "agent.axl"
            plugin.write_text(
                "from axl import Tool\n"
                "def tools():\n"
                "    return [Tool('deploy', lambda target: 'ok:' + target, effect='write', approval=True)]\n"
            )
            program.write_text(
                'agent releaser uses deploy\n  emit call deploy("staging")\nend\nrun releaser\n'
            )
            environment = dict(__import__("os").environ)
            environment["PYTHONPATH"] = f"{root}:{Path(__file__).parents[1]}"

            completed = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "axl",
                    "run",
                    "--plugin",
                    "demo_plugin",
                    "--approve-tool",
                    "deploy",
                    str(program),
                ],
                cwd=Path(__file__).parents[1],
                capture_output=True,
                text=True,
                env=environment,
                check=False,
            )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stdout, "ok:staging\n")


if __name__ == "__main__":
    unittest.main()
