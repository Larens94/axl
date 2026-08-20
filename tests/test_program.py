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


if __name__ == "__main__":
    unittest.main()
