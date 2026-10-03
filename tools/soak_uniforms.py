#!/usr/bin/env python3
"""Measured SCShader soak tests; a short smoke never counts as an hour/eight-hour run."""
from __future__ import annotations

import argparse
import contextlib
import csv
from dataclasses import dataclass, field
from datetime import datetime, timezone
import json
import hashlib
import math
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time

from osc_client import ManagedRenderer, OscClient
from process_memory import windows_rss_kib

# Bound recovery bursts by elapsed traffic time, not a fixed packet count:
# 16 updates at 1 kHz, but enough for normal polling at the supported 50 kHz.
MAX_CATCH_UP_SECONDS = 0.016


# CPython 3.12 on Windows implements monotonic() with GetTickCount64:
# its ~15.6 ms resolution turns 1 kHz traffic into batches of 15/16 updates.
# perf_counter() is monotonic and uses QueryPerformanceCounter on Windows.
def soak_clock() -> float:
    return time.perf_counter()


def due_traffic_updates(next_update: float, now: float, rate: float) -> tuple[int, int, float]:
    """Return a bounded send count, stale ticks to skip, and next scheduled tick."""
    if next_update > now:
        return 0, 0, next_update
    due = math.floor((now - next_update) * rate) + 1
    max_burst = max(1, math.ceil(rate * MAX_CATCH_UP_SECONDS))
    skipped = max(0, due - max_burst)
    return due - skipped, skipped, next_update + skipped / rate


@dataclass
class Health:
    start: float
    timeout: float = 5.0
    last_pong: float = 0.0
    last_status: float = 0.0
    last_frame_progress: float = 0.0
    pongs: int = 0
    statuses: int = 0
    sent_sequence: int = 0
    pong_sequence: int = 0
    first_frame: int | None = None
    frame: int | None = None
    status: list | None = None

    def __post_init__(self):
        self.last_pong = self.last_status = self.last_frame_progress = self.start

    def reply(self, path: str, args: list, now: float):
        if path == "/scshader/v1/error":
            raise RuntimeError(f"unexpected renderer diagnostic: {args}")
        if path == "/scshader/v1/pong":
            if len(args) != 3 or not isinstance(args[0], int):
                raise RuntimeError("malformed pong reply")
            if self.pong_sequence < args[0] <= self.sent_sequence:
                self.pong_sequence = args[0]
                self.pongs += 1
                self.last_pong = now
        elif path == "/scshader/v1/status.reply":
            if len(args) < 9 or any(not isinstance(v, (int, float)) or not math.isfinite(v) for v in args):
                raise RuntimeError("malformed/non-finite status reply")
            frame = args[1]
            if frame < 0 or (self.frame is not None and frame < self.frame):
                raise RuntimeError("renderer frame counter reset or became invalid")
            if self.frame is None:
                self.first_frame = frame
            elif frame > self.frame:
                self.last_frame_progress = now
            self.frame = frame
            self.last_status = now
            self.status = args
            self.statuses += 1
            if len(args) >= 12 and (args[10] != 0 or args[11] != 0):
                raise RuntimeError(f"renderer dropped/rejected work: scheduled={args[10]}, OSC={args[11]}")
            if len(args) >= 18 and args[17] != 0:
                raise RuntimeError(f"renderer suppressed {args[17]} diagnostic(s)")

    def check(self, now: float):
        for label, stamp in [("pong", self.last_pong), ("status", self.last_status),
                             ("frame progress", self.last_frame_progress)]:
            if now - stamp > self.timeout:
                raise RuntimeError(f"no {label} for more than {self.timeout}s")

    def complete(self, now: float, duration: float):
        self.check(now)
        if now - self.start < duration:
            raise RuntimeError("soak ended before the requested duration")
        if not self.pongs or self.statuses < 2 or self.frame <= self.first_frame:
            raise RuntimeError("insufficient pong/status/frame-progress evidence")


