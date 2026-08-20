import unittest

from axl import Interpreter, RuntimeError, parse


class ProgramTest(unittest.TestCase):
    def test_program_remembers_recalls_and_prints(self):
        source = '''
        memory user_style = "short"
        let style = recall user_style
        emit style
        '''

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
        source = '''
        let score = 8
        if score >= 7
            emit score
        end
        if false
            emit score
        end
        '''

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [8])

    def test_memory_preserves_typed_values(self):
        result = Interpreter().run(parse("memory retries = 3\nlet n = recall retries\nemit n"))

        self.assertEqual(result.memory, {"retries": 3})
        self.assertEqual(result.output, [3])

    def test_else_executes_when_condition_is_false(self):
        source = '''
        let healthy = false
        if healthy
            emit "ready"
        else
            emit "blocked"
        end
        '''

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


if __name__ == "__main__":
    unittest.main()
