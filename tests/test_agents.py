import unittest

from axl import Interpreter, RuntimeError, Tool, ValidationError, parse


class AgentWorkflowTest(unittest.TestCase):
    def test_workflow_runs_agents_in_order(self):
        source = """
        agent researcher uses search
            let finding = call search("AXL")
            emit finding
        end

        agent writer
            emit "report-ready"
        end

        workflow release
            run researcher
            run writer
        end

        run release
        """
        search = Tool("search", lambda query: f"found:{query}")

        result = Interpreter(tools=[search]).run(parse(source))

        self.assertEqual(result.output, ["found:AXL", "report-ready"])

    def test_agent_cannot_call_undeclared_tool(self):
        source = """
        agent unsafe
            emit call shell("pwd")
        end
        run unsafe
        """
        shell = Tool("shell", lambda command: command)

        with self.assertRaisesRegex(RuntimeError, "not granted to agent 'unsafe'"):
            Interpreter(tools=[shell]).run(parse(source))

    def test_unknown_runnable_fails_explicitly(self):
        with self.assertRaisesRegex(ValidationError, "unknown runnable 'missing'"):
            Interpreter().run(parse("run missing"))

    def test_agent_local_bindings_do_not_leak(self):
        source = """
        agent worker
            let private = "secret"
        end
        run worker
        emit private
        """

        with self.assertRaisesRegex(RuntimeError, "unknown variable 'private'"):
            Interpreter().run(parse(source))


if __name__ == "__main__":
    unittest.main()
