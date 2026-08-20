import unittest

from axl import ApprovalRequired, Interpreter, RuntimeError, Tool, parse


class PolicyTest(unittest.TestCase):
    def test_mutating_tool_requires_explicit_approval(self):
        tool = Tool(
            "deploy", lambda target: f"deployed:{target}", effect="write", approval=True
        )
        program = parse('emit call deploy("prod")')

        with self.assertRaisesRegex(ApprovalRequired, "deploy"):
            Interpreter(tools=[tool]).run(program)

    def test_approved_tool_records_audit_event(self):
        tool = Tool(
            "deploy", lambda target: f"deployed:{target}", effect="write", approval=True
        )
        interpreter = Interpreter(
            tools=[tool], approve=lambda request: request.tool == "deploy"
        )

        result = interpreter.run(parse('emit call deploy("staging")'))

        self.assertEqual(result.output, ["deployed:staging"])
        self.assertEqual(result.audit[-1].decision, "executed")
        self.assertEqual(result.audit[-1].tool, "deploy")
        self.assertEqual(result.audit[-1].effect, "write")

    def test_denied_tool_never_executes(self):
        called = []
        tool = Tool(
            "erase", lambda: called.append(True), effect="destructive", approval=True
        )
        interpreter = Interpreter(tools=[tool], approve=lambda _request: False)

        with self.assertRaisesRegex(ApprovalRequired, "denied"):
            interpreter.run(parse("emit call erase()"))

        self.assertEqual(called, [])
        self.assertEqual(interpreter.audit[-1].decision, "denied")

    def test_non_boolean_approval_is_denied_fail_closed(self):
        called = []
        tool = Tool(
            "erase",
            lambda: called.append(True) or "ok",
            effect="destructive",
            approval=True,
        )
        interpreter = Interpreter(tools=[tool], approve=lambda _request: "false")

        with self.assertRaisesRegex(ApprovalRequired, "denied"):
            interpreter.run(parse("emit call erase()"))

        self.assertEqual(called, [])
        self.assertEqual(interpreter.audit[-1].decision, "denied")

    def test_approval_provider_failure_denies_safely(self):
        called = []
        tool = Tool(
            "write", lambda: called.append(True) or "ok", effect="write", approval=True
        )

        def broken(_request):
            raise OSError("provider unavailable")

        interpreter = Interpreter(tools=[tool], approve=broken)
        with self.assertRaisesRegex(ApprovalRequired, "approval provider failed"):
            interpreter.run(parse("emit call write()"))

        self.assertEqual(called, [])
        self.assertEqual(interpreter.audit[-1].decision, "denied")

    def test_duplicate_tool_names_are_rejected(self):
        tools = [Tool("same", lambda: "a"), Tool("same", lambda: "b")]

        with self.assertRaisesRegex(ValueError, "duplicate tool"):
            Interpreter(tools=tools)

    def test_plugin_runtime_error_is_audited_as_failed(self):
        def broken():
            raise RuntimeError("plugin failure")

        interpreter = Interpreter(tools=[Tool("broken", broken)])

        with self.assertRaisesRegex(RuntimeError, "tool 'broken' failed"):
            interpreter.run(parse("emit call broken()"))

        self.assertEqual(interpreter.audit[-1].decision, "failed")

    def test_invalid_tool_configuration_is_rejected(self):
        invalid = [
            Tool("if", lambda: "x"),
            Tool("valid", lambda: "x", approval="false"),
            Tool("valid", lambda: "x", effect=[]),
        ]

        for tool in invalid:
            with self.subTest(tool=tool), self.assertRaises(ValueError):
                Interpreter(tools=[tool])

    def test_policy_denials_are_audited(self):
        unknown = Interpreter()
        with self.assertRaisesRegex(RuntimeError, "not allowed"):
            unknown.run(parse("emit call missing()"))
        self.assertEqual(unknown.audit[-1].decision, "denied")

        ungranted = Interpreter(tools=[Tool("read", lambda: "ok")])
        with self.assertRaisesRegex(RuntimeError, "not granted"):
            ungranted.run(parse("agent worker\n  emit call read()\nend\nrun worker"))
        self.assertEqual(ungranted.audit[-1].decision, "denied")


if __name__ == "__main__":
    unittest.main()
