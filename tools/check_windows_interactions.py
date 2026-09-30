#!/usr/bin/env python3
"""Short native Windows window/reload checks on one owned release renderer.

Exercises actual OS minimize/restore, OSC fullscreen/resize/border controls and
watched reloads. --all-monitors additionally moves and fullscreens the owned
window on every connected display, checking native geometry, DPI and GPU target
allocation. Does not qualify physical input, visual appearance or sleep/wake.
"""
import argparse
from contextlib import contextmanager
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
from soak_uniforms import FeedbackResize
from window_checks import check_texture_resize


class MonitorInfo(ctypes.Structure):
    _fields_ = [("cbSize", w.DWORD), ("rcMonitor", w.RECT), ("rcWork", w.RECT),
                ("dwFlags", w.DWORD), ("szDevice", w.WCHAR * 32)]


def rect_values(rect):
    return [rect.left, rect.top, rect.right, rect.bottom]


@contextmanager
def physical_coordinates():
    """Avoid DPI virtualization of the observer's Win32 geometry queries.

    https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddpiawarenesscontext
    """
    api = ctypes.WinDLL("user32", use_last_error=True)
    change = api.SetThreadDpiAwarenessContext
    change.argtypes, change.restype = [w.HANDLE], w.HANDLE
    previous = change(-4)  # DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
    if not previous:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        yield
    finally:
        change(previous)


