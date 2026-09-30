#!/usr/bin/env python3
"""Short native macOS window/reload checks on one owned production renderer.

Requires existing Accessibility permission. Opens/fullscreens/minimizes only its
child's window. Physical input, visible/audio quality and sleep/wake remain manual.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time

from macos_window import Accessibility, OwnedWindow, WindowUnavailable
from osc_client import ManagedRenderer

PREFIX = "/scshader/v1/"


def until(predicate, description, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(0.05)
    raise TimeoutError(description)


def validate_metrics(metrics):
    if (len(metrics) != 12 or min(metrics[:4]) <= 0
            or not math.isfinite(metrics[4]) or metrics[4] <= 0
            or any(abs(metrics[index] * metrics[4] - metrics[index + 2]) > 1 for index in (0, 1))):
        raise RuntimeError("invalid logical/framebuffer Retina mapping")


def validate_health(state):
    if (len(state) != 18 or state[5:8] != [1, 0, 0] or state[14] != 0
            or any(state[index] != 0 for index in (4, 9, 10, 11, 12, 15, 17))
            or state[16] != 1 or not math.isfinite(state[13]) or state[13] <= 0):
        raise RuntimeError("unexpected resource, diagnostic, queue, recovery or compile state")


def validate_resize(before_metrics, before_state, metrics, state):
    for sample in (before_metrics, metrics):
        validate_metrics(sample)
    for sample in (before_state, state):
        validate_health(sample)
    area_change = metrics[2] * metrics[3] - before_metrics[2] * before_metrics[3]
    byte_change = state[13] - before_state[13]
    if ((area_change != 0 and area_change * byte_change <= 0)
            or (area_change == 0 and byte_change != 0)):
        raise RuntimeError("GPU texture allocation did not follow framebuffer area")


def frame_matches(metrics, native, insets):
    validate_metrics(metrics)
    return all(abs(native["size"][index] - metrics[index] - insets[index]) <= 1
               for index in (0, 1))


def exercise(owner, native, fixture, source, record):
    client = owner.client

    def query(endpoint):
        client.send(PREFIX + endpoint)
        reply = client.expect(PREFIX + endpoint + ".reply", timeout=2)
        record["observations"][endpoint] = reply
        return reply

    def progressed():
        frame = query("status")[1]
        def sample():
            state = query("status")
            return state if state[1] > frame else None
        return until(sample, "frames stopped after window change")

    def geometry(predicate, insets=None):
        def sample():
            metrics = query("window/metrics")
            try:
                snapshot = native.snapshot()
            except WindowUnavailable:
                record["observations"]["native"] = "window temporarily unavailable"
                return None
            validate_metrics(metrics)
            record["observations"]["native"] = snapshot
            if predicate(metrics, snapshot) and (insets is None or frame_matches(metrics, snapshot, insets)):
                return metrics, snapshot
            return None
        return until(sample, "native window geometry/state did not reach the requested state")

    def passed(name, details):
        record["checks"][name] = details
        print(f"PASS {name}: {details}", flush=True)

    def keyboard_check(name):
        # CoreGraphics virtual keycodes for ANSI F and Escape. Events target only
        # the owned PID; these are synthetic delivery checks, not physical input.
        events = []
        for code, key in ((3, "f"), (53, "Escape")):
            for pressed in (True, False):
                native.key(code, pressed)
                event = client.expect(PREFIX + "input/key", predicate=lambda args:
                                      args == ["down" if pressed else "up", key, 0])
                events.append(event)
        passed(name, dict(events=events, input_source="synthetic PID-targeted CoreGraphics"))

    record["stage"] = "window_discovery"
    until(native.discover, "native owned window did not appear")
    record["stage"] = "create_shader"
    client.send(PREFIX + "shader/create", "iss", (701, str(fixture), "wgsl"))
    client.expect(PREFIX + "shader/created", predicate=lambda args: args == [701])
    client.send(PREFIX + "uniform/f", "isf", (701, "amount", 0.75))
    metrics, snapshot = geometry(lambda m, n: not n["fullscreen"] and not n["minimized"])
    state = progressed()
    validate_health(state)
    insets = [snapshot["size"][index] - metrics[index] for index in (0, 1)]
    if any(value < 0 or value > 100 for value in insets):
        raise RuntimeError("unexpected native content/frame insets")
    passed("initial", dict(metrics=metrics, native=snapshot, frame_insets=insets, texture_bytes=state[13]))
    client.send(PREFIX + "window/input-enabled", "i", (1,))
    query("status")  # Command ordering ensures the input subscription is active.
    record["stage"] = "keyboard_windowed"
    keyboard_check(record["stage"])

    for index, size in enumerate(((640, 360), (800, 450), (720, 480), (960, 540)), 1):
        record["stage"] = f"resize_reload_{index}"
        previous_metrics, previous_state = metrics, state
        edit = fixture.with_suffix(".next")
        edit.write_text(source + f"\n// macOS interaction reload {index}\n", encoding="utf-8")
        edit.replace(fixture)
        client.send(PREFIX + "window/resize", "ii", size)
        client.expect(PREFIX + "shader/reloaded", predicate=lambda args: args == [701])
        metrics, snapshot = geometry(lambda m, n: m[:2] == list(size), insets)
        state = progressed()
        validate_resize(previous_metrics, previous_state, metrics, state)
        passed(record["stage"], dict(metrics=metrics, native=snapshot, frame=state[1], texture_bytes=state[13]))
    windowed_size = metrics[:2]
    for enabled in (1, 0):
        record["stage"] = f"fullscreen_{enabled}"
        previous_metrics, previous_state = metrics, state
        client.send(PREFIX + "window/fullscreen", "i", (enabled,))
        metrics, snapshot = geometry(lambda m, n: m[7] == enabled and n["fullscreen"] == bool(enabled)
                                     and (bool(enabled) or m[:2] == windowed_size), [0, 0] if enabled else insets)
        # AppKit's fullscreen transition is asynchronous; both OS state and size
        # must converge before measuring progress and GPU targets.
        state = progressed()
        validate_resize(previous_metrics, previous_state, metrics, state)
        passed(record["stage"], dict(metrics=metrics, native=snapshot, frame=state[1], texture_bytes=state[13]))
        if enabled:
            record["stage"] = "keyboard_fullscreen"
            keyboard_check(record["stage"])

    for enabled in (1, 0):
        record["stage"] = f"borderless_{enabled}"
        client.send(PREFIX + "window/borderless", "i", (enabled,))
        metrics, snapshot = geometry(lambda m, n: m[8] == enabled, [0, 0] if enabled else insets)
        validate_health(progressed())
        passed(record["stage"], dict(metrics=metrics, native=snapshot))

    for enabled in (0, 1):
        record["stage"] = f"cursor_visible_{enabled}"
        client.send(PREFIX + "window/cursor-visible", "i", (enabled,))
        metrics, _ = geometry(lambda m, n: m[10] == enabled)
        passed(record["stage"], dict(cursor_visible=metrics[10], scope="reported flag; physical observation pending"))

    record["stage"] = "input_resize"
    client.send(PREFIX + "window/input-enabled", "i", (1,))
    client.send(PREFIX + "window/resize", "ii", (820, 460))
    event = client.expect(PREFIX + "input/resize", predicate=lambda args: args[:2] == [820, 460])
    metrics, snapshot = geometry(lambda m, n: m[:2] == [820, 460], insets)
    if event != metrics[:5]:
        raise RuntimeError("input resize event disagrees with native window metrics")
    passed("input_resize", dict(event=event, native=snapshot))

    for enabled in (1, 0):
        record["stage"] = f"stats_{enabled}"
        client.send(PREFIX + "diagnostics/overlay", "i", (enabled,))
        until(lambda: query("status")[14] == enabled, "stats flag did not change")
        state = progressed()
        passed(record["stage"], dict(frame=state[1], show_stats=state[14], scope="state and frame progress"))

    record["stage"] = "minimize"
    native.minimize(True)
    focus_lost = client.expect(PREFIX + "input/focus", predicate=lambda args: args == [0])
    until(lambda: native.snapshot()["minimized"], "native window did not minimize")
    # Minimized windows need not present. OSC must answer while minimized, then
    # fresh frame progress is required after actual native restoration.
    state = query("status")
    validate_health(state)
    passed("minimize_status", dict(native=native.snapshot(), frame=state[1], focus_event=focus_lost))
    record["stage"] = "restore"
    native.minimize(False)
    until(lambda: not native.snapshot()["minimized"], "native window did not restore")
    # Restoring an AX-minimized window does not promise application activation.
    # Exercise the explicit front command before requiring a focus-gained event.
    client.send(PREFIX + "window/front")
    focus_gained = client.expect(PREFIX + "input/focus", predicate=lambda args: args == [1])
    metrics, snapshot = geometry(lambda m, n: m[:2] == [820, 460] and n["focused"], insets)
    state = progressed()
    validate_health(state)
    passed("restore_frames", dict(metrics=metrics, native=snapshot, frame=state[1],
                                  front_requested=True, focus_event=focus_gained))

    record["stage"] = "native_close"
    native.close_window()
    if owner.process.wait(timeout=5) != 0:
        raise RuntimeError("native window close did not produce exit 0")
    passed("native_close", dict(exit_code=0))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--adapter", default="Apple", help="unique hardware adapter name substring")
    parser.add_argument("--port", type=int, default=57244)
    args = parser.parse_args(argv)
    if sys.platform != "darwin":
        parser.error("requires an interactive native macOS desktop")
    if (not args.renderer.is_file() or args.output_dir.exists() or not args.adapter.strip()
            or not 1 <= args.port <= 65535):
        parser.error("renderer must exist, output directory must be new, adapter/port must be valid")
    args.output_dir.mkdir(parents=True)
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    record = dict(passed=False, state="running", stage="preflight", checks={}, observations={},
                  started_utc=datetime.now(timezone.utc).isoformat(), requested_adapter=args.adapter,
                  renderer_sha256=digest(args.renderer), host_architecture=platform.machine(),
                  macos=platform.mac_ver()[0], physical_input_tested=False,
                  visible_output_tested=False, sleep_wake_tested=False, mixed_dpi_moves_tested=False)
    owner = None
    try:
        arches = subprocess.check_output(["lipo", "-archs", str(args.renderer)], text=True).strip().split()
        translated = subprocess.run(["sysctl", "-in", "sysctl.proc_translated"],
                                    capture_output=True, text=True, timeout=5)
        if arches != [platform.machine()] or translated.stdout.strip() == "1":
            raise RuntimeError("requires a native renderer and host process, not Rosetta")
        record["binary_architectures"] = arches
        api = Accessibility()
        root = Path(__file__).resolve().parents[1]
        inputs = ["tools/check_macos_interactions.py", "tools/macos_window.py", "tools/osc_client.py",
                  "renderer/shaders/fullscreen.wgsl"]
        record["source_hashes"] = {name: digest(root / name) for name in inputs}
        source = (root / inputs[-1]).read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory(prefix="fixture-", dir=args.output_dir) as temporary:
            fixture = Path(temporary).resolve() / "interaction.wgsl"
            fixture.write_text(source, encoding="utf-8")
            record["stage"] = "renderer_startup"
            with ManagedRenderer(args.renderer, args.port, args.output_dir / "renderer.log",
                                 extra_args=["--backend", "metal", "--adapter", args.adapter]) as owner:
                if owner.ready_reply[2] != "Metal" or args.adapter.lower() not in owner.ready_reply[3].lower():
                    raise RuntimeError("renderer did not use the requested Metal adapter")
                record.update(version=owner.ready_reply[1], backend=owner.ready_reply[2], device=owner.ready_reply[3])
                native = OwnedWindow(owner.process, api)
                try:
                    exercise(owner, native, fixture, source, record)
                finally:
                    native.close()
        record.update(passed=True, stage="complete", state="complete")
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record.update(error=f"{type(error).__name__}: {error}",
                      state="blocked" if isinstance(error, PermissionError) else "failed")
        print(f"{record['state'].upper()} at {record['stage']}: {record['error']}", flush=True)
    finally:
        try:
            record["renderer_sha256_after"] = digest(args.renderer)
        except OSError as error:
            record["renderer_sha256_after"] = None
            record["renderer_hash_error"] = str(error)
        if record["renderer_sha256_after"] != record["renderer_sha256"]:
            record.update(passed=False, state="failed", error="renderer hash changed during checks")
        if owner and owner.process:
            record["owned_process_exit_code"] = owner.process.poll()
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        (args.output_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
