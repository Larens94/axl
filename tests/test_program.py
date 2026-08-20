import unittest

from axl import Interpreter, RuntimeError, parse


class ProgramTest(unittest.TestCase):
    def test_program_remembers_recalls_and_prints(self):
        source = """
        memory user_style = "short"
        let style = recall user_style
        emit style
        """

        program = parse(source)
        result = Interpreter().run(program)

        self.assertEqual(result.output, ["short"])
        self.assertEqual(result.memory, {"user_style": "short"})

    def test_recall_of_missing_memory_reports_the_key(self):
        program = parse("let style = recall missing")

        with self.assertRaisesRegex(RuntimeError, "unknown memory 'missing'"):
            Interpreter().run(program)

    def test_typed_expressions_respect_precedence(self):
        program = parse("let total = 2 + 3 * 4\nemit total")

        result = Interpreter().run(program)

        self.assertEqual(result.output, [14])

    def test_condition_executes_only_when_true(self):
        source = """
        let score = 8
        if score >= 7
            emit score
        end
        if false
            emit score
        end
        """

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [8])

    def test_memory_preserves_typed_values(self):
        result = Interpreter().run(
            parse("memory retries = 3\nlet n = recall retries\nemit n")
        )

        self.assertEqual(result.memory, {"retries": 3})
        self.assertEqual(result.output, [3])

    def test_memory_string_may_contain_meta_keyword(self):
        result = Interpreter().run(
            parse('memory text = "contains meta marker"\nemit recall text')
        )

        self.assertEqual(result.output, ["contains meta marker"])

    def test_else_executes_when_condition_is_false(self):
        source = """
        let healthy = false
        if healthy
            emit "ready"
        else
            emit "blocked"
        end
        """

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, ["blocked"])

    def test_registered_tool_can_be_called_as_expression(self):
        program = parse('let result = call combine("AX", "L")\nemit result')
        interpreter = Interpreter(tools={"combine": lambda left, right: left + right})

        result = interpreter.run(program)

        self.assertEqual(result.output, ["AXL"])

    def test_unregistered_tool_is_denied(self):
        program = parse("let result = call shell()")

        with self.assertRaisesRegex(RuntimeError, "tool 'shell' is not allowed"):
            Interpreter().run(program)

    def test_while_repeats_until_condition_is_false(self):
        source = """
        let count = 0
        while count < 3
            emit count
            let count = count + 1
        end
        """

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [0, 1, 2])

    def test_execution_budget_stops_infinite_loop(self):
        program = parse("while true\n  emit 1\nend")

        with self.assertRaisesRegex(RuntimeError, "execution budget exceeded"):
            Interpreter(max_steps=5).run(program)

    def test_fractional_division_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "non-integer division"):
            Interpreter().run(parse("emit 1 / 2"))

    def test_boolean_is_not_an_integer_operand(self):
        with self.assertRaisesRegex(RuntimeError, "invalid operands"):
            Interpreter().run(parse("emit true + 1"))

    def test_tool_result_must_be_an_axl_value(self):
        interpreter = Interpreter(tools={"bad": lambda: None})

        with self.assertRaisesRegex(RuntimeError, "invalid value"):
            interpreter.run(parse("emit call bad()"))

    def test_output_budget_is_enforced(self):
        with self.assertRaisesRegex(RuntimeError, "output budget exceeded"):
            Interpreter(max_output_bytes=3).run(parse('emit "four"'))

    def test_tool_call_budget_is_enforced(self):
        interpreter = Interpreter(tools={"ping": lambda: "ok"}, max_tool_calls=1)

        with self.assertRaisesRegex(RuntimeError, "tool call budget exceeded"):
            interpreter.run(parse("emit call ping()\nemit call ping()"))

    def test_intermediate_value_budget_is_enforced(self):
        interpreter = Interpreter(max_value_bytes=4)

        with self.assertRaisesRegex(RuntimeError, "value budget exceeded"):
            interpreter.run(parse('emit "abc" + "def"'))

    def test_invalid_memory_value_is_rejected(self):
        class InvalidStore:
            def get(self, key, scope):
                return []

            def set(self, *args, **kwargs):
                pass

            def delete(self, *args, **kwargs):
                return False

            def snapshot(self, scope):
                return {}

        with self.assertRaisesRegex(RuntimeError, "memory 'x' contains invalid value"):
            Interpreter(memory_store=InvalidStore()).run(parse("emit recall x"))

    def test_huge_integer_output_fails_with_axl_error(self):
        interpreter = Interpreter(tools={"big": lambda: 1 << 20_000})

        with self.assertRaisesRegex(RuntimeError, "integer output is too large"):
            interpreter.run(parse("emit call big()"))


if __name__ == "__main__":
    unittest.main()