def check_monitor_sample(metrics, native, monitor, fullscreen):
    """Require independent native evidence; OSC acknowledgement alone is insufficient."""
    FeedbackResize.validate_metrics(metrics)
    if native["monitor"]["device"] != monitor["device"]:
        raise RuntimeError("owned window did not reach the requested monitor")
    if metrics[2:4] != native["client_pixels"]:
        raise RuntimeError("OSC framebuffer size differs from native client size")
    if native["dpi"] <= 0 or abs(metrics[4] - native["dpi"] / 96) > 1e-6:
        raise RuntimeError("OSC scale differs from native window DPI")
    if metrics[7] != int(fullscreen):
        raise RuntimeError("fullscreen flag did not reach the requested state")
    rectangle, bounds = native["outer"], monitor["bounds"]
    if fullscreen:
        if rectangle != bounds or metrics[2:4] != [bounds[2] - bounds[0], bounds[3] - bounds[1]]:
            raise RuntimeError("fullscreen does not cover the requested monitor")
    elif not (bounds[0] <= rectangle[0] < rectangle[2] <= bounds[2]
              and bounds[1] <= rectangle[1] < rectangle[3] <= bounds[3]):
        raise RuntimeError("window is not fully inside the requested monitor")
    if any(abs(metrics[index + 5] - rectangle[index] / metrics[4]) > 1 for index in (0, 1)):
        raise RuntimeError("OSC logical position differs from native window position")


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
            "GetClientRect": ([w.HWND, ctypes.POINTER(w.RECT)], w.BOOL),
            "GetWindowRect": ([w.HWND, ctypes.POINTER(w.RECT)], w.BOOL),
            "GetDpiForWindow": ([w.HWND], w.UINT),
            "MonitorFromWindow": ([w.HWND, w.DWORD], w.HANDLE),
            "GetMonitorInfoW": ([w.HANDLE, ctypes.POINTER(MonitorInfo)], w.BOOL),
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

    def monitor_info(self, handle):
        info = MonitorInfo()
        info.cbSize = ctypes.sizeof(info)
        if not self.api.GetMonitorInfoW(handle, ctypes.byref(info)):
            raise ctypes.WinError(ctypes.get_last_error())
        return dict(device=info.szDevice, bounds=rect_values(info.rcMonitor),
                    work=rect_values(info.rcWork), primary=bool(info.dwFlags & 1))

    def monitors(self):
        # https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors
        callback_type = ctypes.WINFUNCTYPE(w.BOOL, w.HANDLE, w.HDC, ctypes.POINTER(w.RECT), w.LPARAM)
        enumerate_monitors = self.api.EnumDisplayMonitors
        enumerate_monitors.argtypes = [w.HDC, ctypes.POINTER(w.RECT), callback_type, w.LPARAM]
        enumerate_monitors.restype = w.BOOL
        handles = []

        @callback_type
        def collect(handle, _dc, _rect, _data):
            handles.append(handle)
            return True

        if not enumerate_monitors(None, None, collect, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        monitors = [self.monitor_info(handle) for handle in handles]
        if not monitors or len({item["device"] for item in monitors}) != len(monitors):
            raise RuntimeError("empty or ambiguous native monitor inventory")
        return monitors

    def snapshot(self):
        self.check_owner()
        client, outer = w.RECT(), w.RECT()
        for function, rectangle in ((self.api.GetClientRect, client), (self.api.GetWindowRect, outer)):
            if not function(self.handle, ctypes.byref(rectangle)):
                raise ctypes.WinError(ctypes.get_last_error())
        # https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow
        return dict(monitor=self.monitor_info(self.api.MonitorFromWindow(self.handle, 2)),
                    client_pixels=[client.right - client.left, client.bottom - client.top],
                    outer=rect_values(outer), dpi=self.api.GetDpiForWindow(self.handle))


def until(predicate, description, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.025)
    raise TimeoutError(description)


def exercise(owner, fixture, source, record, all_monitors=False):
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
        def sample():
            state = query("status")
            return state if state[1] > previous else None
        return until(sample, "frames stopped after window change")

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
    if all_monitors:
        monitors = native.monitors()
        record["displays"] = monitors
        observed_dpi = set()
        previous_metrics, previous_state = query("window/metrics"), query("status")
        for index, monitor in enumerate(monitors):
            record["stage"] = f"monitor_{index}_move"
            client.send("/scshader/v1/window/resize", "ii", (640, 360))
            metrics_where(lambda args: args[:2] == [640, 360])
            ratio = query("window/metrics")[4]
            # OSC positions use the current window's logical coordinate system.
            position = [round((value + 48) / ratio) for value in monitor["work"][:2]]
            client.send("/scshader/v1/window/position", "ii", tuple(position))
            until(lambda: native.snapshot()["monitor"]["device"] == monitor["device"],
                  "owned window did not move to the requested display")
            for fullscreen in (False, True, False):
                name = f"monitor_{index}_{'fullscreen' if fullscreen else 'windowed'}"
                if name in record["checks"]:
                    name += "_restored"
                record["stage"] = name
                frame = query("status")[1]
                client.send("/scshader/v1/window/fullscreen", "i", (int(fullscreen),))
                metrics_where(lambda args: args[7] == int(fullscreen))
                progress(frame)

                def settled_sample():
                    metrics, snapshot = query("window/metrics"), native.snapshot()
                    record["last_monitor_sample"] = dict(metrics=metrics, native=snapshot)
                    try:
                        check_monitor_sample(metrics, snapshot, monitor, fullscreen)
                    except RuntimeError as error:
                        record["last_monitor_mismatch"] = str(error)
                        return None
                    record.pop("last_monitor_mismatch", None)
                    return metrics, snapshot

                metrics, snapshot = until(settled_sample, f"native geometry/DPI did not settle for {name}")
                state = progress(query("status")[1])
                check_texture_resize(previous_metrics, previous_state, metrics, state)
                passed(name, dict(metrics=metrics, native=snapshot, frame=state[1], texture_bytes=state[13]))
                observed_dpi.add(snapshot["dpi"])
                previous_metrics, previous_state = metrics, state
        record["monitor_moves_tested"] = len(monitors) > 1
        record["mixed_dpi_moves_tested"] = len(observed_dpi) > 1
    record["stage"] = "window_close"
    final = query("status")
    if final[15] != 0 or final[17] != 0:
        raise RuntimeError("unexpected device recovery or suppressed diagnostics")
    client.send("/scshader/v1/window/close")
    if owner.process.wait(timeout=5) != 0:
        raise RuntimeError("window close did not produce a clean exit")
    passed("window_close", dict(exit_code=0))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--adapter", required=True, help="unique adapter name substring")
    parser.add_argument("--port", type=int, default=57244)
    parser.add_argument("--all-monitors", action="store_true", help="move and fullscreen the owned window on every display")
    args = parser.parse_args(argv)
    if os.name != "nt":
        parser.error("requires an interactive native Windows desktop")
    if not args.renderer.is_file() or args.output_dir.exists() or not 1 <= args.port <= 65535:
        parser.error("renderer must exist, output directory must be new, port must be valid")
    args.output_dir.mkdir(parents=True)
    record = dict(passed=False, checks={}, requested_adapter=args.adapter,
                  started_utc=datetime.now(timezone.utc).isoformat(),
                  renderer_sha256=hashlib.sha256(args.renderer.read_bytes()).hexdigest(),
                  physical_input_tested=False, sleep_wake_tested=False,
                  mixed_dpi_moves_tested=False, monitor_moves_tested=False, stage="startup")
    source = (Path(__file__).resolve().parents[1] / "renderer/shaders/fullscreen.wgsl").read_text(encoding="utf-8")
    try:
        with tempfile.TemporaryDirectory(prefix="fixture-", dir=args.output_dir) as temporary:
            fixture = Path(temporary).resolve() / "interaction.wgsl"
            fixture.write_text(source, encoding="utf-8")
            with physical_coordinates(), ManagedRenderer(args.renderer, args.port, args.output_dir / "renderer.log",
                                 extra_args=["--backend", "dx12", "--adapter", args.adapter]) as owner:
                record["stage"] = "interactions"
                exercise(owner, fixture, source, record, args.all_monitors)
        record["passed"] = True
        record["stage"] = "complete"
    except (OSError, RuntimeError, TimeoutError) as error:
        record["error"] = str(error)
        print(f"FAIL: {error}", flush=True)
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        (args.output_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
