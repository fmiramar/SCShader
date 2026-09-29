import copy
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
from unittest import TestCase, mock

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import check_linux_interactions as checks


def window(**changes):
    result = dict(address="0x123abc", pid=123, title=checks.TITLE, mapped=True,
                  hidden=False, floating=True, xwayland=False, size=[960, 540], fullscreen=0, monitor=0)
    result.update(changes)
    return result


class OwnershipTests(TestCase):
    def setUp(self):
        self.process = mock.Mock(pid=123)
        self.process.poll.return_value = None
        self.hyprctl = mock.Mock()
        self.hyprctl.clients.return_value = [window()]
        self.observations = {}

    def owned(self, xwayland=False):
        return checks.OwnedWindow(self.process, self.hyprctl, xwayland, self.observations)

    def test_delayed_mapping_and_helper_windows_do_not_select_another_process(self):
        unrelated = window(pid=456, address="0x555", title="private unrelated title")
        self.hyprctl.clients.side_effect = [
            [unrelated, window(mapped=False)],
            [unrelated, window(), window(title="driver helper", address="0x222")],
        ]
        with mock.patch.object(checks.time, "sleep"):
            native = self.owned()
        self.assertEqual(native.address, "0x123abc")
        self.assertNotIn("private unrelated title", json.dumps(self.observations))
        self.hyprctl.call.assert_not_called()

    def test_ambiguous_windows_are_rejected(self):
        self.hyprctl.clients.return_value = [window(), window(address="0x222")]
        with self.assertRaisesRegex(RuntimeError, "ambiguous"):
            self.owned()

    def test_wrong_display_and_unsafe_addresses_are_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "display protocol"):
            self.owned(xwayland=True)
        self.hyprctl.clients.return_value = [window(address='0x123"}); bad()')]
        with self.assertRaisesRegex(RuntimeError, "address"):
            self.owned()

    def test_only_the_owned_address_is_floated(self):
        native = self.owned()
        native.float()
        self.hyprctl.call.assert_called_once_with(
            "eval", 'hl.dispatch(hl.dsp.window.float({action="set", window="address:0x123abc"}))')

    def test_compositor_resize_waits_for_owned_window_geometry(self):
        native = self.owned()
        self.hyprctl.clients.side_effect = [
            [window()], [window()], [window(size=[900, 500])],
        ]
        with mock.patch.object(checks.time, "sleep") as sleep:
            native.resize(900, 500)
        self.hyprctl.call.assert_called_once_with(
            "eval", 'hl.dispatch(hl.dsp.window.resize({x=900, y=500, relative=false, window="address:0x123abc"}))')
        sleep.assert_called_once()
        self.assertEqual(self.observations["owned_windows"][0]["size"], [900, 500])

    def test_dead_process_reused_address_and_changed_title_prevent_mutation(self):
        native = self.owned()
        for replacement in (window(pid=456), window(address="0x456"), window(title="another window")):
            with self.subTest(replacement=replacement):
                self.hyprctl.clients.return_value = [replacement]
                for action in (native.float, lambda: native.resize(900, 500)):
                    with self.assertRaises(RuntimeError):
                        action()
                self.hyprctl.call.assert_not_called()
        self.process.poll.return_value = 0
        for action in (native.float, lambda: native.resize(900, 500)):
            with self.assertRaisesRegex(RuntimeError, "exited"):
                action()
        self.hyprctl.call.assert_not_called()


