from pathlib import Path
import json
import sys
from unittest import TestCase

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from run_sc_checks import check_passed, isolated_script


class SCCheckTests(TestCase):
    def test_success_marker_cannot_hide_shutdown_or_server_errors(self):
        marker = "SCSHADER_TEST_OK\n"
        self.assertTrue(check_passed("compile_check", marker, 0))
        for error in ("ERROR: Could not open archive for writing", "FAILURE IN SERVER /s_new"):
            self.assertFalse(check_passed("compile_check", marker + error, 0))
        self.assertFalse(check_passed("compile_check", marker, 1))
        self.assertFalse(check_passed("compile_check", marker, 0, timed_out=True))
        self.assertFalse(check_passed("verify_scdoc", marker + "WARNING: bad document", 0))

    def test_wrapper_loads_original_path_and_isolates_generated_state(self):
        source = Path("source with spaces/caf\u00e9/test.scd")
        state = Path("test state")
        wrapper = isolated_script(source, state)
        self.assertIn(json.dumps(source.resolve().as_posix(), ensure_ascii=False) + ".load;", wrapper)
        self.assertIn("Archive.global = Archive.new;", wrapper)
        self.assertIn("Archive.archiveDir = ", wrapper)
        self.assertIn("SCDoc.helpTargetDir = ", wrapper)
        self.assertIn("SynthDef.synthDefDir = ", wrapper)

    def test_audio_overrides_are_opt_in_and_before_loading_test(self):
        source, state = Path("test.scd"), Path("state")
        self.assertNotIn("ServerOptions", isolated_script(source, state))
        wrapper = isolated_script(source, state, 'Output "test"', 0)
        self.assertIn('ServerOptions.defaultValues[\\outDevice] = "Output \\"test\\"";', wrapper)
        self.assertIn("ServerOptions.defaultValues[\\numInputBusChannels] = 0;", wrapper)
        self.assertLess(wrapper.index("ServerOptions"), wrapper.index(".load;"))
        with self.assertRaises(ValueError):
            isolated_script(source, state, audio_input_channels=-1)
