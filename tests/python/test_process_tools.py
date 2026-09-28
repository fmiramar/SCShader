import subprocess
import sys
from pathlib import Path
from unittest import TestCase, mock, skipUnless

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import process_tools


class ProcessTests(TestCase):
    def child(self):
        return mock.Mock(pid=12345, poll=mock.Mock(return_value=None))

    def test_windows_spawn_and_pid_scoped_cleanup(self):
        with mock.patch.object(process_tools.os, "name", "nt"), mock.patch.object(process_tools.subprocess, "run") as run:
            self.assertEqual(process_tools.spawn_options(), {"creationflags": 0x200})
            process_tools.stop_process_tree(self.child())
            self.assertEqual(run.call_args.args[0], ["taskkill", "/PID", "12345", "/T", "/F"])
            self.assertEqual(run.call_args.kwargs["timeout"], 5)

    def test_windows_exited_child_is_not_targeted(self):
        child = self.child()
        child.poll.return_value = 0
        with mock.patch.object(process_tools.os, "name", "nt"), mock.patch.object(process_tools.subprocess, "run") as run:
            process_tools.stop_process_tree(child)
            run.assert_not_called()

    def test_windows_taskkill_timeout_has_bounded_fallback(self):
        child = self.child()
        with mock.patch.object(process_tools.os, "name", "nt"), mock.patch.object(
                process_tools.subprocess, "run", side_effect=subprocess.TimeoutExpired("taskkill", 5)):
            process_tools.stop_process_tree(child)
            child.kill.assert_called_once()
            child.wait.assert_called_with(timeout=5)

    def test_posix_session_cleanup_and_exit_race(self):
        with mock.patch.object(process_tools.os, "name", "posix"), mock.patch.object(
                process_tools.os, "killpg", create=True, side_effect=ProcessLookupError) as kill:
            self.assertEqual(process_tools.spawn_options(), {"start_new_session": True})
            process_tools.stop_process_tree(self.child())
            self.assertEqual(kill.call_args.args[0], 12345)

    def test_invalid_pid_never_signals(self):
        with self.assertRaises(ValueError):
            process_tools.stop_process_tree(mock.Mock(pid=0))


@skipUnless(sys.platform == "win32", "native Windows job ownership")
class WindowsOwnershipTests(TestCase):
    def assert_descendant_cleanup(self, exit_parent):
        from process_memory import windows_api
        api, _ = windows_api()
        code = ("import subprocess, sys, time; "
                "child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'], "
                "stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); "
                "print(child.pid, flush=True); " + ("" if exit_parent else "time.sleep(30)"))
        process = process_tools.spawn_owned([sys.executable, "-c", code], stdout=subprocess.PIPE, text=True)
        handle = None
        try:
            # Bound the reader as well as the process in case child startup fails.
            from concurrent.futures import ThreadPoolExecutor
            pool = ThreadPoolExecutor(max_workers=1)
            try:
                pid = int(pool.submit(process.stdout.readline).result(timeout=5))
            finally:
                pool.shutdown(wait=False)
            handle = api.OpenProcess(0x100000, False, pid)  # SYNCHRONIZE, exact owned child
            self.assertTrue(handle)
            if exit_parent:
                self.assertEqual(process.wait(timeout=5), 0)
            self.assertEqual(api.WaitForSingleObject(handle, 0), 258)
            process_tools.stop_process_tree(process)
            self.assertEqual(api.WaitForSingleObject(handle, 5000), 0)
            process_tools.stop_process_tree(process)  # cleanup is idempotent
        finally:
            process_tools.stop_process_tree(process)
            process.stdout.close()
            if handle:
                api.CloseHandle(handle)

    def test_cleanup_after_parent_exit(self):
        self.assert_descendant_cleanup(True)

    def test_cleanup_of_live_parent_tree(self):
        self.assert_descendant_cleanup(False)
