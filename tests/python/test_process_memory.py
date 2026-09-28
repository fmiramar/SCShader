import os
from pathlib import Path
import subprocess
import sys
from unittest import TestCase, mock, skipUnless

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from soak_uniforms import process_rss_kib


@skipUnless(os.name == "nt", "native Windows process-memory API")
class WindowsMemoryTests(TestCase):
    def test_working_set_is_sampled_without_a_subprocess(self):
        with mock.patch("subprocess.run", side_effect=AssertionError("unexpected shell launch")):
            self.assertGreater(process_rss_kib(os.getpid()), 0)

    def test_exited_child_and_invalid_pid_are_not_reported_as_alive(self):
        child = subprocess.Popen([sys.executable, "-c", "pass"])
        child.wait(timeout=10)
        self.assertIn(process_rss_kib(child.pid), (None, 0))
        self.assertIsNone(process_rss_kib(-1))
