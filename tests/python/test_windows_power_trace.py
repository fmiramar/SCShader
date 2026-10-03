import ctypes
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from windows_power_trace import DISPLAY_GUID, PowerSetting, WindowsPowerTrace, display_state


class PowerTraceTests(unittest.TestCase):
    def test_validates_the_native_notification_before_reading_its_state(self):
        for value, name in enumerate(("off", "on", "dim")):
            setting = PowerSetting((ctypes.c_ubyte * 16).from_buffer_copy(DISPLAY_GUID), 4, value)
            self.assertEqual(display_state(setting), name)
        for length, value in [(0, 1), (8, 1), (4, 3)]:
            setting = PowerSetting((ctypes.c_ubyte * 16).from_buffer_copy(DISPLAY_GUID), length, value)
            with self.assertRaises(ValueError):
                display_state(setting)
        with self.assertRaises(ValueError):
            display_state(PowerSetting((ctypes.c_ubyte * 16)(), 4, 1))

    @unittest.skipUnless(os.name == "nt", "Windows native notification")
    def test_native_initial_state_and_registration_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "power.jsonl"
            with WindowsPowerTrace(path) as trace:
                self.assertTrue(trace.handle.value)
            self.assertIsNone(trace.handle.value)
            events = [json.loads(line) for line in path.read_text().splitlines()]
            self.assertGreaterEqual(len(events), 1)
            self.assertTrue(all(event["state"] in ("off", "on", "dim") for event in events))


if __name__ == "__main__":
    unittest.main()
