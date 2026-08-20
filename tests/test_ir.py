import json
import unittest

from axl import Interpreter, parse
from axl.serialization import IR_VERSION, program_from_json, program_to_json


class IRSerializationTest(unittest.TestCase):
    def test_ir_literal_rejects_isolated_unicode_surrogate(self):
        payload = (
            '{"ir_version":"1.1","program":{"type":"Program","instructions":'
            '[{"type":"Emit","value":{"type":"Literal","value":"\\ud800"}}]}}'
        )

        with self.assertRaisesRegex(ValueError, "invalid Unicode string"):
            program_from_json(payload)

    def test_ir_payload_rejects_raw_isolated_unicode_surrogate(self):
        with self.assertRaisesRegex(ValueError, "invalid Unicode payload"):
            program_from_json('"\ud800"')

    def test_program_round_trips_through_versioned_json(self):
        source = """
        let count = 0
        while count < 2
            if count == 0
                emit "first"
            else
                emit count
            end
            let count = count + 1
        end
        """
        encoded = program_to_json(parse(source))

        document = json.loads(encoded)
        restored = program_from_json(encoded)

        self.assertEqual(document["ir_version"], IR_VERSION)
        self.assertEqual(Interpreter().run(restored).output, ["first", 1])

    def test_function_round_trips_and_executes_from_ir(self):
        source = "fn add(a: int, b: int) -> int\n  return a + b\nend\nemit add(20, 22)"

        restored = program_from_json(program_to_json(parse(source)))

        self.assertEqual(Interpreter().run(restored).output, [42])

    def test_legacy_ir_1_0_let_is_upgraded(self):
        payload = json.dumps(
            {
                "ir_version": "1.0",
                "program": {
                    "type": "Program",
                    "instructions": [
                        {
                            "type": "Let",
                            "target": "answer",
                            "value": {"type": "Literal", "value": 42},
                        },
                        {
                            "type": "Emit",
                            "value": {"type": "Variable", "name": "answer"},
                        },
                    ],
                },
            }
        )

        restored = program_from_json(payload)

        self.assertEqual(Interpreter().run(restored).output, [42])

    def test_unknown_ir_version_is_rejected(self):
        payload = json.dumps(
            {"ir_version": "999", "program": {"type": "Program", "instructions": []}}
        )

        with self.assertRaisesRegex(ValueError, "unsupported IR version"):
            program_from_json(payload)

    def test_invalid_literal_type_is_rejected(self):
        payload = json.dumps(
            {
                "ir_version": IR_VERSION,
                "program": {
                    "type": "Program",
                    "instructions": [
                        {"type": "Emit", "value": {"type": "Literal", "value": 1.5}}
                    ],
                },
            }
        )

        with self.assertRaisesRegex(ValueError, "literal"):
            program_from_json(payload)

    def test_expression_cannot_appear_as_instruction(self):
        payload = json.dumps(
            {
                "ir_version": IR_VERSION,
                "program": {
                    "type": "Program",
                    "instructions": [{"type": "Literal", "value": 1}],
                },
            }
        )

        with self.assertRaisesRegex(ValueError, "instruction"):
            program_from_json(payload)

    def test_instruction_collection_must_be_array(self):
        payload = json.dumps(
            {
                "ir_version": IR_VERSION,
                "program": {"type": "Program", "instructions": "abc"},
            }
        )

        with self.assertRaisesRegex(ValueError, "instructions"):
            program_from_json(payload)

    def test_duplicate_json_keys_are_rejected(self):
        payload = '{"ir_version":"1.0","ir_version":"1.0","program":{"type":"Program","instructions":[]}}'

        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            program_from_json(payload)

    def test_extra_envelope_fields_are_rejected(self):
        payload = json.dumps(
            {
                "ir_version": IR_VERSION,
                "extra": True,
                "program": {"type": "Program", "instructions": []},
            }
        )

        with self.assertRaisesRegex(ValueError, "IR envelope fields"):
            program_from_json(payload)

    def test_non_string_agent_name_is_rejected(self):
        payload = json.dumps(
            {
                "ir_version": IR_VERSION,
                "program": {
                    "type": "Program",
                    "instructions": [
                        {"type": "Agent", "name": [], "tools": [], "body": []}
                    ],
                },
            }
        )

        with self.assertRaisesRegex(ValueError, "identifier"):
            program_from_json(payload)

    def test_serializer_rejects_invalid_program(self):
        from axl.ir import Literal, Program

        with self.assertRaises(ValueError):
            program_to_json(Program((Literal("not-an-instruction"),)))


if __name__ == "__main__":
    unittest.main()
