import json
import re
import unittest
from pathlib import Path

from axl import Interpreter, parse
from axl.serialization import IR_VERSION, program_from_json, program_to_json


class IRSerializationTest(unittest.TestCase):
    def test_ir_1_2_schema_type_pattern_is_canonical(self):
        schema = json.loads(
            (
                Path(__file__).parents[1] / "schema" / "axl-ir-1.2.schema.json"
            ).read_text()
        )
        pattern = schema["$defs"]["typeName"]["pattern"]

        self.assertIsNotNone(re.fullmatch(pattern, "list<list<int>>"))
        self.assertIsNone(re.fullmatch(pattern, "list<foo>"))
        self.assertIsNone(re.fullmatch(pattern, "list<int>>"))

    def test_typed_list_round_trips_through_ir_1_2(self):
        program = parse("2;10|xs|#1,#2,#3,~3|li;12|$xs")

        encoded = program_to_json(program)
        restored = program_from_json(encoded)

        self.assertEqual(json.loads(encoded)["ir_version"], "1.2")
        self.assertEqual(Interpreter().run(restored).output, [(1, 2, 3)])

    def test_legacy_ir_versions_reject_list_features(self):
        payloads = [
            {
                "type": "Program",
                "instructions": [
                    {
                        "type": "Emit",
                        "value": {"type": "ListExpression", "items": []},
                    }
                ],
            },
            {
                "type": "Program",
                "instructions": [
                    {
                        "type": "Let",
                        "target": "xs",
                        "value": {"type": "Literal", "value": 1},
                        "type_name": "list<int>",
                    }
                ],
            },
        ]
        for version in ("1.0", "1.1"):
            for payload in payloads:
                document = {"ir_version": version, "program": payload}

                with (
                    self.subTest(
                        version=version, node=payload["instructions"][0]["type"]
                    ),
                    self.assertRaisesRegex(ValueError, "requir(?:e|es) AX-IR 1.2"),
                ):
                    program_from_json(json.dumps(document))

    def test_excessively_nested_list_type_is_controlled_error(self):
        type_name = "list<" * 2_000 + "int" + ">" * 2_000
        document = {
            "ir_version": "1.2",
            "program": {
                "type": "Program",
                "instructions": [
                    {
                        "type": "Let",
                        "target": "xs",
                        "value": {"type": "ListExpression", "items": []},
                        "type_name": type_name,
                    }
                ],
            },
        }

        with self.assertRaisesRegex(ValueError, "type nesting is too deep"):
            program_from_json(json.dumps(document))

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

    def test_non_string_ir_version_is_controlled_error(self):
        for version in ([], {}):
            payload = json.dumps(
                {
                    "ir_version": version,
                    "program": {"type": "Program", "instructions": []},
                }
            )

            with (
                self.subTest(version=version),
                self.assertRaisesRegex(ValueError, "IR version must be a string"),
            ):
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