class HyprctlTests(TestCase):
    def test_text_error_even_with_zero_exit_is_not_success(self):
        with mock.patch.object(checks.subprocess, "run", return_value=subprocess.CompletedProcess(
                [], 0, "Lua error: unknown dispatcher", "")):
            with self.assertRaisesRegex(RuntimeError, "did not acknowledge"):
                checks.Hyprctl().call("eval", "expression")

    def test_calls_have_timeout_and_do_not_use_shell_interpolation(self):
        with mock.patch.object(checks.subprocess, "run", return_value=subprocess.CompletedProcess(
                [], 0, "[]", "")) as run:
            self.assertEqual(checks.Hyprctl().clients(), [])
            run.assert_called_once_with(["hyprctl", "-j", "clients"],
                                        capture_output=True, text=True, timeout=2)

    def test_invalid_json_shape_is_rejected(self):
        for output in ('{"error": "no session"}', '["not a window"]'):
            with self.subTest(output=output), mock.patch.object(checks.subprocess, "run", return_value=
                    subprocess.CompletedProcess([], 0, output, "")):
                with self.assertRaisesRegex(RuntimeError, "window list"):
                    checks.Hyprctl().clients()

    def test_ipc_failure_is_saved_before_starting_renderer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            renderer = root / "renderer"
            renderer.write_bytes(b"test fixture, never executed")
            output = root / "evidence"
            with mock.patch.dict(checks.os.environ, {"HYPRLAND_INSTANCE_SIGNATURE": "test"}), \
                    mock.patch.object(checks.sys, "platform", "linux"), \
                    mock.patch.object(checks.Hyprctl, "call", side_effect=RuntimeError("IPC unavailable")), \
                    mock.patch.object(checks, "ManagedRenderer") as managed, \
                    mock.patch("builtins.print"):
                result = checks.main(["--renderer", str(renderer), "--output-dir", str(output),
                                      "--adapter", "NVIDIA", "--window-system", "wayland"])
            self.assertEqual(result, 1)
            managed.assert_not_called()
            record = json.loads((output / "run.json").read_text())
            self.assertFalse(record["passed"])
            self.assertEqual(record["stage"], "preflight")
            self.assertIn("IPC unavailable", record["error"])
            self.assertEqual(len(record["renderer_sha256"]), 64)
            self.assertIn("finished_utc", record)


class ResizeRegressionTests(TestCase):
    def samples(self, width=960, height=540, ratio=1):
        pixels = [int(width * ratio), int(height * ratio)]
        metrics = [width, height, *pixels, ratio, 0, 0, 0, 1, 1, 1, checks.TITLE]
        state = [60, 120, -1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0,
                 8 + pixels[0] * pixels[1] * 16, 0, 0, 1, 0]
        return metrics, state

    def test_os_metrics_alone_cannot_hide_stale_gpu_targets(self):
        old_metrics, old_state = self.samples()
        metrics, state = self.samples(640, 360)
        state[13] = old_state[13]
        with self.assertRaisesRegex(RuntimeError, "GPU texture allocation"):
            checks.check_texture_resize(old_metrics, old_state, metrics, state)

    def test_resize_growth_shrink_and_hidpi_use_framebuffer_area(self):
        old_metrics, old_state = self.samples()
        for dimensions in ((640, 360, 1), (1280, 720, 1), (640, 360, 2), (960, 540, 1)):
            with self.subTest(dimensions=dimensions):
                checks.check_texture_resize(old_metrics, old_state, *self.samples(*dimensions))

    def test_diagnostics_and_unexpected_resources_remain_failures(self):
        metrics, original = self.samples()
        for index, value in ((4, 1), (5, 2), (6, 1), (7, 1), (9, 64), (10, 1), (11, 1),
                             (12, 1), (13, float("nan")), (14, 1), (15, 1), (16, 0), (17, 1)):
            state = copy.copy(original)
            state[index] = value
            with self.subTest(index=index), self.assertRaises(RuntimeError):
                checks.check_texture_resize(metrics, original, metrics, state)

    def test_same_pixel_area_cannot_hide_an_allocation_leak(self):
        metrics, original = self.samples()
        state = copy.copy(original)
        state[13] += 4096
        with self.assertRaisesRegex(RuntimeError, "GPU texture allocation"):
            checks.check_texture_resize(metrics, original, metrics, state)


