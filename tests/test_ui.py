import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from axl import ValidationError, parse, program_to_compact, typecheck, validate
from axl.ir import Annotation, UiView
from axl.serialization import program_to_json


ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples" / "streaming_home.axl"


class AxUiTest(unittest.TestCase):
    def test_source_3_ui_round_trips_through_canonical_writer(self):
        source = EXAMPLE.read_text(encoding="utf-8").strip()
        program = parse(source)

        validate(program)
        typecheck(program)

        self.assertEqual(parse(program_to_compact(program)), program)
        self.assertEqual(sum(isinstance(item, Annotation) for item in program.instructions), 3)
        self.assertEqual(sum(isinstance(item, UiView) for item in program.instructions), 1)

    def test_registry_rejects_property_not_supported_by_component(self):
        program = parse('3;60|1;61|1|1;62|99|"invalid";99')

        with self.assertRaisesRegex(ValidationError, "property '99'"):
            validate(program)

    def test_ui_is_not_mislabeled_as_published_ir_1_2(self):
        with self.assertRaisesRegex(ValueError, "not available in AX-IR 1.2"):
            program_to_json(parse('3;60|1;61|1|1;62|1|"App";99'))

    def test_cli_build_generates_complete_web_artifact_from_axl(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "site"
            completed = subprocess.run(
                [sys.executable, "-m", "axl", "build", str(EXAMPLE), "--target", "web", "-o", str(output)],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            page = output.joinpath("index.html").read_text(encoding="utf-8") if output.exists() else ""

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("NEON CITY", page)
        self.assertIn("ax-ui.css", page)
        self.assertNotIn("React", page)


if __name__ == "__main__":
    unittest.main()
