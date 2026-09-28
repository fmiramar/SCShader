from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from check_help_examples import runnable_blocks, sc_fixture


class HelpExampleTests(unittest.TestCase):
    def test_all_multiline_blocks_but_not_inline_code(self):
        text = "Inline code::foo::\ncode::\n(\n1;\n)\n::\ncode::\n(\n2;\n)\n::\n"
        self.assertEqual(len(runnable_blocks(text)), 2)

    def test_placeholder_and_document_relative_paths_are_rejected(self):
        for expression in ('"/path/to/renderer"', 'thisProcess.nowExecutingPath'):
            with self.assertRaises(ValueError):
                runnable_blocks(f"code::\n(\n{expression};\n)\n::\n")

    def test_parentheses_are_required(self):
        with self.assertRaises(ValueError):
            runnable_blocks("code::\n1;\n::\n")

    def test_observer_does_not_rewrite_example(self):
        import json
        code = '(\n"a path with spaces".postln;\n)'
        self.assertIn(json.dumps(code), sc_fixture(code, "(nil)", "server"))


if __name__ == "__main__":
    unittest.main()