@dataclass
class MemoryHealth:
    max_kib: float
    growth_kib: float
    warmup: float
    initial: int | None = None
    baseline: int | None = None
    peak: int = 0
    latest: int | None = None

    def sample(self, rss: int | None, elapsed: float):
        if rss is None or rss <= 0:
            raise RuntimeError("renderer PID disappeared or RSS sampling failed")
        self.initial = rss if self.initial is None else self.initial
        self.latest = rss
        self.peak = max(self.peak, rss)
        if rss > self.max_kib:
            raise RuntimeError(f"RSS exceeded absolute bound: {rss / 1024:.1f} MiB")
        if elapsed >= self.warmup:
            self.baseline = rss if self.baseline is None else self.baseline
            if rss - self.baseline > self.growth_kib:
                raise RuntimeError(f"RSS growth exceeded bound: {(rss - self.baseline) / 1024:.1f} MiB")


def feedback_resize_interval(duration: float) -> float:
    """Exercise resizing in short checks; retain the 30-second long-soak cadence."""
    return min(30.0, duration / 4.0)


@dataclass
class FeedbackResize:
    metrics: list
    requests: int = 0
    pending: tuple[int, int] | None = None
    requested_at: float = 0.0
    observations: list = field(default_factory=list)

    @staticmethod
    def validate_metrics(metrics):
        if (len(metrics) != 12
                or any(not isinstance(v, (int, float)) or not math.isfinite(v) for v in metrics[:11])
                or not isinstance(metrics[11], str)
                or min(metrics[:5]) <= 0
                or any(abs(metrics[i] * metrics[4] - metrics[i + 2]) > 1 for i in (0, 1))):
            raise RuntimeError("invalid logical/framebuffer resize metrics")

    def __post_init__(self):
        self.validate_metrics(self.metrics)

    def request(self, now):
        if self.pending is not None:
            raise RuntimeError("previous feedback resize is still pending")
        self.pending = (960, 540) if self.metrics[:2] == [1280, 720] else (1280, 720)
        self.requested_at = now
        self.requests += 1
        return self.pending

    def reply(self, metrics, now):
        self.validate_metrics(metrics)
        if self.pending is not None and metrics[:2] == list(self.pending):
            self.observations.append(dict(requested=list(self.pending), metrics=list(metrics),
                                          latency_seconds=now - self.requested_at))
            self.metrics = list(metrics)
            self.pending = None

    def check(self, now):
        if self.pending is not None and now - self.requested_at > 5:
            raise RuntimeError(f"feedback resize to {self.pending} was not confirmed by window metrics")

    def complete(self):
        if not self.observations or self.pending is not None or len(self.observations) != self.requests:
            raise RuntimeError("feedback run has missing or unconfirmed resizes")


