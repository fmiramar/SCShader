from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import check_macos_interactions as checks
from macos_window import OwnedWindow, WindowUnavailable


def metrics(width=960, height=540):
    return [width, height, width * 2, height * 2, 2.0, 0, 0, 0, 0, 1, 1, "SCShader verification"]


def state(width=960, height=540):
    return [60, 120, -1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, width * height * 64 + 260, 0, 0, 1, 0]


class OwnershipTests(unittest.TestCase):
    def setUp(self):
        self.process = Mock(pid=123)
        self.process.poll.return_value = None
        self.api = Mock()
        self.api.application.return_value = "application"
        self.api.windows.return_value = ["window"]
        self.api.pid.return_value = 123
        self.api.equal.side_effect = lambda a, b: a == b
        self.api.attribute.side_effect = lambda window, name: {
            "AXTitle": "SCShader verification", "AXRole": "AXWindow",
            "AXSize": [960, 572], "AXPosition": [0, 0],
            "AXMinimized": False, "AXFullScreen": False, "AXFocused": True,
        }[name]
        self.native = OwnedWindow(self.process, self.api)

    def tearDown(self):
        self.native.close()

    def test_delayed_window_is_allowed_but_cannot_be_mutated(self):
        self.api.windows.return_value = []
        self.assertFalse(self.native.discover())
        for action in (lambda: self.native.minimize(True), self.native.close_window):
            with self.assertRaises(WindowUnavailable):
                action()
        self.api.set_minimized.assert_not_called()
        self.api.press_close.assert_not_called()

    def test_only_matching_owned_window_is_selected_and_retained(self):
        self.api.windows.return_value = ["other process", "window"]
        self.api.pid.side_effect = lambda window: 456 if window == "other process" else 123
        self.assertTrue(self.native.discover())
        self.assertEqual(self.native.window, "window")
        self.api.release.assert_called_once_with("other process")
        self.native.minimize(True)
        self.api.set_minimized.assert_called_once_with("window", True)

    def test_ambiguous_windows_are_rejected(self):
        self.api.windows.return_value = ["one", "two"]
        with self.assertRaisesRegex(RuntimeError, "ambiguous"):
            self.native.discover()
        self.assertEqual(self.api.release.call_count, 2)

    def test_fullscreen_temporary_absence_retains_identity(self):
        self.native.discover()
        self.api.windows.return_value = []
        self.assertFalse(self.native.discover())
        self.assertEqual(self.native.window, "window")
        self.api.windows.return_value = ["window"]
        self.assertTrue(self.native.discover())

    def test_replaced_window_is_not_automatically_adopted(self):
        self.native.discover()
        self.api.windows.return_value = ["replacement"]
        for action in (lambda: self.native.minimize(True), self.native.close_window):
            with self.assertRaisesRegex(RuntimeError, "identity changed"):
                action()
        self.api.set_minimized.assert_not_called()
        self.api.press_close.assert_not_called()

    def test_dead_child_rejects_native_mutations(self):
        self.native.discover()
        self.process.poll.return_value = 0
        for action in (lambda: self.native.minimize(True), self.native.close_window):
            with self.assertRaisesRegex(RuntimeError, "exited"):
                action()
        self.api.set_minimized.assert_not_called()
        self.api.press_close.assert_not_called()

    def test_changed_title_or_owner_rejects_native_mutations(self):
        self.native.discover()
        for replacement in (456, 123):
            self.api.pid.return_value = replacement
            if replacement == 123:
                self.api.attribute.side_effect = lambda window, name: "AXWindow" if name == "AXRole" else "different"
            with self.assertRaises(WindowUnavailable):
                self.native.minimize(True)
        self.api.set_minimized.assert_not_called()

    def test_keyboard_events_require_owned_focused_window_and_target_only_its_pid(self):
        self.native.key(53, True)
        self.api.key.assert_called_once_with(123, 53, True)
        self.api.key.reset_mock()
        original = self.api.attribute.side_effect
        self.api.attribute.side_effect = lambda window, name: False if name == "AXFocused" else original(window, name)
        with self.assertRaisesRegex(RuntimeError, "not focused"):
            self.native.key(53, True)
        self.api.key.assert_not_called()


class EvidenceTests(unittest.TestCase):
    def test_retina_pixels_are_required_even_if_logical_size_changed(self):
        sample = metrics(640, 360)
        sample[2:4] = [1920, 1080]
        with self.assertRaisesRegex(RuntimeError, "Retina"):
            checks.validate_metrics(sample)
        for invalid in (metrics()[:-1], metrics(0, 0), metrics()[:4] + [float("nan")] + metrics()[5:]):
            with self.assertRaises(RuntimeError):
                checks.validate_metrics(invalid)

    def test_old_native_frame_cannot_confirm_a_reported_resize(self):
        self.assertFalse(checks.frame_matches(metrics(640, 360), {"size": [960, 572]}, [0, 32]))
        self.assertTrue(checks.frame_matches(metrics(640, 360), {"size": [640, 392]}, [0, 32]))
        self.assertFalse(checks.frame_matches(metrics(), {"size": [960, 572]}, [0, 0]))

    def test_gpu_allocations_must_follow_resize_and_stay_stable_at_same_size(self):
        checks.validate_resize(metrics(), state(), metrics(640, 360), state(640, 360))
        with self.assertRaisesRegex(RuntimeError, "GPU texture"):
            checks.validate_resize(metrics(), state(), metrics(640, 360), state())
        with self.assertRaisesRegex(RuntimeError, "GPU texture"):
            checks.validate_resize(metrics(), state(), metrics(), state(640, 360))

    def test_suppressed_errors_or_device_recovery_cannot_count_as_healthy(self):
        for index in (4, 9, 10, 11, 12, 15, 17):
            sample = state()
            sample[index] = 1
            with self.subTest(index=index), self.assertRaises(RuntimeError):
                checks.validate_health(sample)
        sample = state()
        sample[16] = 0
        with self.assertRaises(RuntimeError):
            checks.validate_health(sample)

    def test_ignored_state_change_times_out(self):
        with patch.object(checks.time, "monotonic", side_effect=[0, 0, 2]), patch.object(checks.time, "sleep"):
            with self.assertRaisesRegex(TimeoutError, "ignored"):
                checks.until(lambda: False, "ignored state change", timeout=1)
