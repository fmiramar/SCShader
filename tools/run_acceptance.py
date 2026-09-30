#!/usr/bin/env python3
"""Run selected measured soak modes against one unchanged candidate.

Default: one hour of traffic. --final-eight-hour is an explicit, user-run final
release sign-off. Other modes are opt-in; short runs never qualify as long tests.
The output directory must be new. Interrupting stops only this runner's own child
process group; completed evidence remains and partial runs are never passes.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys

from process_tools import spawn_owned, stop_process_tree

MODES = ("traffic", "trivial", "reload", "feedback")


def digest(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def validate_result(result: dict, mode: str, seconds: float, renderer_hash: str, backend="auto", window_system="auto",
                    adapter=None, power_preference="high-performance"):
    if (result.get("passed") is not True or result.get("mode") != mode
            or result.get("renderer_sha256") != renderer_hash
            or result.get("requested_seconds") != seconds
            or not math.isfinite(result.get("elapsed_seconds", 0))
            or result.get("elapsed_seconds", 0) < seconds
            or result.get("rss_monitoring") is not True
            or result.get("requested_rate") != 1000
            or result.get("overlay_enabled") is not False
            or result.get("requested_backend", "auto") != backend
            or result.get("requested_window_system", "auto") != window_system
            or result.get("requested_adapter") != adapter
            or result.get("requested_power_preference", "high-performance") != power_preference
            or result.get("max_rss_mib") != 512
            or result.get("max_rss_growth_mib") != 128
            or result.get("memory_warmup_seconds") != 30):
        raise RuntimeError(f"{mode}: missing, mismatched, failed, or incomplete evidence")
    if adapter and adapter.strip().lower() not in result.get("device", "").lower():
        raise RuntimeError(f"{mode}: renderer did not report the requested adapter")
    if mode == "feedback":
        requested, confirmed = result.get("resize_actions"), result.get("resize_confirmations")
        if (type(requested) is not int or requested < 1 or type(confirmed) is not int
                or confirmed != requested):
            raise RuntimeError("feedback: missing or unconfirmed resize evidence")


def stop_child(process):
    stop_process_tree(process)


def arguments(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--backend", choices=["auto", "metal", "dx12", "vulkan", "gl"], default="auto")
    parser.add_argument("--window-system", choices=["auto", "x11", "wayland"], default="auto")
    parser.add_argument("--adapter", help="unique GPU name substring, e.g. NVIDIA or Intel")
    parser.add_argument("--power-preference", choices=["high-performance", "low-power", "none"], default="high-performance")
    duration = parser.add_mutually_exclusive_group()
    duration.add_argument("--minutes", type=float, help="duration per selected mode (default: 60)")
    duration.add_argument("--seconds", type=float, help="explicit short smoke override")
    duration.add_argument("--final-eight-hour", action="store_true", help="user-run final release sign-off; never enabled by default")
    parser.add_argument("--modes", choices=MODES, nargs="+", default=["traffic"], help="explicit scenarios; default: traffic only")
    parser.add_argument("--port", type=int, default=57166)
    args = parser.parse_args(argv)
    seconds = (args.seconds if args.seconds is not None else
               8 * 3600 if args.final_eight_hour else (args.minutes if args.minutes is not None else 60) * 60)
    if not math.isfinite(seconds) or seconds <= 0 or not 1 <= args.port <= 65535:
        parser.error("duration and port must be valid")
    if len(set(args.modes)) != len(args.modes):
        parser.error("modes must not contain duplicates")
    if args.adapter is not None and not args.adapter.strip():
        parser.error("adapter must not be empty")
    return args, seconds


def qualifications(modes, seconds):
    return dict(qualifies_as_one_hour_check="traffic" in modes and seconds >= 3600,
                qualifies_as_eight_hour_check="traffic" in modes and seconds >= 8 * 3600,
                qualifies_as_eight_hour_suite=set(modes) == set(MODES) and seconds >= 8 * 3600)


def main():
    args, seconds = arguments()
    if not args.renderer.is_file() or args.output_dir.exists():
        print("renderer must exist and output directory must be new", file=sys.stderr)
        return 2
    args.output_dir.mkdir(parents=True)
    renderer_hash = digest(args.renderer)
    record = dict(state="running", passed=False, **qualifications(args.modes, 0),
                  started_utc=datetime.now(timezone.utc).isoformat(),
                  seconds_per_mode=seconds, selected_modes=args.modes,
                  requested_backend=args.backend, requested_window_system=args.window_system,
                  requested_adapter=args.adapter, requested_power_preference=args.power_preference,
                  renderer_sha256=renderer_hash, modes={}, active_mode=None)

    def save():
        temporary = args.output_dir / "run.json.tmp"
        temporary.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        temporary.replace(args.output_dir / "run.json")

    save()
    try:
        for mode in args.modes:
            if digest(args.renderer) != renderer_hash:
                raise RuntimeError("candidate executable changed between runs")
            record["active_mode"] = mode
            save()
            print(f"START {mode}: {seconds:g} seconds; evidence {args.output_dir / mode}", flush=True)
            command = [sys.executable, str(Path(__file__).with_name("soak_uniforms.py")),
                       "--renderer", str(args.renderer.resolve()), "--port", str(args.port),
                       "--mode", mode, "--seconds", str(seconds), "--rate", "1000",
                       "--backend", args.backend, "--window-system", args.window_system,
                       "--power-preference", args.power_preference,
                       "--max-rss-mib", "512", "--max-rss-growth-mib", "128", "--memory-warmup", "30",
                       "--csv", str(args.output_dir / f"{mode}.csv"),
                       "--json", str(args.output_dir / f"{mode}.json"),
                       "--renderer-log", str(args.output_dir / f"{mode}-renderer.log")]
            if args.adapter:
                command.extend(["--adapter", args.adapter])
            with (args.output_dir / f"{mode}-console.log").open("x") as log:
                process = spawn_owned(command, stdout=log, stderr=subprocess.STDOUT)
                try:
                    code = process.wait(timeout=seconds + 60)
                finally:
                    stop_child(process)
            result_path = args.output_dir / f"{mode}.json"
            result = json.loads(result_path.read_text()) if result_path.is_file() else {}
            record["modes"][mode] = result
            save()
            if code:
                raise RuntimeError(f"{mode}: exit {code}; {result.get('reason', 'see console log')}")
            validate_result(result, mode, seconds, renderer_hash, args.backend, args.window_system,
                            args.adapter, args.power_preference)
            print(f"PASS {mode}: {result['elapsed_seconds']:.2f} measured seconds", flush=True)
        record.update(state="complete", passed=True, active_mode=None,
                      **qualifications(args.modes, seconds))
        return 0
    except KeyboardInterrupt:
        record.update(state="interrupted", reason="Interrupted; unfinished acceptance does not pass")
        return 130
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        record.update(state="failed", reason=str(error))
        print(f"FAIL {error}", flush=True)
        return 1
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        save()


if __name__ == "__main__":
    sys.exit(main())
