from pathlib import Path
import sys
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import stress_runtime as stress
from stress_runtime import check_resources, check_diagnostic_bound


class StressRuntimeTests(unittest.TestCase):
    def test_exact_resource_cleanup_is_required(self):
        state = [60, 100, -1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1234, 0, 0, 1]
        check_resources(state, (0, 0, 0), 1234)
        for field in (4, 5, 6, 7, 9, 13, 15):
            changed = state.copy()
            changed[field] += 1
            with self.assertRaises(RuntimeError):
                check_resources(changed, (0, 0, 0), 1234)

    def test_diagnostic_bound_is_duration_aware(self):
        check_diagnostic_bound(60, 2)
        with self.assertRaises(RuntimeError):
            check_diagnostic_bound(1000, 2)

    def test_startup_resize_is_settled_before_fixed_size_memory_baseline(self):
        clock = [0.0]
        probe = mock.Mock()
        metrics = lambda w, h: [w, h, w, h, 1, 0, 0, 0, 0, 1, 1, "test"]
        initial, final = metrics(952, 514), metrics(945, 1025)
        probe.metrics.side_effect = lambda: initial if clock[0] < 0.1 else final
        frame = [0]

        def status():
            frame[0] += 1
            size = probe.metrics()
            return [60, frame[0], -1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                    size[2] * size[3] * 16 + 260, 0, 0, 1]

        probe.status.side_effect = status
        result = {}
        with mock.patch.object(stress.time, "monotonic", side_effect=lambda: clock[0]), \
                mock.patch.object(stress.time, "sleep", side_effect=lambda seconds: clock.__setitem__(0, clock[0] + seconds)):
            baseline_metrics, baseline_state = stress.stable_window_baseline(probe, result)
        self.assertEqual(baseline_metrics, final)
        self.assertEqual(baseline_state[13], 15498260)
        self.assertGreaterEqual(clock[0], 0.6)
        self.assertEqual([s["texture_bytes"] for s in result["startup_window_samples"]], [7829508, 15498260])
        # A real allocation leak at the settled size must still fail exactly.
        leaked = baseline_state.copy()
        leaked[13] += 4096
        with self.assertRaisesRegex(RuntimeError, "owned texture bytes"):
            check_resources(leaked, (0, 0, 0), baseline_state[13])

    def test_resize_during_stress_is_reported_without_rebasing(self):
        baseline = [952, 514, 952, 514, 1, 0, 0, 0, 0, 1, 1, "test"]
        changed = [945, 1025, 945, 1025, *baseline[4:]]
        probe = mock.Mock()
        probe.metrics.return_value = changed
        result = {}
        with self.assertRaisesRegex(RuntimeError, "window dimensions changed"):
            stress.check_window_unchanged(probe, baseline, result)
        self.assertEqual(result["last_window_metrics"], changed)
        self.assertEqual(baseline[:2], [952, 514])

    def test_baseline_requires_progress_and_has_a_deadline(self):
        probe = mock.Mock()
        probe.metrics.return_value = [960, 540, 960, 540, 1, 0, 0, 0, 0, 1, 1, "test"]
        probe.status.return_value = [60, 1, -1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 8294660, 0, 0, 1]
        clock = [0.0]
        with mock.patch.object(stress.time, "monotonic", side_effect=lambda: clock[0]), \
                mock.patch.object(stress.time, "sleep", side_effect=lambda seconds: clock.__setitem__(0, clock[0] + seconds)):
            with self.assertRaisesRegex(TimeoutError, "did not settle"):
                stress.stable_window_baseline(probe, {}, timeout=0.7)


if __name__ == "__main__":
    unittest.main()