class CoordinateTests(TestCase):
    def test_xwayland_compositor_and_renderer_scaling_are_independent(self):
        process = mock.Mock(pid=123)
        process.poll.return_value = None
        hyprctl = mock.Mock()
        observations = {}
        hyprctl.clients.return_value = [window(xwayland=True, size=[900, 500])]
        native = checks.OwnedWindow(process, hyprctl, True, observations)
        # Captured failure: Xft 150%, compositor 100%, 900x500 framebuffer.
        metrics = [600, 333, 900, 500, 1.5, 0, 0, 0, 1, 1, 1, checks.TITLE]
        hyprctl.call.side_effect = lambda command, *args, **kwargs: (
            {"int": 1} if command == "getoption" else [{"id": 0, "scale": 1}])
        self.assertTrue(native.size_matches(metrics, [900, 500]))
        self.assertTrue(native.size_matches(metrics))
        self.assertFalse(native.size_matches(metrics, [600, 333]))
        self.assertFalse(native.size_matches([600, 333, 1440, 810, *metrics[4:]], [900, 500]))
        self.assertEqual(observations["xwayland_geometry"]["compositor_to_pixels"], 1)

        # A scaled monitor with force_zero_scaling changes compositor-to-pixel
        # conversion; this is not the renderer's independent DPI ratio.
        hyprctl.clients.return_value = [window(xwayland=True, size=[600, 333])]
        hyprctl.call.side_effect = lambda command, *args, **kwargs: (
            {"int": 1} if command == "getoption" else [{"id": 0, "scale": 1.5}])
        self.assertTrue(native.size_matches(metrics, [600, 333]))
        self.assertTrue(native.size_matches(metrics))
        hyprctl.call.side_effect = None
        hyprctl.call.return_value = {"int": 0}
        self.assertFalse(native.size_matches(metrics))
        hyprctl.clients.return_value = [window(xwayland=True, size=[900, 500])]
        self.assertTrue(native.size_matches(metrics))

    def test_bool_and_legacy_integer_options_produce_the_same_geometry(self):
        process = mock.Mock(pid=123)
        process.poll.return_value = None
        hyprctl = mock.Mock()
        hyprctl.clients.return_value = [window(xwayland=True, size=[600, 333])]
        observations = {}
        native = checks.OwnedWindow(process, hyprctl, True, observations)
        for option in ({"bool": True}, {"int": 1}, {"bool": False}, {"int": 0}):
            with self.subTest(option=option):
                hyprctl.call.side_effect = lambda command, *args, **kwargs: (
                    option if command == "getoption" else [{"id": 0, "scale": 1.5}])
                enabled = bool(next(iter(option.values())))
                pixels = [900, 500] if enabled else [600, 333]
                metrics = [600, 333, *pixels, 1.5, 0, 0, 0, 1, 1, 1, checks.TITLE]
                self.assertTrue(native.size_matches(metrics, [600, 333]))
                self.assertEqual(observations["xwayland_geometry"]["force_zero_scaling"], enabled)
                self.assertEqual(observations["xwayland_scaling_option"], option)

    def test_unknown_xwayland_scaling_fails_explicitly(self):
        process = mock.Mock(pid=123)
        process.poll.return_value = None
        hyprctl = mock.Mock()
        hyprctl.clients.return_value = [window(xwayland=True)]
        native = checks.OwnedWindow(process, hyprctl, True, {})
        metrics, _ = ResizeRegressionTests().samples()
        for option, monitors in (({}, []), ({"bool": "false"}, []), ({"int": 2}, []),
                                 ({"int": 1}, []), ({"int": 1}, [{"id": 0, "scale": 0}])):
            with self.subTest(option=option, monitors=monitors):
                hyprctl.call.side_effect = [option, monitors]
                with self.assertRaises(RuntimeError):
                    native.size_matches(metrics)


