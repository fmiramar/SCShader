#!/usr/bin/env python3
"""Short, bounded resource-churn and OSC-flood regressions; not soak qualification."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import platform
import struct
import sys
import tempfile
import time

from osc_client import ManagedRenderer, osc_bundle, osc_message
from soak_uniforms import process_rss_kib

MODES = ("resources", "protocol_flood", "command_flood", "continuous")
PREFIX = "/scshader/v1/"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def check_resources(state, counts, texture_bytes=None):
    require(len(state) >= 17, "renderer lacks resource/diagnostic status fields")
    require(state[5:8] == list(counts), f"unexpected live resource counts: {state[5:8]}")
    if texture_bytes is not None:
        require(state[13] == texture_bytes, f"owned texture bytes did not return to baseline: {state[13]}")
    require(state[4] == 0 and state[9] == 0, "unexpected scheduled work after cleanup")
    require(state[15] == 0, "unexpected GPU device recovery")


def check_diagnostic_bound(count, elapsed):
    # Contract: burst 20 + 20/second, with one token of timing tolerance.
    require(count <= 21 + math.ceil(elapsed * 20),
            f"diagnostics are not bounded: {count} replies in {elapsed:.3f}s")


class Probe:
    def __init__(self, owner):
        self.owner, self.client = owner, owner.client
        self.errors = Counter()
        self.statuses = self.pongs = 0
        self.frame = None
        self.sequence = 0

    def receive(self, timeout=0):
        reply = self.client.receive(timeout)
        if reply:
            address, args = reply
            if address == PREFIX + "error":
                require(len(args) >= 5, "malformed diagnostic")
                self.errors[args[3]] += 1
            elif address == PREFIX + "status.reply":
                self.statuses += 1
                if self.frame is not None:
                    require(args[1] >= self.frame, "frame counter reset")
                self.frame = args[1]
            elif address == PREFIX + "pong":
                require(0 < args[0] <= self.sequence, "unsolicited pong")
                self.pongs += 1
        return reply

    def barrier(self, timeout=5):
        self.sequence += 1
        self.client.send(PREFIX + "ping", "id", (self.sequence, 0.0))
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            reply = self.receive(0.01)
            if reply and reply[0] == PREFIX + "pong" and reply[1][0] == self.sequence:
                return
        raise TimeoutError("owned renderer did not acknowledge work within deadline")

    def status(self):
        self.client.send(PREFIX + "status")
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            reply = self.receive(0.01)
            if reply and reply[0] == PREFIX + "status.reply":
                return reply[1]
        raise TimeoutError("no status reply")

    def frames(self, after):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            state = self.status()
            if state[1] > after:
                return state
            time.sleep(0.01)
        raise TimeoutError("renderer stopped drawing")

    def memory(self):
        rss = process_rss_kib(self.owner.process.pid)
        require(rss is not None and 0 < rss <= 512 * 1024, f"invalid/excessive RSS: {rss}")
        return rss


def resources(probe, args, result):
    project = Path(__file__).resolve().parents[1]
    state = probe.status()
    check_resources(state, (0, 0, 0))
    baseline_bytes, first_frame = state[13], state[1]
    samples = []
    result.update(completed_cycles=0, warmup_cycles=10, baseline_texture_bytes=baseline_bytes)
    with tempfile.TemporaryDirectory(prefix="scshader-churn-") as temporary:
        image = Path(temporary) / "source with spaces.ppm"
        image.write_bytes(b"P6\n32 32\n255\n" + bytes([64, 128, 192]) * (32 * 32))
        shader = str(project / "renderer/shaders/fullscreen.wgsl")
        for cycle in range(args.cycles):
            # Reuse IDs, bind all resource types and render before freeing them.
            create = [osc_message(PREFIX + "shader/create", "iss", (id, shader, "wgsl")) for id in (101, 102)]
            create += [osc_message(PREFIX + "texture/create", "is", (201, str(image))),
                       osc_message(PREFIX + "buffer/create", "ii", (301, 512)),
                       osc_message(PREFIX + "buffer/write", "iib", (301, 0, struct.pack(">512f", *([0.25] * 512)))),
                       osc_message(PREFIX + "shader/texture", "isi", (102, "source", 201)),
                       osc_message(PREFIX + "shader/buffer", "isi", (102, "spectrum", 301)),
                       osc_message(PREFIX + "graph/set", "ii", (101, 102))]
            probe.client.send_packet(osc_bundle(create))
            probe.barrier()
            live = probe.frames(state[1])
            check_resources(live, (2, 1, 1))
            require(live[13] > baseline_bytes, "resource allocation did not change owned texture bytes")
            # Free graph members while selected; later clear must remain safe.
            release = [osc_message(PREFIX + "shader/free", "i", (id,)) for id in (101, 102)]
            release += [osc_message(PREFIX + "texture/free", "i", (201,)),
                        osc_message(PREFIX + "buffer/free", "i", (301,)),
                        osc_message(PREFIX + "graph/set")]
            probe.client.send_packet(osc_bundle(release))
            probe.barrier()
            state = probe.frames(live[1])
            check_resources(state, (0, 0, 0), baseline_bytes)
            require(not probe.errors, f"resource churn diagnostics: {probe.errors}")
            rss = probe.memory()
            samples.append(rss)
            result.update(completed_cycles=cycle + 1, peak_rss_kib=max(samples), final_rss_kib=rss)
            if cycle >= 10:
                result["baseline_rss_kib"] = samples[10]
                require(rss - samples[10] <= 128 * 1024, "post-warmup RSS growth exceeds 128 MiB")
            if (cycle + 1) % 20 == 0:
                print(f"resources: {cycle + 1}/{args.cycles} cycles", flush=True)
    require(state[1] > first_frame, "no frame progress during churn")
    result.update(first_frame=first_frame, final_frame=state[1], final_texture_bytes=state[13])


def flood(probe, args, result, mode):
    first = probe.status()
    baseline_rss = probe.memory()
    if mode == "protocol_flood":
        packet = b"bad"  # rejected before the recursive OSC decoder
    else:
        packet = osc_message(PREFIX + "uniform/f", "isf", (999999 if mode == "command_flood" else 1, "amount", 0.25))
    commands_per_packet = 128 if mode == "continuous" else 1
    if mode == "continuous":
        # 128 updates fit below the macOS UDP datagram ceiling. Atomic bundles
        # stress execution/admission without relying on a very fast Python sender.
        packet = osc_bundle([packet] * commands_per_packet)
    rate = 2000
    start = time.monotonic()
    next_health = start
    sent = 0
    last_health = start
    last_frame = first[1]
    max_health_gap = 0.0
    live_frame_progress = 0
    pongs_before = probe.pongs
    statuses_before = probe.statuses
    while time.monotonic() - start < args.seconds:
        now = time.monotonic()
        target = min(int((now - start) * rate), math.ceil(args.seconds * rate))
        # Put probes ahead of the catch-up burst. Coarse host timers can group
        # large datagrams; trailing probes can be lost in UDP admission even
        # while frames continue. Keep the offered load and health limits intact.
        if now >= next_health:
            probe.sequence += 1
            probe.client.send(PREFIX + "ping", "id", (probe.sequence, 0.0))
            probe.client.send(PREFIX + "status")
            next_health = now + 0.2
        for _ in range(min(64, max(0, target - sent))):
            probe.client.send_packet(packet)
            sent += 1
        for _ in range(256):
            reply = probe.receive()
            if reply is None:
                break
            if reply[0] == PREFIX + "status.reply" and reply[1][1] > last_frame:
                now = time.monotonic()
                max_health_gap = max(max_health_gap, now - last_health)
                last_health = now
                last_frame = reply[1][1]
                live_frame_progress += 1
        time.sleep(0.001)
    max_health_gap = max(max_health_gap, time.monotonic() - last_health)
    live_pongs = probe.pongs - pongs_before
    live_statuses = probe.statuses - statuses_before
    probe.barrier()
    last = probe.status()
    elapsed = time.monotonic() - start
    rss = probe.memory()
    result.update(sent_packets=sent, elapsed_flood_seconds=elapsed, errors=dict(probe.errors),
                  commands_per_packet=commands_per_packet, offered_commands=sent * commands_per_packet,
                  pongs=probe.pongs, statuses=probe.statuses, first_frame=first[1], final_frame=last[1],
                  baseline_rss_kib=baseline_rss, final_rss_kib=rss,
                  live_pongs=live_pongs, live_statuses=live_statuses,
                  live_frame_progress_samples=live_frame_progress, max_frame_health_gap_seconds=max_health_gap,
                  dropped_continuous=last[11], suppressed_diagnostics=last[17] if len(last) > 17 else None)
    result["incoming_queue_saturated"] = last[11] > 0
    require(sent >= args.seconds * rate * 0.8, "sender did not sustain the test's minimum offered load")
    require(probe.pongs >= 2 and probe.statuses >= 2 and last[1] > first[1], "insufficient live responsiveness")
    require(live_pongs >= 2 and live_statuses >= 2 and live_frame_progress >= 2 and max_health_gap <= 1,
            "renderer was not responsive while flood traffic was active")
    require(rss - baseline_rss <= 128 * 1024, "flood RSS growth exceeds 128 MiB")
    check_resources(last, (0, 0, 0), first[13])
    expected = {"E_PROTOCOL"} if mode == "protocol_flood" else {"E_BAD_ARGUMENT", "E_RESOURCE_NOT_FOUND", "E_QUEUE_FULL"}
    if mode == "continuous":
        expected = {"E_QUEUE_FULL"}
    require(set(probe.errors) <= expected, f"unexpected flood diagnostics: {probe.errors}")
    if mode != "continuous":
        check_diagnostic_bound(sum(probe.errors.values()), elapsed)
        require(result["suppressed_diagnostics"] is not None and result["suppressed_diagnostics"] > 0,
                "missing suppression accounting under diagnostic overload")
    elif args.require_saturation:
        require(last[11] > 0, "valid-update load did not saturate the incoming queue")
    # Ordinary control/get must still work once traffic stops.
    probe.client.send(PREFIX + "uniform/f", "isf", (1, "amount", 0.75))
    probe.barrier()
    probe.client.send(PREFIX + "uniform/get", "isi", (1, "amount", 987))
    value = probe.client.expect(PREFIX + "uniform/value", predicate=lambda values: values[2] == 987)
    require(struct.unpack(">f", value[4])[0] == 0.75, "control updates did not recover after flood")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--modes", choices=MODES, nargs="+", default=list(MODES))
    parser.add_argument("--cycles", type=int, default=80)
    parser.add_argument("--seconds", type=float, default=2, help="flood duration per mode, 1-10 seconds")
    parser.add_argument("--port", type=int, default=57167)
    parser.add_argument("--require-saturation", action="store_true", help="fail unless continuous traffic demonstrably fills the incoming queue")
    args = parser.parse_args()
    if not 12 <= args.cycles <= 200 or not 1 <= args.seconds <= 10 or not 1 <= args.port <= 65535:
        parser.error("cycles must be 12-200, flood seconds 1-10, port 1-65535")
    if not args.renderer.is_file() or args.output_dir.exists() or len(set(args.modes)) != len(args.modes):
        parser.error("renderer must exist; output directory and modes must be unique")
    if args.require_saturation and "continuous" not in args.modes:
        parser.error("--require-saturation requires continuous mode")
    args.output_dir.mkdir(parents=True)
    with args.renderer.open("rb") as source:
        sha = hashlib.file_digest(source, "sha256").hexdigest()
    record = dict(passed=False, state="running", renderer_sha256=sha, modes={},
                  platform=platform.system(), architecture=platform.machine(),
                  started_utc=datetime.now(timezone.utc).isoformat(), qualifies_as_one_hour_check=False,
                  qualifies_as_eight_hour_check=False)

    def save():
        temporary = args.output_dir / "run.json.tmp"
        temporary.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        temporary.replace(args.output_dir / "run.json")

    save()
    try:
        for mode in args.modes:
            result = dict(passed=False)
            record["modes"][mode] = result
            started = time.monotonic()
            try:
                with args.renderer.open("rb") as source:
                    require(hashlib.file_digest(source, "sha256").hexdigest() == sha, "renderer changed during tests")
                with ManagedRenderer(args.renderer, args.port, args.output_dir / f"{mode}-renderer.log") as owner:
                    result.update(renderer_version=owner.ready_reply[1], backend=owner.ready_reply[2], device=owner.ready_reply[3])
                    probe = Probe(owner)
                    if mode == "resources":
                        resources(probe, args, result)
                    else:
                        flood(probe, args, result, mode)
                log_path = args.output_dir / f"{mode}-renderer.log"
                result["renderer_log_bytes"] = log_path.stat().st_size
                if mode in ("protocol_flood", "command_flood"):
                    lines = log_path.read_text(encoding="utf-8", errors="replace").splitlines()
                    logged = sum(line.startswith("SCShader ") and " error E_" in line for line in lines)
                    result["logged_diagnostics"] = logged
                    check_diagnostic_bound(logged, result["elapsed_flood_seconds"])
                result["passed"] = True
            except (OSError, ValueError, RuntimeError, TimeoutError) as error:
                result["reason"] = str(error)
            result["elapsed_seconds"] = time.monotonic() - started
            print(f"{'PASS' if result['passed'] else 'FAIL'} {mode}: {result.get('reason', 'bounded checks completed')}", flush=True)
            save()
        record.update(state="complete", passed=all(result["passed"] for result in record["modes"].values()))
    except KeyboardInterrupt:
        record.update(state="interrupted")
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save()
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
