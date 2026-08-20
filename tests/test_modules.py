import tempfile
import unittest
from pathlib import Path

from axl import Interpreter, TypeCheckError, typecheck
from axl.compiler import CompileError, compile_file


class ModuleTest(unittest.TestCase):
    def test_imported_function_uses_namespace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "math.axl").write_text(
                "fn add(a: int, b: int) -> int\n  return a + b\nend\n",
                encoding="utf-8",
            )
            app = root / "app.axl"
            app.write_text(
                'import math from "math.axl"\nemit math.add(20, 22)\n',
                encoding="utf-8",
            )

            program = compile_file(app)
            typecheck(program)

            self.assertEqual(Interpreter().run(program).output, [42])

    def test_module_local_function_calls_are_namespaced(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "numbers.axl").write_text(
                "fn double(value: int) -> int\n  return value * 2\nend\n"
                "fn quadruple(value: int) -> int\n  return double(double(value))\nend\n",
                encoding="utf-8",
            )
            app = root / "app.axl"
            app.write_text(
                'import numbers from "numbers.axl"\nemit numbers.quadruple(3)\n',
                encoding="utf-8",
            )

            result = Interpreter().run(compile_file(app))

            self.assertEqual(result.output, [12])

    def test_duplicate_import_alias_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "one.axl").write_text("", encoding="utf-8")
            app = root / "app.axl"
            app.write_text(
                'import same from "one.axl"\nimport same from "one.axl"\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(CompileError, "duplicate import alias 'same'"):
                compile_file(app)

    def test_cyclic_import_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = root / "first.axl"
            second = root / "second.axl"
            first.write_text('import second from "second.axl"\n', encoding="utf-8")
            second.write_text('import first from "first.axl"\n', encoding="utf-8")

            with self.assertRaisesRegex(CompileError, "cyclic module import"):
                compile_file(first)

    def test_unknown_namespace_is_static_error(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_text("emit missing.value()\n", encoding="utf-8")

            with self.assertRaisesRegex(
                TypeCheckError, "unknown function 'missing.value'"
            ):
                typecheck(compile_file(app))


if __name__ == "__main__":
    unittest.main()
