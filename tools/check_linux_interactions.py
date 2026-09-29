#!/usr/bin/env python3
"""Short Vulkan resize/reload/fullscreen checks on an owned Hyprland window.

Requires Hyprland's Lua dispatch API (0.55+). X11 here means Xwayland, not
native Xorg. Physical input, minimize, sleep and visual appearance are manual.
See docs/LINUX_INTERACTION_CHECKS.md for scope and upstream API references.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

from osc_client import ManagedRenderer

PREFIX = "/scshader/v1/"
TITLE = "SCShader verification"


def until(sample, description, timeout=5):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = sample()
        if result:
            return result
        # hyprctl requests are synchronous; do not flood the compositor.
        time.sleep(0.1)
    raise TimeoutError(description)


class Hyprctl:
    def call(self, *arguments, as_json=False):
        command = ["hyprctl", *(["-j"] if as_json else []), *arguments]
        result = subprocess.run(command, capture_output=True, text=True, timeout=2)
        output = result.stdout.strip()
        if result.returncode != 0:
            raise RuntimeError(f"hyprctl {arguments[0]}: {result.stderr.strip() or output}")
        if as_json:
            return json.loads(output)
        # Lua errors can be returned as text even when the CLI exits zero.
        if output != "ok":
            raise RuntimeError(f"hyprctl {arguments[0]} did not acknowledge success: {output}")

    def clients(self):
        clients = self.call("clients", as_json=True)
        if not isinstance(clients, list) or any(not isinstance(item, dict) for item in clients):
            raise RuntimeError("hyprctl clients did not return a window list")
        return clients


class OwnedWindow:
    def __init__(self, process, hyprctl, xwayland, observations):
        self.process, self.hyprctl = process, hyprctl
        self.xwayland, self.observations = xwayland, observations
        self.address = None
        until(self.snapshot, "could not find one mapped window owned by the renderer")

    def snapshot(self):
        if self.process.poll() is not None:
            raise RuntimeError("owned renderer exited before the window check completed")
        clients = self.hyprctl.clients()
        owned = [item for item in clients if item.get("pid") == self.process.pid]
        # Save only our process's window details, never other application titles.
        fields = ("address", "pid", "title", "mapped", "hidden", "floating",
                  "xwayland", "size", "at", "fullscreen", "fullscreenClient", "monitor")
        self.observations["owned_windows"] = [
            {key: item[key] for key in fields if key in item} for item in owned
        ]
        matches = [item for item in owned if item.get("title") == TITLE and item.get("mapped") is True]
        if len(matches) > 1:
            raise RuntimeError("ambiguous owned renderer windows")
        if not matches:
            if self.address is not None:
                raise RuntimeError("owned renderer window disappeared or changed identity")
            return None
        window = matches[0]
        address = window.get("address", "")
        if not isinstance(address, str) or not re.fullmatch(r"0x[0-9a-fA-F]+", address):
            raise RuntimeError("invalid owned window address")
        if self.address is not None and address != self.address:
            raise RuntimeError("owned renderer window address changed")
        if window.get("xwayland") is not self.xwayland:
            raise RuntimeError("renderer did not use the requested display protocol")
        self.address = address
        return window

    def float(self):
        self.snapshot()  # Recheck live process and window before any mutation.
        selector = json.dumps("address:" + self.address)
        self.hyprctl.call("eval", 'hl.dispatch(hl.dsp.window.float({action="set", window='
                          + selector + '}))')
        return until(lambda: self.snapshot().get("floating") is True,
                     "compositor did not float the owned window")

    def resize(self, width, height):
        self.snapshot()  # Recheck live process and window before any mutation.
        selector = json.dumps("address:" + self.address)
        self.hyprctl.call("eval", 'hl.dispatch(hl.dsp.window.resize({x='
                          + str(width) + ', y=' + str(height)
                          + ', relative=false, window=' + selector + '}))')
        return until(lambda: self.snapshot().get("size") == [width, height],
                     f"compositor did not resize the owned window to {width}x{height}")

    def size_matches(self, metrics, compositor_size=None):
        """Compare content sizes in their protocol's coordinate space.

        X11 metrics' logical size uses winit/Xft DPI, independently of Hyprland
        output scaling. Hyprland configures X11 in pixels, scaled by the monitor
        only with force_zero_scaling. Its clients JSON truncates logical sizes.
        See the upstream realToReportSize/xwaylandSizeToReal references in docs.
        """
        window = self.snapshot()
        size = window.get("size")
        if compositor_size is not None and size != compositor_size:
            return False
        if not self.xwayland:
            return metrics[:2] == size
        option = self.hyprctl.call("getoption", "xwayland:force_zero_scaling", as_json=True)
        self.observations["xwayland_scaling_option"] = option
        if isinstance(option, dict) and type(option.get("bool")) is bool:
            force_zero = option["bool"]
        elif (isinstance(option, dict) and "bool" not in option
              and type(option.get("int")) is int and option["int"] in (0, 1)):
            force_zero = bool(option["int"])
        else:
            raise RuntimeError("could not read Xwayland force_zero_scaling")
        scale = 1
        if force_zero:
            monitors = self.hyprctl.call("monitors", as_json=True)
            if not isinstance(monitors, list) or any(not isinstance(item, dict) for item in monitors):
                raise RuntimeError("hyprctl monitors did not return a monitor list")
            matches = [item for item in monitors if item.get("id") == window.get("monitor")]
            if len(matches) != 1:
                raise RuntimeError("could not identify the owned window's monitor")
            scale = matches[0].get("scale")
            if not isinstance(scale, (int, float)) or not math.isfinite(scale) or scale <= 0:
                raise RuntimeError("invalid compositor monitor scale")
        self.observations["xwayland_geometry"] = dict(
            monitor=window.get("monitor"), force_zero_scaling=force_zero,
            compositor_to_pixels=scale)
        if compositor_size is not None:
            # An integer compositor request is rounded when configured to X11.
            return metrics[2:4] == [math.floor(value * scale + 0.5) for value in compositor_size]
        # An OSC request starts in renderer logical units. Compare the measured
        # pixels converted back to Hyprland's truncated clients JSON geometry.
        return size == [math.floor(value / scale) for value in metrics[2:4]]


def check_texture_resize(previous_metrics, previous_status, metrics, status):
    """Catch a resized OS window whose GPU feedback/graph targets stayed stale.

    This fixture adds no images, buffers or overlay. A change in pixel area must
    therefore change the owned texture allocation in the same direction.
    Metrics alone cannot detect the ignored synchronous winit size result.
    """
    for sizes in (previous_metrics, metrics):
        if (len(sizes) != 12 or min(sizes[:4]) <= 0
                or not math.isfinite(sizes[4]) or sizes[4] <= 0):
            raise RuntimeError("invalid window dimensions or scale")
    for state in (previous_status, status):
        if len(state) < 18 or state[5:8] != [1, 0, 0] or state[14] != 0:
            raise RuntimeError("unexpected resources/overlay during resize check")
        if not math.isfinite(state[13]) or state[13] <= 0:
            raise RuntimeError("invalid owned texture allocation")
        if any(state[index] != 0 for index in (4, 9, 10, 11, 12, 15, 17)) or state[16] != 1:
            raise RuntimeError("unexpected scheduling, recovery, compilation or diagnostic state")
    area_change = metrics[2] * metrics[3] - previous_metrics[2] * previous_metrics[3]
    byte_change = status[13] - previous_status[13]
    if ((area_change != 0 and area_change * byte_change <= 0)
            or (area_change == 0 and byte_change != 0)):
        raise RuntimeError("GPU texture allocation did not follow the window's pixel area")


def exercise(owner, native, fixture, source, record, resize_driver="osc"):
    client = owner.client

    def query(endpoint):
        client.send(PREFIX + endpoint)
        reply = client.expect(PREFIX + endpoint + ".reply", timeout=2)
        record["observations"][endpoint] = reply
        return reply

    def metrics_where(predicate, description):
        def sample():
            metrics = query("window/metrics")
            return metrics if predicate(metrics) else None
        return until(sample, description)

    def progressed(frame):
        def sample():
            status = query("status")
            return status if status[1] > frame else None
        return until(sample, "frames stopped after window change")

    def passed(name, details):
        record["checks"][name] = details
        print(f"PASS {name}: {details}", flush=True)

    record["stage"] = "float_window"
    native.float()
    # Start borderless so compositor geometry and the content size can be
    # compared without guessing client-side decoration extents.
    record["stage"] = "create_shader"
    client.send(PREFIX + "shader/create", "iss", (701, str(fixture), "wgsl"))
    client.expect(PREFIX + "shader/created", predicate=lambda args: args == [701])
    client.send(PREFIX + "uniform/f", "isf", (701, "amount", 0.75))
    metrics = query("window/metrics")
    state = query("status")
    check_texture_resize(metrics, state, metrics, state)
    passed("initial", dict(metrics=metrics, texture_bytes=state[13]))

    # Establish that the owned window follows a compositor configure before
    # exercising the selected resize path. Floating in Hyprland's IPC does not
    # guarantee the Wayland configure's tiled flags have been cleared.
    record["stage"] = "compositor_resize_setup"
    previous_metrics, previous_state = metrics, state
    native.resize(900, 500)
    metrics = metrics_where(lambda args: native.size_matches(args, [900, 500]),
                            "renderer did not follow Hyprland's floating-window resize")
    state = progressed(query("status")[1])
    check_texture_resize(previous_metrics, previous_state, metrics, state)
    passed("compositor_resize_setup", dict(metrics=metrics, texture_bytes=state[13],
                                           compositor_size=[900, 500]))

    for index, (width, height) in enumerate(((640, 360), (800, 450), (720, 480), (960, 540)), 1):
        name = f"resize_reload_{index}"
        record["stage"] = name
        previous_metrics, previous_state = metrics, state
        edit = fixture.with_suffix(".next")
        edit.write_text(source + f"\n// Linux interaction reload {index}\n", encoding="utf-8")
        edit.replace(fixture)
        if resize_driver == "compositor":
            native.resize(width, height)
        else:
            client.send(PREFIX + "window/resize", "ii", (width, height))
        client.expect(PREFIX + "shader/reloaded", predicate=lambda args: args == [701])
        if resize_driver == "compositor":
            metrics = metrics_where(lambda args: native.size_matches(args, [width, height]),
                                    f"renderer did not follow compositor size {width}x{height}")
        else:
            metrics = metrics_where(lambda args: args[:2] == [width, height]
                                    and native.size_matches(args),
                                    f"renderer/compositor did not reach logical size {width}x{height}")
        # Wait for progress after size convergence, not just during the reload.
        state = progressed(query("status")[1])
        check_texture_resize(previous_metrics, previous_state, metrics, state)
        passed(name, dict(metrics=metrics, texture_bytes=state[13], frame=state[1],
                          resize_driver=resize_driver, requested_size=[width, height],
                          compositor_size=native.snapshot()["size"]))

    record["osc_resize_completed"] = resize_driver == "osc"

    for enabled in (1, 0):
        name = f"fullscreen_{enabled}"
        record["stage"] = name
        client.send(PREFIX + "window/fullscreen", "i", (enabled,))
        metrics = metrics_where(lambda args: args[7] == enabled, "fullscreen flag did not change")
        # Hyprland distinguishes maximize (1) from actual fullscreen (2).
        until(lambda: native.snapshot().get("fullscreen") == (2 if enabled else 0),
              "compositor fullscreen state did not change")
        state = progressed(query("status")[1])
        passed(name, dict(metrics=query("window/metrics"), frame=state[1]))

    for enabled in (0, 1):
        record["stage"] = f"borderless_{enabled}"
        client.send(PREFIX + "window/borderless", "i", (enabled,))
        metrics = metrics_where(lambda args: args[8] == enabled, "borderless flag did not change")
        progressed(query("status")[1])
        passed(record["stage"], dict(metrics=metrics, scope="requested flag and frame progress"))

    record["stage"] = "final_health"
    metrics, state = query("window/metrics"), query("status")
    check_texture_resize(metrics, state, metrics, state)
    record["stage"] = "window_close"
    client.send(PREFIX + "window/close")
    if owner.process.wait(timeout=5) != 0:
        raise RuntimeError("window close did not produce exit 0")
    passed("window_close", dict(exit_code=0))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--adapter", required=True, help="unique hardware adapter name substring")
    parser.add_argument("--window-system", choices=("wayland", "x11"), required=True)
    parser.add_argument("--resize-driver", choices=("osc", "compositor"), default="osc",
                        help="explicit resize path; compositor does not qualify OSC resizing")
    parser.add_argument("--port", type=int, default=57244)
    args = parser.parse_args(argv)
    if sys.platform != "linux" or not os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"):
        parser.error("requires an interactive native Linux Hyprland session")
    if (not args.renderer.is_file() or args.output_dir.exists()
            or not args.adapter.strip() or not 1 <= args.port <= 65535):
        parser.error("renderer must exist, output directory must be new, adapter/port must be valid")
    args.output_dir.mkdir(parents=True)
    record = dict(passed=False, checks={}, observations={}, stage="preflight",
                  requested_adapter=args.adapter, requested_display=args.window_system,
                  started_utc=datetime.now(timezone.utc).isoformat(),
                  renderer_sha256=hashlib.sha256(args.renderer.read_bytes()).hexdigest(),
                  resize_driver=args.resize_driver, osc_resize_completed=False,
                  physical_input_tested=False, minimize_tested=False, sleep_wake_tested=False,
                  mixed_dpi_moves_tested=False, native_xorg_tested=False)
    try:
        hyprctl = Hyprctl()
        version = hyprctl.call("version", as_json=True)
        record["hyprland_version"] = {key: version[key] for key in ("tag", "commit", "version") if key in version}
        hyprctl.clients()  # Fail before launching a renderer if IPC is unavailable.
        source = (Path(__file__).resolve().parents[1] / "renderer/shaders/fullscreen.wgsl").read_text(encoding="utf-8")
        with tempfile.TemporaryDirectory(prefix="fixture-", dir=args.output_dir) as temporary:
            fixture = Path(temporary).resolve() / "interaction.wgsl"
            fixture.write_text(source, encoding="utf-8")
            record["stage"] = "renderer_startup"
            with ManagedRenderer(args.renderer, args.port, args.output_dir / "renderer.log",
                                 extra_args=["--backend", "vulkan", "--adapter", args.adapter,
                                             "--window-system", args.window_system, "--borderless"]) as owner:
                if (owner.ready_reply[2] != "Vulkan"
                        or args.adapter.strip().lower() not in owner.ready_reply[3].lower()):
                    raise RuntimeError("renderer did not use the requested Vulkan adapter")
                record.update(version=owner.ready_reply[1], backend=owner.ready_reply[2],
                              device=owner.ready_reply[3])
                record["stage"] = "window_discovery"
                native = OwnedWindow(owner.process, hyprctl, args.window_system == "x11", record["observations"])
                record["actual_display"] = "Xwayland" if native.xwayland else "Wayland"
                exercise(owner, native, fixture, source, record, args.resize_driver)
        record.update(passed=True, stage="complete")
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        record["error"] = f"{type(error).__name__}: {error}"
        print(f"FAIL at {record['stage']}: {record['error']}", flush=True)
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        (args.output_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
