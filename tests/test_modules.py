import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from axl import Interpreter, TypeCheckError, typecheck
from axl.compiler import CompileError, compile_file


class ModuleTest(unittest.TestCase):
    def test_single_module_size_is_checked_before_read(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_text("2;12|#1")

            with (
                patch("axl.compiler._MAX_TOTAL_SOURCE_BYTES", 4),
                patch.object(
                    Path, "read_text", side_effect=AssertionError("read_text called")
                ),
                self.assertRaisesRegex(CompileError, "aggregate module source"),
            ):
                compile_file(app)

    def test_legacy_import_is_rejected_inside_function(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "m.axl").write_text("fn x() -> int\n  return 1\nend\n")
            app = root / "app.axl"
            app.write_text('fn f() -> int\n  import m from "m.axl"\n  return 1\nend\n')

            with self.assertRaisesRegex(CompileError, "top-level"):
                compile_file(app)

    def test_invalid_utf8_module_is_a_compile_error(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_bytes(b"\xff\xfe")

            with self.assertRaisesRegex(CompileError, "cannot read module"):
                compile_file(app)

    def test_import_graph_has_a_module_count_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "a.axl").write_text('import b from "b.axl"\n')
            (root / "b.axl").write_text("")

            with (
                patch("axl.compiler._MAX_MODULES", 1),
                self.assertRaisesRegex(CompileError, "module count"),
            ):
                compile_file(root / "a.axl")

    def test_import_graph_has_an_aggregate_source_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "a.axl").write_text('import b from "b.axl"\n')
            (root / "b.axl").write_text("# padding\n")

            with (
                patch("axl.compiler._MAX_TOTAL_SOURCE_BYTES", 30),
                self.assertRaisesRegex(CompileError, "aggregate module source"),
            ):
                compile_file(root / "a.axl")

    def test_import_cannot_escape_module_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            project = root / "project"
            project.mkdir()
            (root / "outside.axl").write_text("fn x() -> int\n  return 1\nend\n")
            app = project / "app.axl"
            app.write_text('import outside from "../outside.axl"\n')

            with self.assertRaisesRegex(CompileError, "module root"):
                compile_file(app)

    def test_absolute_import_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            module = root / "module.axl"
            module.write_text("fn x() -> int\n  return 1\nend\n")
            app = root / "app.axl"
            app.write_text(f'import module from "{module}"\n')

            with self.assertRaisesRegex(CompileError, "relative"):
                compile_file(app)

    def test_import_depth_has_a_controlled_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for index in range(258):
                source = f'import next from "m{index + 1}.axl"\n' if index < 257 else ""
                (root / f"m{index}.axl").write_text(source)

            with self.assertRaisesRegex(CompileError, "import depth"):
                compile_file(root / "m0.axl")

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
