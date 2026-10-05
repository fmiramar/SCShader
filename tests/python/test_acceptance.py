from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from run_acceptance import arguments, qualifications, validate_result


class AcceptanceTests(unittest.TestCase):
    def test_default_is_one_hour_traffic_only(self):
        args, seconds = arguments(["--renderer", "renderer", "--output-dir", "new-output"])
        self.assertEqual(seconds, 3600)
        self.assertEqual(args.modes, ["traffic"])
        self.assertFalse(args.final_eight_hour)

    def test_final_eight_hour_is_explicit_and_short_modes_are_opt_in(self):
        base = ["--renderer", "renderer", "--output-dir", "new-output"]
        args, seconds = arguments(base + ["--final-eight-hour"])
        self.assertEqual(seconds, 28800)
        self.assertEqual(args.modes, ["traffic"])
        args, seconds = arguments(base + ["--seconds", "3", "--modes", "trivial", "reload", "feedback"])
        self.assertEqual(seconds, 3)
        self.assertEqual(args.modes, ["trivial", "reload", "feedback"])

    def test_qualification_does_not_promote_short_runs_or_a_single_mode_to_a_suite(self):
        self.assertFalse(any(qualifications(["traffic"], 3).values()))
        one_hour = qualifications(["traffic"], 3600)
        self.assertTrue(one_hour["qualifies_as_one_hour_check"])
        self.assertFalse(one_hour["qualifies_as_eight_hour_check"])
        eight_hour = qualifications(["traffic"], 28800)
        self.assertTrue(eight_hour["qualifies_as_eight_hour_check"])
        self.assertFalse(eight_hour["qualifies_as_eight_hour_suite"])

    def result(self):
        return dict(passed=True, mode="traffic", renderer_sha256="expected", requested_seconds=28800,
                    elapsed_seconds=28800.1, rss_monitoring=True, requested_rate=1000, overlay_enabled=False,
                    max_rss_mib=512, max_rss_growth_mib=128, memory_warmup_seconds=30, sender_pid=1234)

    def test_exact_completed_candidate_is_accepted(self):
        validate_result(self.result(), "traffic", 28800, "expected")

    def test_feedback_requires_completed_resizes_even_in_short_runs(self):
        result = self.result()
        result.update(mode="feedback", requested_seconds=10, elapsed_seconds=10.01)
        for counts in ({}, {"resize_actions": 0}, {"resize_actions": 3, "resize_confirmations": 0},
                       {"resize_actions": 3, "resize_confirmations": 2}):
            with self.assertRaisesRegex(RuntimeError, "resize evidence"):
                validate_result(dict(result, **counts), "feedback", 10, "expected")
        result.update(resize_actions=3, resize_confirmations=3)
        validate_result(result, "feedback", 10, "expected")

    def test_short_different_failed_and_unmonitored_results_do_not_pass(self):
        for key, value in [("passed", False), ("mode", "reload"), ("renderer_sha256", "changed"),
                           ("requested_seconds", 60), ("elapsed_seconds", 60),
                           ("elapsed_seconds", float("nan")), ("rss_monitoring", False),
                           ("max_rss_mib", 1024), ("max_rss_growth_mib", 256),
                           ("memory_warmup_seconds", 200), ("requested_rate", 100), ("overlay_enabled", True),
                           ("sender_pid", 0)]:
            result = self.result()
            result[key] = value
            with self.assertRaises(RuntimeError):
                validate_result(result, "traffic", 28800, "expected")

    def test_missing_result_is_not_a_pass(self):
        with self.assertRaises(RuntimeError):
            validate_result({}, "traffic", 28800, "expected")

    def test_backend_and_display_requests_must_match_evidence(self):
        result = self.result()
        with self.assertRaises(RuntimeError):
            validate_result(result, "traffic", 28800, "expected", "vulkan", "wayland")
        result.update(requested_backend="vulkan", requested_window_system="wayland")
        validate_result(result, "traffic", 28800, "expected", "vulkan", "wayland")
        args, _ = arguments(["--renderer", "renderer", "--output-dir", "new", "--seconds", "10",
                             "--backend", "dx12", "--window-system", "auto"])
        self.assertEqual(args.backend, "dx12")

    def test_hybrid_gpu_evidence_must_match_request_and_reported_device(self):
        result = self.result()
        result.update(requested_adapter="NVIDIA", requested_power_preference="high-performance",
                      device="NVIDIA GeForce GTX 1050 Ti")
        validate_result(result, "traffic", 28800, "expected", adapter="NVIDIA")
        with self.assertRaises(RuntimeError):
            validate_result(result, "traffic", 28800, "expected", adapter="Intel")
        result["device"] = "Intel UHD Graphics"
        with self.assertRaises(RuntimeError):
            validate_result(result, "traffic", 28800, "expected", adapter="NVIDIA")
        args, _ = arguments(["--renderer", "renderer", "--output-dir", "new", "--seconds", "10",
                             "--adapter", "Intel", "--power-preference", "low-power"])
        self.assertEqual(args.adapter, "Intel")
        self.assertEqual(args.power_preference, "low-power")

    def test_window_position_request_is_signed_and_must_match_evidence(self):
        args, _ = arguments(["--renderer", "renderer", "--output-dir", "new", "--seconds", "10",
                             "--position", "-1856", "64"])
        self.assertEqual(args.position, [-1856, 64])
        result = self.result()
        result["requested_window_position"] = [-1856, 64]
        validate_result(result, "traffic", 28800, "expected", position=[-1856, 64])
        with self.assertRaises(RuntimeError):
            validate_result(result, "traffic", 28800, "expected", position=[64, 64])
