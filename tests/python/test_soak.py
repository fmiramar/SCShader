from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from osc_client import decode_message, osc_message, osc_bundle
from soak_uniforms import Health, MemoryHealth


class WireTests(unittest.TestCase):
    def test_exact_wire_roundtrip(self):
        values = (-2147483648, 0.25, 123456.75, "control", b"\xff\xff\xff\xff", 4294967295)
        self.assertEqual(decode_message(osc_message("/test", "ifdsbh", values)), ("/test", list(values)))

    def test_rejects_malformed_replies(self):
        valid = osc_message("/test", "s", ("ok",))
        for packet in [valid[:-1], valid + bytes(4), b"/test", osc_message("/test", "i", (1,))[:-1]]:
            with self.assertRaises(ValueError):
                decode_message(packet)

    def test_bundle_header_and_sizes(self):
        packet = osc_message("/test")
        bundle = osc_bundle([packet])
        self.assertEqual(bundle[:16], b"#bundle\0" + struct.pack(">Q", 1))
        self.assertEqual(struct.unpack(">i", bundle[16:20])[0], len(packet))


class HealthTests(unittest.TestCase):
    def test_suppressed_errors_cannot_hide_a_failed_soak(self):
        state = self.status(100) + [0, 4096, 0, 0, 1, 2]
        with self.assertRaisesRegex(RuntimeError, "suppressed"):
            Health(0).reply("/scshader/v1/status.reply", state, 1)

    def status(self, frame, rejected=0, dropped=0):
        return [60.0, frame, -1.0, 1.0, 0, 0, 0, 0, 1, 0, rejected, dropped]

    def test_pongs_do_not_hide_frozen_frames(self):
        health = Health(0)
        for now in range(7):
            health.sent_sequence += 1
            health.reply("/scshader/v1/pong", [health.sent_sequence, 0, now], now)
            health.reply("/scshader/v1/status.reply", self.status(10), now)
        with self.assertRaisesRegex(RuntimeError, "frame progress"):
            health.check(6)

    def test_rejects_unexpected_errors_overload_and_early_completion(self):
        with self.assertRaisesRegex(RuntimeError, "diagnostic"):
            Health(0).reply("/scshader/v1/error", ["error", "gpu", 0, "E_GPU", "lost"], 0)
        for args in [self.status(1, rejected=1), self.status(1, dropped=1)]:
            with self.assertRaisesRegex(RuntimeError, "dropped/rejected"):
                Health(0).reply("/scshader/v1/status.reply", args, 0)
        with self.assertRaisesRegex(RuntimeError, "requested duration"):
            Health(0).complete(1, 60)

    def test_stale_or_unsolicited_pongs_do_not_count(self):
        health = Health(0)
        health.sent_sequence = 1
        health.reply("/scshader/v1/pong", [1, 0, 0], 1)
        health.reply("/scshader/v1/pong", [1, 0, 0], 2)
        health.reply("/scshader/v1/pong", [2, 0, 0], 3)
        self.assertEqual(health.pongs, 1)
        self.assertEqual(health.last_pong, 1)

    def test_healthy_run_has_real_progress(self):
        health = Health(0)
        for now in range(4):
            health.sent_sequence += 1
            health.reply("/scshader/v1/pong", [health.sent_sequence, 0, now], now)
            health.reply("/scshader/v1/status.reply", self.status(now * 60), now)
        health.complete(3, 3)

    def test_rss_bounds_and_missing_pid_are_failures(self):
        memory = MemoryHealth(1000, 100, 1)
        memory.sample(500, 0)
        memory.sample(600, 1)
        memory.sample(690, 2)
        with self.assertRaisesRegex(RuntimeError, "growth"):
            memory.sample(701, 3)
        with self.assertRaisesRegex(RuntimeError, "absolute"):
            MemoryHealth(1000, 100, 10).sample(1001, 0)
        with self.assertRaisesRegex(RuntimeError, "PID"):
            MemoryHealth(1000, 100, 0).sample(None, 0)


if __name__ == "__main__":
    unittest.main()
