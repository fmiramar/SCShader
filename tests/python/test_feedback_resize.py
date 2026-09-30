from collections import deque
from contextlib import nullcontext
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from soak_uniforms import FeedbackResize, feedback_resize_interval, run


def metrics(width=960, height=540, scale=2):
    return [width, height, width * scale, height * scale, scale, 0, 0, 0, 0, 1, 1, "Test"]


class FeedbackResizeTests(unittest.TestCase):
    def test_short_run_cadence_and_unchanged_long_run_cadence(self):
        self.assertEqual(feedback_resize_interval(10), 2.5)
        self.assertEqual(feedback_resize_interval(3), 0.75)
        self.assertEqual(feedback_resize_interval(3600), 30)

    def test_initial_large_window_starts_with_a_real_shrink(self):
        resize = FeedbackResize(metrics(1280, 720))
        self.assertEqual(resize.request(1), (960, 540))
        resize.reply(metrics(), 2)
        resize.complete()
        self.assertEqual(resize.request(3), (1280, 720))

    def test_stale_metrics_cannot_confirm_a_resize(self):
        resize = FeedbackResize(metrics())
        with self.assertRaisesRegex(RuntimeError, "missing or unconfirmed"):
            resize.complete()
        resize.request(1)
        resize.reply(metrics(), 2)
        self.assertEqual(len(resize.observations), 0)
        with self.assertRaisesRegex(RuntimeError, "not confirmed"):
            resize.check(7)
        with self.assertRaisesRegex(RuntimeError, "unconfirmed"):
            resize.complete()
        resize.reply(metrics(1280, 720), 7)
        resize.complete()

    def test_retina_framebuffer_mapping_is_checked(self):
        resize = FeedbackResize(metrics())
        resize.request(1)
        invalid = metrics(1280, 720)
        invalid[2:4] = [960, 540]
        with self.assertRaisesRegex(RuntimeError, "logical/framebuffer"):
            resize.reply(invalid, 2)
        for invalid in [[], metrics()[:-1], metrics(scale=float("nan")), metrics(0, 0)]:
            with self.assertRaisesRegex(RuntimeError, "logical/framebuffer"):
                FeedbackResize(invalid)

    def exercise(self, accepts_resize):
        """Run the actual timed soak loop with a simulated clock/OSC renderer."""
        clock = SimpleNamespace(now=0.0)
        replies = deque()
        geometry = metrics()

        def sleep(seconds):
            clock.now += seconds

        def send(address, tags="", values=()):
            if address.endswith("/ping"):
                replies.append(("/scshader/v1/pong", [values[0], values[1], clock.now]))
            elif address.endswith("/status"):
                replies.append((address + ".reply", [60, int(clock.now * 60), -1, 1, 0, 1, 0, 0,
                                                     1, 0, 0, 0, 0, 4096, 0, 0, 1, 0]))
            elif address.endswith("/window/metrics"):
                replies.append((address + ".reply", list(geometry)))
            elif address.endswith("/window/resize") and accepts_resize:
                geometry[:] = metrics(*values)

        def expect(address, **kwargs):
            path, values = replies.popleft()
            self.assertEqual(path, address)
            return values

        client = SimpleNamespace(send=send, expect=expect,
                                 receive=lambda: replies.popleft() if replies else None,
                                 send_packet=lambda packet: replies.append(
                                     ("/scshader/v1/error", ["error", "protocol", 0, "E_PROTOCOL", "test"])))
        owner = SimpleNamespace(client=client, process=SimpleNamespace(pid=123, poll=lambda: None),
                                ready_reply=[1, "0.0.18", "Metal", "Simulated GPU"])
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "renderer"
            binary.write_bytes(b"fixture")
            args = SimpleNamespace(renderer=binary, renderer_log=None, port=57166, adapter=None,
                                   power_preference="high-performance", backend="metal", window_system="auto",
                                   overlay=False, mode="feedback", csv=None, max_rss_mib=512,
                                   max_rss_growth_mib=128, memory_warmup=30, rate=1000, quit=False)
            with (patch("soak_uniforms.ManagedRenderer", return_value=nullcontext(owner)),
                  patch("soak_uniforms.time.monotonic", side_effect=lambda: clock.now),
                  patch("soak_uniforms.time.sleep", side_effect=sleep),
                  patch("soak_uniforms.process_rss_kib", return_value=1000),
                  patch("builtins.print")):
                result = {}
                run(args, 10, result)
                return result

    def test_ten_second_feedback_loop_resizes_and_confirms_retina_sizes(self):
        result = self.exercise(True)
        self.assertEqual(result["resize_actions"], 3)
        self.assertEqual(result["resize_confirmations"], 3)
        self.assertEqual([sample["requested"] for sample in result["resize_observations"]],
                         [[1280, 720], [960, 540], [1280, 720]])

    def test_live_loop_rejects_renderer_ignoring_resize_requests(self):
        with self.assertRaisesRegex(RuntimeError, "not confirmed"):
            self.exercise(False)
