#!/usr/bin/env python3
"""Run installed SCShader class, SCDoc, and native-window checks with timeouts.

Live checks open short-lived renderer windows and some boot a private audio server.
Install Classes/HelpSource and build the debug renderer before running this tool.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

from process_tools import spawn_owned, stop_process_tree


def find_sclang():
    located = shutil.which("sclang")
    if located:
        return located
    if sys.platform == "darwin":
        return "/Applications/SuperCollider.app/Contents/MacOS/sclang"
    if sys.platform == "win32":
        candidates = []
        for variable in ("ProgramW6432", "ProgramFiles"):
            root = os.environ.get(variable)
            if root:
                candidates.extend(Path(root).glob("SuperCollider*/sclang.exe"))
        if candidates:
            def version(path):
                return tuple(int(part) for part in re.findall(r"\d+", path.parent.name))
            return str(max(candidates, key=version))
    return None


def isolated_script(script: Path, state: Path, audio_output_device=None, audio_input_channels=None) -> str:
    """Keep test archives/help output separate from the user's running SC session."""
    quote = lambda path: json.dumps(path.resolve().as_posix(), ensure_ascii=False)
    audio = ""
    if audio_output_device is not None:
        audio += ("ServerOptions.defaultValues[\\outDevice] = "
                  + json.dumps(audio_output_device, ensure_ascii=False) + ";\n")
    if audio_input_channels is not None:
        if audio_input_channels < 0:
            raise ValueError("audio input channels must be non-negative")
        audio += f"ServerOptions.defaultValues[\\numInputBusChannels] = {audio_input_channels};\n"
    return ("(\nArchive.global = Archive.new;\n"
            f"Archive.archiveDir = {quote(state)};\n"
            f"SCDoc.helpTargetDir = {quote(state / 'Help')};\n"
            f"SynthDef.synthDefDir = {quote(state / 'synthdefs')};\n"
            "SynthDef.synthDefDir.mkdir;\n"
            f"{audio}{quote(script)}.load;\n)\n")


def add_audio_arguments(parser):
    parser.add_argument("--audio-output-device", help="explicit SC output device for owned test servers only")
    parser.add_argument("--audio-input-channels", type=int, help="input channel count for owned test servers (0 disables hardware input)")


def check_passed(check, text, returncode, timed_out=False):
    ok = (not timed_out and returncode == 0 and re.search(r"SCSHADER_\w+_OK", text)
          and not re.search(r"ERROR:|FAILURE IN SERVER", text))
    if check == "verify_scdoc" and re.search(r"(?:WARNING|ERROR|SCDoc:.*(?:warning|error))", text, re.IGNORECASE):
        ok = False
    return bool(ok)


def main() -> int:
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checks", nargs="*", help="test stems (default: all, except timing benchmark)")
    parser.add_argument("--sclang", default=find_sclang(), help="sclang executable, required if absent from PATH")
    parser.add_argument("--renderer", type=Path, help="test a particular native/packaged renderer")
    parser.add_argument("--sclang-config", type=Path, help="explicit class-library configuration for an isolated check")
    parser.add_argument("--working-directory", type=Path, help="child launch directory (use the SC installation to test IDE DLL discovery)")
    parser.add_argument("--output-dir", type=Path, help="fresh evidence directory; default: build/test-logs")
    parser.add_argument("--timeout", type=float, default=60)
    add_audio_arguments(parser)
    args = parser.parse_args()
    if not args.sclang:
        parser.error("sclang not found; pass --sclang with its full path")
    if args.timeout <= 0:
        parser.error("timeout must be positive")
    if args.audio_input_channels is not None and args.audio_input_channels < 0:
        parser.error("audio input channels must be non-negative")
    if args.sclang_config and not args.sclang_config.is_file():
        parser.error("sclang configuration does not exist")
    if args.working_directory and not args.working_directory.is_dir():
        parser.error("working directory does not exist")
    if args.output_dir and args.output_dir.exists():
        parser.error("output directory must be new")
    environment = os.environ.copy()
    if args.renderer:
        if not args.renderer.is_file():
            parser.error("renderer does not exist")
        environment["SCSHADER_RENDERER"] = str(args.renderer.resolve())
    tests = project / "tests/sc"
    checks = args.checks or [path.stem for path in sorted(tests.glob("*.scd")) if path.stem != "timing_benchmark"]
    logs = args.output_dir or project / "build/test-logs"
    logs.mkdir(parents=True, exist_ok=True)
    state = logs / "state"
    state.mkdir(exist_ok=True)
    failed = []
    for check in checks:
        if not re.fullmatch(r"[a-z_]+", check) or not (tests / f"{check}.scd").is_file():
            parser.error(f"unknown check: {check}")
        log = logs / f"{check}.log"
        wrapper = logs / f"{check}-runner.scd"
        wrapper.write_text(isolated_script(tests / f"{check}.scd", state,
                                          args.audio_output_device, args.audio_input_channels), encoding="utf-8")
        timed_out = False
        with log.open("w", encoding="utf-8") as output:
            command = [args.sclang, "-D"]
            if args.sclang_config:
                command.extend(["-l", str(args.sclang_config.resolve())])
            command.append(str(wrapper.resolve()))
            process = spawn_owned(command, cwd=args.working_directory or project,
                                  stdout=output, stderr=subprocess.STDOUT, env=environment)
            try:
                process.wait(timeout=args.timeout)
            except subprocess.TimeoutExpired:
                timed_out = True
            finally:
                stop_process_tree(process)
        text = log.read_text(encoding="utf-8", errors="replace")
        ok = check_passed(check, text, process.returncode, timed_out)
        print(f"{'PASS' if ok else 'FAIL'} {check}: {log}", flush=True)
        if not ok:
            failed.append(check)
            print(text[-4000:], flush=True)
    return bool(failed)


if __name__ == "__main__":
    sys.exit(main())