def process_rss_kib(pid: int) -> int | None:
    if os.name == "nt":
        return windows_rss_kib(pid)
    command = ["ps", "-o", "rss=", "-p", str(pid)]
    try:
        result = subprocess.run(command, check=False, capture_output=True, text=True, timeout=3)
        return int(result.stdout.strip()) if result.returncode == 0 else None
    except (ValueError, OSError, subprocess.TimeoutExpired):
        return None


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", choices=["127.0.0.1"], default="127.0.0.1")
    parser.add_argument("--port", type=int, default=57140)
    parser.add_argument("--minutes", type=float, default=60.0)
    parser.add_argument("--seconds", type=float, help="short smoke override; reported with its real duration")
    parser.add_argument("--rate", type=float, default=1000.0, help="updates/sec in traffic mode (max 50000)")
    parser.add_argument("--mode", choices=["traffic", "trivial", "reload", "feedback"], default="traffic")
    parser.add_argument("--overlay", action="store_true", help="enable and verify the diagnostic overlay throughout the run")
    parser.add_argument("--renderer", type=Path, help="launch and own this renderer; omit to attach")
    parser.add_argument("--renderer-log", type=Path)
    parser.add_argument("--backend", choices=["auto", "metal", "dx12", "vulkan", "gl"], default="auto")
    parser.add_argument("--window-system", choices=["auto", "x11", "wayland"], default="auto")
    parser.add_argument("--adapter", help="unique GPU name substring")
    parser.add_argument("--power-preference", choices=["high-performance", "low-power", "none"], default="high-performance")
    parser.add_argument("--pid", type=int, help="attached renderer PID for RSS sampling")
    parser.add_argument("--csv", type=Path, help="stream metrics here, including on failure/interruption")
    parser.add_argument("--json", type=Path, help="write measured result; passed is false on failure/interruption")
    parser.add_argument("--max-rss-mib", type=float, default=512.0)
    parser.add_argument("--max-rss-growth-mib", type=float, default=128.0)
    parser.add_argument("--memory-warmup", type=float, default=30.0)
    parser.add_argument("--quit", action="store_true", help="quit attached renderer after success; owned children always stop")
    args = parser.parse_args()
    duration = args.seconds if args.seconds is not None else args.minutes * 60.0
    if (not all(math.isfinite(v) for v in [duration, args.rate, args.max_rss_mib, args.max_rss_growth_mib, args.memory_warmup])
            or duration <= 0 or not 0 < args.rate <= 50000 or not 1 <= args.port <= 65535
            or args.max_rss_mib <= 0 or args.max_rss_growth_mib <= 0 or args.memory_warmup < 0):
        parser.error("duration, rate, port, and memory bounds must be finite and valid")
    if args.renderer and args.pid:
        parser.error("--pid is for attached renderers; --renderer captures its own child's PID")
    if not args.renderer and (args.backend != "auto" or args.window_system != "auto"
                              or args.adapter is not None or args.power_preference != "high-performance"):
        parser.error("backend/display/GPU selection requires an owned --renderer")
    if args.adapter is not None and not args.adapter.strip():
        parser.error("adapter must not be empty")
    if args.pid is not None and args.pid <= 0:
        parser.error("--pid must be positive")
    paths = [p.resolve() for p in [args.csv, args.json, args.renderer_log] if p]
    if len(paths) != len(set(paths)) or (args.renderer and args.renderer.resolve() in paths):
        parser.error("output paths must be distinct and cannot overwrite the renderer")
    if args.renderer_log and not args.renderer:
        parser.error("--renderer-log requires an owned --renderer")
    for path in paths:
        if path.exists():
            parser.error(f"output already exists; choose a fresh evidence path: {path}")
        path.parent.mkdir(parents=True, exist_ok=True)
    return args, duration


