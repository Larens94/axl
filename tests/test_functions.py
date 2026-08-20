import unittest

from axl import Interpreter, RuntimeError, TypeCheckError, parse, typecheck


class FunctionTest(unittest.TestCase):
    def test_typed_function_executes_end_to_end(self):
        source = """
        fn add(a: int, b: int) -> int
            return a + b
        end
        let total: int = add(2, 3)
        emit total
        """

        program = parse(source)
        typecheck(program)
        result = Interpreter().run(program)

        self.assertEqual(result.output, [5])

    def test_function_argument_type_is_checked_before_execution(self):
        program = parse(
            'fn double(value: int) -> int\n  return value * 2\nend\nemit double("x")'
        )

        with self.assertRaisesRegex(
            TypeCheckError, "argument 1 of 'double' must be int, got string"
        ):
            typecheck(program)

    def test_function_return_type_is_checked(self):
        program = parse('fn broken() -> int\n  return "wrong"\nend\nemit broken()')

        with self.assertRaisesRegex(
            TypeCheckError, "function 'broken' must return int, got string"
        ):
            typecheck(program)

    def test_function_parameters_and_locals_do_not_leak(self):
        program = parse(
            "fn identity(value: int) -> int\n"
            "  let local: int = value\n"
            "  return local\n"
            "end\n"
            "emit identity(7)\n"
            "emit value"
        )

        with self.assertRaisesRegex(TypeCheckError, "unknown variable 'value'"):
            typecheck(program)

    def test_missing_return_is_rejected(self):
        program = parse("fn missing() -> int\n  let x: int = 1\nend\nemit missing()")

        with self.assertRaisesRegex(TypeCheckError, "may complete without returning"):
            typecheck(program)

    def test_recursive_function_is_stopped_by_call_depth(self):
        program = parse(
            "fn recurse(value: int) -> int\n"
            "  return recurse(value + 1)\n"
            "end\n"
            "emit recurse(0)"
        )
        typecheck(program)

        with self.assertRaisesRegex(RuntimeError, "function call depth exceeded"):
            Interpreter().run(program)

    def test_duplicate_function_names_are_rejected(self):
        program = parse(
            "fn same() -> int\n  return 1\nend\nfn same() -> int\n  return 2\nend"
        )

        with self.assertRaisesRegex(TypeCheckError, "duplicate function 'same'"):
            typecheck(program)


if __name__ == "__main__":
    unittest.main()
