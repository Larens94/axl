import tempfile
import unittest
from pathlib import Path

from axl import Interpreter, ParseError, RuntimeError, TypeCheckError, parse, typecheck
from axl.compact import CompactParseError, program_to_compact
from axl.compiler import CompileError, compile_file
from axl.ir import Emit, Let, ListExpression, Literal, MapValue, Program, ToolCall


class CompactSyntaxTest(unittest.TestCase):
    def test_compact_typed_map_executes_end_to_end(self):
        source = '2;10|scores|"alice",#7,"bob",#9,%2|msi;12|$scores'

        program = parse(source)
        typecheck(program)
        compact = program_to_compact(program)
        result = Interpreter().run(program)

        self.assertEqual(compact, source)
        self.assertEqual(result.output, [MapValue((("alice", 7), ("bob", 9)))])

    def test_compact_map_supports_nested_list_values(self):
        source = '2;10|groups|"a",#1,#2,~2,%1|msli;12|$groups'

        program = parse(source)
        typecheck(program)

        self.assertEqual(program_to_compact(program), source)
        self.assertEqual(
            Interpreter().run(program).output, [MapValue((("a", (1, 2)),))]
        )

    def test_compact_map_rejects_mixed_value_types(self):
        program = parse('2;10|values|"a",#1,"b","two",%2|msi')

        with self.assertRaisesRegex(TypeCheckError, "map values must have one type"):
            typecheck(program)

    def test_compact_map_rejects_duplicate_keys(self):
        program = parse('2;12|"a",#1,"a",#2,%2')

        with self.assertRaisesRegex(RuntimeError, "map keys must be unique"):
            Interpreter().run(program)

    def test_compact_map_type_depth_is_bounded(self):
        source = f"2;10|value|#1|{'l' * 17}i"

        with self.assertRaisesRegex(ParseError, "type nesting is too deep"):
            parse(source)

    def test_compact_empty_map_adopts_declared_types(self):
        program = parse("2;10|values|%0|msi;12|$values")

        typecheck(program)

        self.assertEqual(Interpreter().run(program).output, [MapValue(())])

    def test_empty_map_unifies_recursively_inside_list(self):
        program = parse('2;10|values|%0,"x",#1,%1,~2|lmsi;12|$values')

        typecheck(program)

        self.assertEqual(
            Interpreter().run(program).output,
            [(MapValue(()), MapValue((("x", 1),)))],
        )

    def test_compact_map_rejects_collection_keys_statically(self):
        program = parse("2;10|values|#1,~1,#2,%1|mlii")

        with self.assertRaisesRegex(TypeCheckError, "map keys must be scalar"):
            typecheck(program)

    def test_unknown_nested_list_type_is_rejected_statically(self):
        program = Program((Let("xs", ListExpression((Literal(1),)), "list<unknown>"),))

        with self.assertRaisesRegex(TypeCheckError, "unknown type 'list<unknown>'"):
            typecheck(program)

    def test_inferred_list_type_cannot_exceed_depth_limit(self):
        expression = Literal(1)
        for _ in range(17):
            expression = ListExpression((expression,))
        program = Program((Let("xs", expression),))

        with self.assertRaisesRegex(TypeCheckError, "type nesting is too deep"):
            typecheck(program)

    def test_compact_typed_list_executes_end_to_end(self):
        program = parse("2;10|xs|#1,#2,#3,~3|li;12|$xs")

        typecheck(program)
        compact = program_to_compact(program)
        result = Interpreter().run(parse(compact))

        self.assertEqual(compact, "2;10|xs|#1,#2,#3,~3|li;12|$xs")
        self.assertEqual(result.output, [(1, 2, 3)])

    def test_compact_nested_list_round_trips(self):
        source = "2;10|matrix|#1,#2,~2,#3,#4,~2,~2|lli;12|$matrix"

        program = parse(source)
        typecheck(program)

        self.assertEqual(program_to_compact(program), source)
        self.assertEqual(Interpreter().run(program).output, [((1, 2), (3, 4))])

    def test_compact_function_accepts_and_returns_typed_list(self):
        source = "2;40|identity|xs:li|li;11|$xs;99;12|#1,#2,~2,^identity/1"

        program = parse(source)
        typecheck(program)
        result = Interpreter().run(program)

        self.assertEqual(result.output, [(1, 2)])

    def test_tool_capability_can_return_a_bounded_list(self):
        program = parse("2;12|!items/0")
        interpreter = Interpreter(tools={"items": lambda: (1, 2, 3)})

        result = interpreter.run(program)

        self.assertEqual(result.output, [(1, 2, 3)])

    def test_tool_capability_list_depth_is_bounded(self):
        value = 0
        for _ in range(257):
            value = (value,)
        interpreter = Interpreter(tools={"deep": lambda: value})

        with self.assertRaisesRegex(RuntimeError, "value nesting exceeds 256"):
            interpreter.run(parse("2;12|!deep/0"))

    def test_compact_empty_list_adopts_declared_item_type(self):
        program = parse("2;10|xs|~0|li;12|$xs")

        typecheck(program)
        result = Interpreter().run(program)

        self.assertEqual(result.output, [()])

    def test_nested_empty_list_adopts_declared_item_type(self):
        program = parse("2;10|xs|~0,#1,~1,~2|lli;12|$xs")

        typecheck(program)
        result = Interpreter().run(program)

        self.assertEqual(result.output, [((), (1,))])

    def test_compact_typed_list_rejects_mixed_item_types(self):
        program = parse('2;10|xs|#1,"two",~2|li')

        with self.assertRaisesRegex(ValueError, "list items must have one type"):
            typecheck(program)

    def test_compact_writer_rejects_call_above_parser_arity_limit(self):
        arguments = tuple(Literal(1) for _ in range(65_536))
        program = Program((Emit(ToolCall("tool", arguments)),))

        with self.assertRaisesRegex(CompactParseError, "invalid call arity"):
            program_to_compact(program)

    def test_compact_ignores_all_whitespace_outside_strings(self):
        source = "2;40|f| a : i , b : i | i;11|$ a,$ b,+;99;12|#1,#2,^ f / 2"

        normalized = program_to_compact(parse(source))
        result = Interpreter().run(parse(normalized))

        self.assertEqual(normalized, "2;40|f|a:i,b:i|i;11|$a,$b,+;99;12|#1,#2,^f/2")
        self.assertEqual(result.output, [3])

    def test_compact_boolean_whitespace_is_non_structural(self):
        self.assertEqual(Interpreter().run(parse("2;12|? 1")).output, [True])

    def test_compact_rejects_isolated_unicode_surrogate(self):
        with self.assertRaisesRegex(ParseError, "invalid Unicode string"):
            parse(r'2;12|"\ud800"')

        with self.assertRaisesRegex(ParseError, "invalid Unicode source"):
            parse('2;12|"\ud800"')

    def test_compact_writer_rejects_isolated_unicode_surrogate(self):
        program = Program((Emit(Literal("\ud800")),))

        with self.assertRaisesRegex(CompactParseError, "invalid Unicode string"):
            program_to_compact(program)

    def test_compact_writer_never_emits_source_above_parser_limit(self):
        program = Program((Emit(Literal("x" * 1_000_000)),))

        with self.assertRaisesRegex(CompactParseError, "source exceeds"):
            program_to_compact(program)

    def test_compact_writer_rejects_unencodable_type_without_key_error(self):
        program = Program((Let("x", Literal(1), "unknown"),))

        with self.assertRaisesRegex(CompactParseError, "cannot encode type"):
            program_to_compact(program)

    def test_compact_string_round_trip_preserves_all_delimiters(self):
        source = r'2;12|" a;|,\\\"b "'

        normalized = program_to_compact(parse(source))
        result = Interpreter().run(parse(normalized))

        self.assertEqual(result.output, [' a;|,\\"b '])

    def test_compact_ignores_whitespace_outside_strings(self):
        source = " 2 ;\n 10 | x | #2 , #3 , + | i ;\n12 | $x "

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [5])

    def test_compact_unterminated_string_is_compile_error(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_text('2;12|"unterminated')

            with self.assertRaisesRegex(CompileError, "unterminated string"):
                compile_file(app)

    def test_compact_huge_call_arity_is_controlled_parse_error(self):
        source = "2;12|!tool/" + "9" * 5_000

        with self.assertRaisesRegex(ParseError, "invalid call arity"):
            parse(source)

    def test_compact_import_is_rejected_inside_a_block(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "m.axl").write_text("2;40|x||i;11|#1;99")
            app = root / "app.axl"
            app.write_text("2;40|f||i;1|m|m.axl;11|#1;99")

            with self.assertRaisesRegex(CompileError, "top-level"):
                compile_file(app)

    def test_compact_excessive_nesting_is_a_controlled_parse_error(self):
        source = "2;" + ";".join(["30|?1"] * 1_500 + ["99"] * 1_500)

        with self.assertRaisesRegex(ParseError, "nesting is too deep"):
            parse(source)

    def test_compact_memory_keeps_metadata_and_recall(self):
        source = '2;20|k|"v"|95|60|bot;12|@k;21|k'

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, ["v"])
        self.assertEqual(result.memory, {})

    def test_compact_if_and_while_use_end_opcode_not_indentation(self):
        source = '2;10|n|#0;32|$n,#3,<;30|$n,#1,=;12|"one";31;12|$n;99;10|n|$n,#1,+;99'

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [0, "one", 2])

    def test_compact_function_uses_typed_signature_and_postfix_call(self):
        source = "2;40|add|a:i,b:i|i;11|$a,$b,+;99;10|n|#7,#8,^add/2|i;12|$n"

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [15])

    def test_compact_agents_workflows_and_tool_calls_execute(self):
        source = '2;50|r|join;10|x|"AX","L",!join/2;12|$x;99;51|w;52|r;99;52|w'
        interpreter = Interpreter(tools={"join": lambda left, right: left + right})

        result = interpreter.run(parse(source))

        self.assertEqual(result.output, ["AXL"])

    def test_compact_import_uses_opcode_and_namespace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "m.axl").write_text("2;40|add|a:i,b:i|i;11|$a,$b,+;99")
            app = root / "app.axl"
            app.write_text("2;1|m|m.axl;12|#20,#22,^m.add/2")

            result = Interpreter().run(compile_file(app))

        self.assertEqual(result.output, [42])

    def test_compact_import_allows_non_structural_whitespace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "m.axl").write_text("2;40|x||i;11|#1;99")
            app = root / "app.axl"
            app.write_text(" 2 ; 1 | m | m.axl ; 12 | ^m.x/0 ")

            result = Interpreter().run(compile_file(app))

        self.assertEqual(result.output, [1])

    def test_legacy_program_normalizes_to_canonical_compact_source(self):
        legacy = "let total: int = 2 + 3 * 4\nemit total"

        compact = program_to_compact(parse(legacy))
        result = Interpreter().run(parse(compact))

        self.assertEqual(compact, "2;10|total|#2,#3,#4,*,+|i;12|$total")
        self.assertEqual(result.output, [14])

    def test_compact_file_preserves_semicolon_inside_string(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_text('2;12|"a;b"')

            result = Interpreter().run(compile_file(app))

        self.assertEqual(result.output, ["a;b"])

    def test_compact_file_never_treats_string_content_as_import_frame(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / "app.axl"
            app.write_text('2;12|"a;1|x|missing.axl;b"')

            result = Interpreter().run(compile_file(app))

        self.assertEqual(result.output, ["a;1|x|missing.axl;b"])

    def test_single_line_opcode_stream_executes_rpn_expression(self):
        source = "2;10|x|#2,#3,#4,*,+;12|$x"

        result = Interpreter().run(parse(source))

        self.assertEqual(result.output, [14])


if __name__ == "__main__":
    unittest.main()
