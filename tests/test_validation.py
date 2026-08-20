import unittest

from axl import ValidationError, parse, validate


class ValidationTest(unittest.TestCase):
    def test_legacy_literal_rejects_isolated_unicode_surrogate(self):
        program = parse(r'emit "\ud800"')

        with self.assertRaisesRegex(ValidationError, "invalid Unicode string"):
            validate(program)

    def test_duplicate_runnable_names_are_rejected(self):
        program = parse("agent worker\nend\nworkflow worker\nend")

        with self.assertRaisesRegex(ValidationError, "duplicate runnable 'worker'"):
            validate(program)

    def test_unresolved_nested_run_is_rejected(self):
        program = parse("workflow release\n  run missing\nend\nrun release")

        with self.assertRaisesRegex(ValidationError, "unknown runnable 'missing'"):
            validate(program)

    def test_declarations_inside_control_flow_are_rejected(self):
        program = parse("if true\n  agent hidden\n  end\nend")

        with self.assertRaisesRegex(ValidationError, "top-level"):
            validate(program)

    def test_recursive_workflows_are_rejected(self):
        program = parse(
            "workflow first\n  run second\nend\nworkflow second\n  run first\nend\nrun first"
        )

        with self.assertRaisesRegex(ValidationError, "cycle"):
            validate(program)

    def test_reserved_keyword_cannot_be_identifier(self):
        with self.assertRaisesRegex(ValueError, "reserved"):
            parse("let true = 7")

    def test_reserved_keyword_cannot_be_tool_name(self):
        with self.assertRaisesRegex(ValueError, "reserved"):
            parse("emit call agent()")

    def test_duplicate_agent_grants_are_rejected(self):
        with self.assertRaisesRegex(ValidationError, "duplicate tool grant"):
            validate(parse("agent worker uses read,read\nend"))

    def test_duplicate_memory_metadata_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate memory metadata"):
            parse("memory x = 1 meta confidence=10 confidence=20")

    def test_deep_acyclic_workflow_graph_validates_without_recursion(self):
        count = 1500
        declarations = "\n".join(
            f"workflow w{i}\n  run w{i + 1}\nend" for i in range(count - 1)
        )
        program = parse(declarations + f"\nworkflow w{count - 1}\nend\nrun w0")

        with self.assertRaisesRegex(ValidationError, "call depth"):
            validate(program)

    def test_excessive_block_nesting_is_rejected(self):
        depth = 300
        program = parse(("if true\n" * depth) + 'emit "ok"\n' + ("end\n" * depth))

        with self.assertRaisesRegex(ValidationError, "nesting depth"):
            validate(program)


if __name__ == "__main__":
    unittest.main()
