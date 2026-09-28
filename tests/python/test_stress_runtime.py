from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
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


if __name__ == "__main__":
    unittest.main()