def run(args, duration, result):
    with contextlib.ExitStack() as stack:
        tracing = bool(args.renderer) and os.environ.get("SCSHADER_TRACE_TIMING") == "1"
        result["timing_trace_enabled"] = tracing
        power_trace = None
        if tracing and os.name == "nt" and args.csv:
            from windows_power_trace import WindowsPowerTrace
            power_trace = stack.enter_context(WindowsPowerTrace(args.csv.with_suffix(".power.jsonl")))
        owned = None
        if args.renderer:
            with args.renderer.open("rb") as binary:
                result["renderer_sha256"] = hashlib.file_digest(binary, "sha256").hexdigest()
            extra_args = []
            if args.adapter:
                extra_args.extend(["--adapter", args.adapter])
            if args.power_preference != "high-performance":
                extra_args.extend(["--power-preference", args.power_preference])
            if args.backend != "auto":
                extra_args.extend(["--backend", args.backend])
            if args.window_system != "auto":
                extra_args.extend(["--window-system", args.window_system])
            owned = stack.enter_context(ManagedRenderer(args.renderer, args.port, args.renderer_log, extra_args))
            client, pid = owned.client, owned.process.pid
            ready = owned.ready_reply
        else:
            client = stack.enter_context(contextlib.closing(OscClient(args.port)))
            ready = client.hello()
            pid = args.pid
        result.update(renderer_version=ready[1], backend=ready[2], device=ready[3],
                      platform=platform.system(), architecture=platform.machine())
        expected_backend = {"metal": "Metal", "dx12": "Dx12", "vulkan": "Vulkan", "gl": "Gl"}.get(args.backend)
        if expected_backend and ready[2] != expected_backend:
            raise RuntimeError(f"requested {args.backend} but renderer reported {ready[2]}")
        if args.adapter and args.adapter.strip().lower() not in ready[3].lower():
            raise RuntimeError(f"requested adapter {args.adapter!r} but renderer reported {ready[3]!r}")
        print(f"renderer={ready[1]} backend={ready[2]} pid={pid}", flush=True)
        # The one deliberate malformed packet must produce precisely E_PROTOCOL.
        client.send_packet(b"/scshader/v1/uniform/f\0")
        error = client.expect("/scshader/v1/error", timeout=2)
        if len(error) != 5 or error[3] != "E_PROTOCOL":
            raise RuntimeError(f"wrong malformed-packet diagnostic: {error}")
        result["expected_protocol_errors"] = 1
        if args.overlay:
            client.send("/scshader/v1/diagnostics/overlay", "i", (1,))
            client.send("/scshader/v1/status")
            status = client.expect("/scshader/v1/status.reply")
            if len(status) < 15 or status[14] != 1:
                raise RuntimeError("renderer did not enable the requested overlay")

        reload_path = None
        original = None
        resource = 1
        if args.mode == "reload":
            temporary = Path(stack.enter_context(tempfile.TemporaryDirectory(prefix="scshader-soak-")))
            reload_path = temporary / "watched.wgsl"
            original = (Path(__file__).resolve().parents[1] / "renderer/shaders/fullscreen.wgsl").read_text()
            reload_path.write_text(original)
            resource = 10101
            client.send("/scshader/v1/shader/create", "iss", (resource, str(reload_path), "wgsl"))
            client.expect("/scshader/v1/shader/created", predicate=lambda values: values == [resource])
        resize = None
        if args.mode == "feedback":
            client.send("/scshader/v1/shader/feedback", "isf", (resource, "previous", 0.92))
            client.send("/scshader/v1/window/metrics")
            resize = FeedbackResize(client.expect("/scshader/v1/window/metrics.reply"))

        writer = None
        output = stack.enter_context(args.csv.open("x", newline="", encoding="utf-8")) if args.csv else None
        if output:
            writer = csv.writer(output)
            writer.writerow(["elapsed_seconds", "updates", "pongs", "frame_index", "fps",
                             "scheduled_commands", "scheduled_payload_bytes", "rejected_scheduled",
                             "dropped_continuous", "reloads", "rss_kib"])
            output.flush()
        start = soak_clock()
        result["sender_clock"] = dict(name="perf_counter", **vars(time.get_clock_info("perf_counter")))
        result["python_version"] = platform.python_version()
        result.update(started_utc=datetime.now(timezone.utc).isoformat(), pid=pid, elapsed_seconds=0.0)
        health = Health(start)
        memory = MemoryHealth(args.max_rss_mib * 1024, args.max_rss_growth_mib * 1024, args.memory_warmup)
        next_update = next_ping = next_sample = start
        resize_interval = feedback_resize_interval(duration)
        next_action = start + (1.0 if args.mode == "reload" else resize_interval)
        next_resize_probe = start
        reload_pending = None
        actions = updates = reloads = skipped_updates = max_burst = 0
        max_sender_lag_ms = 0.0
        rss = None

        def sample(now):
            nonlocal rss
            if power_trace:
                power_trace.drain()
            elapsed = now - start
            if pid is not None:
                rss = process_rss_kib(pid)
                memory.sample(rss, elapsed)
            status = health.status or [0] * 12
            row = [elapsed, updates, health.pongs, health.frame, status[0], status[4],
                   status[9] if len(status) > 9 else None, status[10] if len(status) > 10 else None,
                   status[11] if len(status) > 11 else None, reloads, rss]
            if writer:
                writer.writerow(row)
                output.flush()
            print(f"{elapsed:8.1f}s updates={updates} pongs={health.pongs} frames={health.frame} "
                  f"reloads={reloads} rss_kib={rss}", flush=True)

        try:
            while soak_clock() - start < duration:
                now = soak_clock()
                if owned and owned.process.poll() is not None:
                    raise RuntimeError(f"renderer exited with {owned.process.returncode}")
                if args.mode == "traffic":
                    max_sender_lag_ms = max(max_sender_lag_ms, (now - next_update) * 1000)
                    if now - next_update > 0.25:
                        raise RuntimeError("traffic generator stalled; requested rate was not maintained")
                    send_count, skipped, next_update = due_traffic_updates(next_update, now, args.rate)
                    skipped_updates += skipped
                    max_burst = max(max_burst, send_count)
                    for _ in range(send_count):
                        client.send("/scshader/v1/uniform/f", "isf",
                                    (resource, "amount", 0.5 + 0.5 * math.sin((next_update - start) * 2)))
                        updates += 1
                        next_update += 1.0 / args.rate
                if now >= next_ping:
                    health.sent_sequence += 1
                    client.send("/scshader/v1/ping", "id", (health.sent_sequence, now - start))
                    client.send("/scshader/v1/status")
                    next_ping = now + 1.0
                for _ in range(256):
                    reply = client.receive()
                    if reply is None:
                        break
                    health.reply(*reply, now)
                    if resize is not None and reply[0] == "/scshader/v1/window/metrics.reply":
                        resize.reply(reply[1], now)
                    if args.overlay and reply[0] == "/scshader/v1/status.reply" and (len(reply[1]) < 15 or reply[1][14] != 1):
                        raise RuntimeError("diagnostic overlay state was lost during the soak")
                    if reply[0] == "/scshader/v1/shader/reloaded" and reply[1] == [resource]:
                        if reload_pending is None:
                            raise RuntimeError("unexpected duplicate reload acknowledgement")
                        reloads += 1
                        reload_pending = None
                health.check(now)
                if reload_pending is not None and now - reload_pending > 5:
                    raise RuntimeError("watched shader reload did not complete")
                if resize is not None:
                    resize.check(now)
                    if resize.pending is not None and now >= next_resize_probe:
                        client.send("/scshader/v1/window/metrics")
                        next_resize_probe = now + 0.1
                if now >= next_action:
                    if args.mode == "reload" and reload_pending is None:
                        actions += 1
                        reload_path.write_text(original + f"\n// soak edit {actions}\n")
                        reload_pending = now
                        next_action = now + 1.0
                    elif resize is not None and resize.pending is None:
                        client.send("/scshader/v1/window/resize", "ii", resize.request(now))
                        next_action = now + resize_interval
                if now >= next_sample:
                    sample(now)
                    next_sample = now + 10.0
                time.sleep(0.001)
            now = soak_clock()
            health.complete(now, duration)
            if args.mode == "reload" and duration >= 2 and not reloads:
                raise RuntimeError("no completed hot reloads")
            if resize is not None:
                resize.complete()
            if args.mode == "traffic" and updates < duration * args.rate * 0.99:
                raise RuntimeError("traffic rate fell below 99% of requested rate")
            sample(now)
            if args.quit and not owned:
                client.send("/scshader/v1/quit")
        finally:
            result.update(elapsed_seconds=soak_clock() - start, updates=updates,
                          skipped_updates=skipped_updates, max_update_burst=max_burst, pongs=health.pongs,
                          max_sender_lag_ms=max_sender_lag_ms,
                          status_samples=health.statuses, first_frame=health.first_frame, last_frame=health.frame,
                          reloads=reloads, resize_actions=resize.requests if resize else 0,
                          resize_confirmations=len(resize.observations) if resize else 0,
                          resize_observations=resize.observations if resize else [],
                          resize_interval_seconds=resize_interval if resize else None,
                          rss_initial_kib=memory.initial, rss_peak_kib=memory.peak or None,
                          rss_final_kib=memory.latest, rss_baseline_kib=memory.baseline,
                          rss_monitoring=pid is not None)


def main() -> int:
    args, duration = arguments()
    result = dict(passed=False, mode=args.mode, requested_seconds=duration, requested_rate=args.rate,
                  overlay_enabled=args.overlay,
                  requested_backend=args.backend, requested_window_system=args.window_system,
                  requested_adapter=args.adapter, requested_power_preference=args.power_preference,
                  elapsed_seconds=0.0, expected_protocol_errors=0, max_rss_mib=args.max_rss_mib,
                  max_rss_growth_mib=args.max_rss_growth_mib, memory_warmup_seconds=args.memory_warmup)
    code = 1
    try:
        run(args, duration, result)
        result["passed"] = True
        code = 0
    except KeyboardInterrupt:
        result["reason"] = "interrupted; not a completed soak"
        code = 130
    except (OSError, RuntimeError, ValueError, TimeoutError) as error:
        result["reason"] = str(error)
    finally:
        result["finished_utc"] = datetime.now(timezone.utc).isoformat()
        if args.json:
            with args.json.open("x", encoding="utf-8") as output:
                output.write(json.dumps(result, indent=2) + "\n")
        print(("PASS" if result["passed"] else "FAIL") + ": " + json.dumps(result), flush=True)
    return code


if __name__ == "__main__":
    sys.exit(main())