class ResizeDriverTests(TestCase):
    def test_refused_osc_resize_stays_failed_without_automatic_compositor_fallback(self):
        self.exercise_driver("osc", expected_pass=False)

    def test_explicit_compositor_run_does_not_claim_osc_resize_coverage(self):
        self.exercise_driver("compositor", expected_pass=True)

    def test_xwayland_hidpi_osc_and_compositor_resize_paths(self):
        for driver in ("osc", "compositor"):
            with self.subTest(driver=driver):
                record = self.exercise_driver(driver, expected_pass=True, xwayland=True, accept_osc=True)
                self.assertEqual(record["checks"]["compositor_resize_setup"]["metrics"][:5],
                                 [600, 333, 900, 500, 1.5])
                first = record["checks"]["resize_reload_1"]
                self.assertEqual(first["compositor_size"], [960, 540] if driver == "osc" else [640, 360])

    def test_xwayland_hidpi_still_rejects_refused_osc_requests(self):
        self.exercise_driver("osc", expected_pass=False, xwayland=True)

    def test_xwayland_metrics_cannot_hide_stale_compositor_geometry(self):
        self.exercise_driver("osc", expected_pass=False, xwayland=True,
                             accept_osc=True, stale_geometry=True)

    def test_xwayland_scaled_resize_cannot_hide_stale_gpu_targets(self):
        self.exercise_driver("osc", expected_pass=False, xwayland=True,
                             accept_osc=True, stale_gpu=True)

    def exercise_driver(self, driver, expected_pass, xwayland=False, accept_osc=False,
                        stale_geometry=False, stale_gpu=False):
        # Simulate protocol geometry separately from renderer DPI and resource
        # sizes. No desktop/GPU is used; size_matches is the real owned check.
        geometry = [960, 540]
        pixels = geometry.copy()
        ratio = 1.5 if xwayland else 1
        flags = {"fullscreen": 0, "borderless": 1}
        frame = 0
        client, native = mock.Mock(), mock.Mock()
        native.xwayland = xwayland
        native.hyprctl.call.side_effect = lambda command, *args, **kwargs: (
            {"option": "xwayland:force_zero_scaling", "bool": True, "set": True}
            if command == "getoption" else [{"id": 0, "scale": 1}])
        native.observations = {}
        native.size_matches.side_effect = lambda *args: checks.OwnedWindow.size_matches(native, *args)
        native.snapshot.side_effect = lambda: dict(size=geometry.copy(), fullscreen=2 * flags["fullscreen"], monitor=0)

        def resize(width, height):
            geometry[:] = [width, height]
            pixels[:] = geometry

        native.resize.side_effect = resize

        def send(address, tags="", values=()):
            field = address.removeprefix(checks.PREFIX + "window/")
            if field in flags:
                flags[field] = values[0]
            elif field == "resize" and accept_osc:
                pixels[:] = [math.floor(value * ratio + 0.5) for value in values]
                if not stale_geometry:
                    geometry[:] = pixels

        def expect(address, **kwargs):
            nonlocal frame
            logical = [math.floor(value / ratio + 0.5) for value in pixels]
            metrics, state = ResizeRegressionTests().samples(*logical, ratio)
            metrics[2:4] = pixels
            area = 900 * 500 if stale_gpu and record.get("stage") == "resize_reload_1" else pixels[0] * pixels[1]
            state[13] = 8 + area * 16
            if address.endswith("window/metrics.reply"):
                metrics[7:9] = [flags["fullscreen"], flags["borderless"]]
                return metrics
            if address.endswith("status.reply"):
                frame += 1
                state[1] = frame
                return state
            return [701]

        def bounded_sample(sample, description, timeout=5):
            for _ in range(2):
                result = sample()
                if result:
                    return result
            raise TimeoutError(description)

        client.send.side_effect, client.expect.side_effect = send, expect
        owner = mock.Mock(client=client)
        owner.process.wait.return_value = 0
        record = dict(observations={}, checks={}, osc_resize_completed=False)
        with tempfile.TemporaryDirectory() as temporary, \
                mock.patch.object(checks, "until", side_effect=bounded_sample), \
                mock.patch("builtins.print"):
            fixture = Path(temporary) / "fixture.wgsl"
            if expected_pass:
                checks.exercise(owner, native, fixture, "// fixture", record, driver)
                self.assertIn("window_close", record["checks"])
                self.assertEqual(native.resize.call_count, 5 if driver == "compositor" else 1)
                osc_calls = [call for call in client.send.call_args_list
                             if call.args[0] == checks.PREFIX + "window/resize"]
                self.assertEqual(len(osc_calls), 0 if driver == "compositor" else 4)
            else:
                error = (RuntimeError, "GPU texture allocation") if stale_gpu else (TimeoutError, "logical size 640x360")
                with self.assertRaisesRegex(*error):
                    checks.exercise(owner, native, fixture, "// fixture", record, driver)
                native.resize.assert_called_once_with(900, 500)
                self.assertNotIn("resize_reload_1", record["checks"])
        self.assertEqual(record["osc_resize_completed"], expected_pass and driver == "osc")
        return record
