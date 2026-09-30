import copy
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from check_windows_interactions import OwnedWindow, check_monitor_sample


def sample(scale=1, left=1920, fullscreen=False):
    monitor = dict(device="display-b", bounds=[left, -200, left + 1920, 880])
    pixels = [1920, 1080] if fullscreen else [int(640 * scale), int(360 * scale)]
    outer = list(monitor["bounds"]) if fullscreen else [left + 48, -152, left + 48 + pixels[0], -152 + pixels[1]]
    native = dict(monitor=monitor, outer=outer, client_pixels=pixels, dpi=96 * scale)
    metrics = [round(pixels[0] / scale), round(pixels[1] / scale), *pixels, scale,
               round(outer[0] / scale), round(outer[1] / scale), int(fullscreen), 0, 1, 1, "test"]
    return metrics, native, monitor


class MonitorTests(unittest.TestCase):
    def test_native_geometry_at_positive_and_negative_desktop_origins_and_mixed_dpi(self):
        for left in (-1920, 0, 1920):
            for scale in (1, 1.25, 1.5, 2):
                for fullscreen in (False, True):
                    with self.subTest(left=left, scale=scale, fullscreen=fullscreen):
                        check_monitor_sample(*sample(scale, left, fullscreen), fullscreen)

    def test_stale_osc_framebuffer_cannot_pass_using_only_a_resize_acknowledgement(self):
        metrics, native, monitor = sample(1.5)
        metrics[:5] = [640, 360, 640, 360, 1]
        with self.assertRaisesRegex(RuntimeError, "native client size"):
            check_monitor_sample(metrics, native, monitor, False)

    def test_same_client_pixels_do_not_hide_stale_dpi(self):
        metrics, native, monitor = sample(1.5)
        metrics[:2], metrics[4] = metrics[2:4], 1
        with self.assertRaisesRegex(RuntimeError, "native window DPI"):
            check_monitor_sample(metrics, native, monitor, False)

    def test_move_must_reach_the_requested_monitor_and_be_fully_contained(self):
        metrics, native, monitor = sample()
        wrong = copy.deepcopy(native)
        wrong["monitor"]["device"] = "display-a"
        with self.assertRaisesRegex(RuntimeError, "requested monitor"):
            check_monitor_sample(metrics, wrong, monitor, False)
        native["outer"][2] = monitor["bounds"][2] + 1
        with self.assertRaisesRegex(RuntimeError, "fully inside"):
            check_monitor_sample(metrics, native, monitor, False)

    def test_fullscreen_flag_alone_cannot_pass_on_wrong_native_geometry(self):
        metrics, native, monitor = sample(fullscreen=True)
        native["outer"][0] += 1
        with self.assertRaisesRegex(RuntimeError, "does not cover"):
            check_monitor_sample(metrics, native, monitor, True)

    def test_position_uses_logical_units_at_the_measured_scale(self):
        metrics, native, monitor = sample(2)
        metrics[5] = native["outer"][0]
        with self.assertRaisesRegex(RuntimeError, "logical position"):
            check_monitor_sample(metrics, native, monitor, False)

    def test_invalid_scale_and_framebuffer_relationship_are_rejected(self):
        for scale in (float("nan"), 0, -1):
            metrics, native, monitor = sample()
            metrics[4] = scale
            with self.subTest(scale=scale), self.assertRaisesRegex(RuntimeError, "logical/framebuffer"):
                check_monitor_sample(metrics, native, monitor, False)


class OwnershipTests(unittest.TestCase):
    def test_exited_or_reused_window_prevents_show_and_snapshot(self):
        native = OwnedWindow.__new__(OwnedWindow)
        native.process, native.api, native.handle = Mock(pid=123), Mock(), 1
        for exited in (True, False):
            native.process.poll.return_value = 0 if exited else None
            # Successful PID query that leaves the out-parameter at zero: wrong owner.
            native.api.GetWindowThreadProcessId.return_value = 1
            with self.subTest(exited=exited):
                for action in (lambda: native.show(6), native.snapshot):
                    with self.assertRaisesRegex(RuntimeError, "no longer exists"):
                        action()
                native.api.ShowWindowAsync.assert_not_called()
                native.api.GetClientRect.assert_not_called()
