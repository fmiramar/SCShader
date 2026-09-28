#!/usr/bin/env python3
"""Short native Windows window/reload checks on one owned release renderer.

Exercises actual OS minimize/restore, OSC fullscreen/resize/border controls and
watched reloads. Does not simulate physical input or qualify visual appearance,
sleep/wake, multi-monitor moves or long-duration stability.
"""
import argparse
import ctypes
from ctypes import wintypes as w
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import tempfile
import time

from osc_client import ManagedRenderer


class OwnedWindow:
    """Resolve/mutate only a live child process's visible top-level window.

    https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumwindows
    https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
    https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
    """
    def __init__(self, process):
        self.process = process
        self.api = api = ctypes.WinDLL("user32", use_last_error=True)
        callback_type = ctypes.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
        signatures = {
            "EnumWindows": ([callback_type, w.LPARAM], w.BOOL),
            "GetWindowThreadProcessId": ([w.HWND, ctypes.POINTER(w.DWORD)], w.DWORD),
            "IsWindowVisible": ([w.HWND], w.BOOL),
            "GetWindowTextW": ([w.HWND, w.LPWSTR, ctypes.c_int], ctypes.c_int),
            "IsIconic": ([w.HWND], w.BOOL),
            "ShowWindowAsync": ([w.HWND, ctypes.c_int], w.BOOL),
            "GetSystemMetrics": ([ctypes.c_int], ctypes.c_int),
        }
        for name, (arguments, result) in signatures.items():
            function = getattr(api, name)
            function.argtypes, function.restype = arguments, result
        windows = []

        @callback_type
        def collect(handle, _):
            pid = w.DWORD()
            api.GetWindowThreadProcessId(handle, ctypes.byref(pid))
            if pid.value == process.pid and api.IsWindowVisible(handle):
                # Graphics drivers can create additional visible helper HWNDs.
                # ManagedRenderer supplies this exact title for its main window.
                title = ctypes.create_unicode_buffer(256)
                api.GetWindowTextW(handle, title, len(title))
                if title.value == "SCShader verification":
                    windows.append(handle)
            return True

        if not api.EnumWindows(collect, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        if len(windows) != 1:
            raise RuntimeError(f"expected one owned visible window, found {len(windows)}")
        self.handle = windows[0]
        self.check_owner()

    def check_owner(self):
        pid = w.DWORD()
        if (self.process.poll() is not None
                or not self.api.GetWindowThreadProcessId(self.handle, ctypes.byref(pid))
                or pid.value != self.process.pid):
            raise RuntimeError("owned renderer/window no longer exists")

    def minimized(self):
        self.check_owner()
        return bool(self.api.IsIconic(self.handle))

    def show(self, command):
        self.check_owner()
        if not self.api.ShowWindowAsync(self.handle, command):
            raise RuntimeError("ShowWindowAsync could not post the owned-window request")


def until(predicate, description, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.025)
    raise TimeoutError(description)


def exercise(owner, fixture, source, record):
    client = owner.client
    native = OwnedWindow(owner.process)
    if owner.ready_reply[2] != "Dx12" or record["requested_adapter"].lower() not in owner.ready_reply[3].lower():
        raise RuntimeError("renderer did not use the requested D3D12 adapter")
    record.update(version=owner.ready_reply[1], backend=owner.ready_reply[2],
                  device=owner.ready_reply[3], monitor_count=native.api.GetSystemMetrics(80))

    def query(endpoint):
        client.send("/scshader/v1/" + endpoint)
        return client.expect("/scshader/v1/" + endpoint + ".reply", timeout=2)

    def progress(previous):
        return until(lambda: query("status")[1] > previous, "frames stopped after window change")

    def metrics_where(predicate):
        def sample():
            metrics = query("window/metrics")
            return metrics if predicate(metrics) else None
        return until(sample, "native window metrics did not reach requested state")

    def passed(name, details):
        record["checks"][name] = details
        print(f"PASS {name}: {details}", flush=True)

    baseline = query("window/metrics")
    passed("initial_metrics", baseline)
    client.send("/scshader/v1/shader/create", "iss", (701, str(fixture), "wgsl"))
    client.expect("/scshader/v1/shader/created")
    for index, (width, height) in enumerate(((640, 360), (800, 450), (720, 480), (960, 540))):
        frame = query("status")[1]
        # Replace only our generated fixture, atomically, while requesting resize.
        edit = fixture.with_suffix(".next")
        edit.write_text(source + f"\n// interaction reload {index}\n", encoding="utf-8")
        edit.replace(fixture)
        client.send("/scshader/v1/window/resize", "ii", (width, height))
        client.expect("/scshader/v1/shader/reloaded", predicate=lambda args: args[0] == 701)
        metrics = metrics_where(lambda args: args[:2] == [width, height])
        progress(frame)
        passed(f"resize_reload_{index + 1}", metrics[:5])

    for enabled in (1, 0):
        frame = query("status")[1]
        client.send("/scshader/v1/window/fullscreen", "i", (enabled,))
        metrics = metrics_where(lambda args: args[7] == enabled)
        progress(frame)
        passed(f"fullscreen_{enabled}", metrics[:5])

    for enabled in (1, 0):
        client.send("/scshader/v1/window/borderless", "i", (enabled,))
        metrics = metrics_where(lambda args: args[8] == enabled)
        passed(f"borderless_{enabled}", metrics[:5])

    native.show(6)  # SW_MINIMIZE
    until(native.minimized, "native window did not minimize")
    minimized = query("status")  # OSC must still answer while minimized.
    passed("minimize_status", dict(frame=minimized[1]))
    native.show(9)  # SW_RESTORE
    until(lambda: not native.minimized(), "native window did not restore")
    restored_frame = query("status")[1]
    progress(restored_frame)
    restored = query("window/metrics")
    if min(restored[:4]) <= 0:
        raise RuntimeError("restored window has invalid logical/pixel size")
    passed("restore_frames", restored[:5])
    final = query("status")
    if final[15] != 0 or final[17] != 0:
        raise RuntimeError("unexpected device recovery or suppressed diagnostics")
    client.send("/scshader/v1/window/close")
    if owner.process.wait(timeout=5) != 0:
        raise RuntimeError("window close did not produce a clean exit")
    passed("window_close", dict(exit_code=0))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--adapter", required=True, help="unique adapter name substring")
    parser.add_argument("--port", type=int, default=57244)
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("requires an interactive native Windows desktop")
    if not args.renderer.is_file() or args.output_dir.exists() or not 1 <= args.port <= 65535:
        parser.error("renderer must exist, output directory must be new, port must be valid")
    args.output_dir.mkdir(parents=True)
    record = dict(passed=False, checks={}, requested_adapter=args.adapter,
                  started_utc=datetime.now(timezone.utc).isoformat(),
                  renderer_sha256=hashlib.sha256(args.renderer.read_bytes()).hexdigest(),
                  physical_input_tested=False, sleep_wake_tested=False,
                  mixed_dpi_moves_tested=False)
    source = (Path(__file__).resolve().parents[1] / "renderer/shaders/fullscreen.wgsl").read_text(encoding="utf-8")
    try:
        with tempfile.TemporaryDirectory(prefix="fixture-", dir=args.output_dir) as temporary:
            fixture = Path(temporary).resolve() / "interaction.wgsl"
            fixture.write_text(source, encoding="utf-8")
            with ManagedRenderer(args.renderer, args.port, args.output_dir / "renderer.log",
                                 extra_args=["--backend", "dx12", "--adapter", args.adapter]) as owner:
                exercise(owner, fixture, source, record)
        record["passed"] = True
    except (OSError, RuntimeError, TimeoutError) as error:
        record["error"] = str(error)
        print(f"FAIL: {error}", flush=True)
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        (args.output_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
