#!/usr/bin/env python3
"""Execute every installed runnable SCShader help example and its cleanup verbatim."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import socket
import subprocess
import sys
import time

from process_tools import spawn_owned, stop_process_tree
from run_sc_checks import add_audio_arguments, find_sclang, isolated_script

GUIDE_MODES = ("typed", "pattern", "analysis", "fft", "graph", "window")
CLASS_MODES = {
    "ShaderServer": "server", "ShaderWindow": "window", "Shader": "shader",
    "ShaderToy": "shader", "ShaderDef": "definition", "ShaderBus": "bus",
    "ShaderBuffer": "buffer", "ShaderGraph": "graph", "ShaderAnalysis": "analysis",
    "ShaderFFTTexture": "fft",
}


def runnable_blocks(text):
    blocks = re.findall(r"^code::\s*\n(.*?)^::\s*$", text, re.M | re.S)
    for block in blocks:
        if not block.strip().startswith("(") or not block.strip().endswith(")"):
            raise ValueError("runnable help blocks must be parenthesized")
        if "/path/to/" in block or "thisProcess.nowExecutingPath" in block:
            raise ValueError("runnable help must use installed assets, not placeholder/document paths")
    return blocks


def check_ports_free(wait_seconds=0):
    # Never attach to, send quit to, or silently reuse an existing user renderer/server.
    deadline = time.monotonic() + wait_seconds
    for port in (57140, 57312):
        while True:
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
                    probe.bind(("127.0.0.1", port))
                break
            except OSError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.05)


def sc_fixture(setup, cleanup, mode):
    # Only the surrounding observer/owned audio server is test code. Neither
    # setup nor cleanup is rewritten; unsaved-document execution has a nil path.
    return r'''(
var setup = SETUP_CODE, cleanup = CLEANUP_CODE, mode = MODE_NAME;
var completed = false, status, frames, finish;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
s = Server(\scshaderHelpAudio, NetAddr("127.0.0.1", 57312));
Server.default = s;
s.options.numOutputBusChannels = 2;
s.volume = -90; // quiet automated verification, not a change to the example
ShaderServer.default.rendererPath_("/path/to/stale-renderer");
finish = { |passed|
    if(completed.not) {
        completed = true;
        ~scShaderSetup.tryPerform(\stop);
        p.tryPerform(\stop);
        ShaderServer.all.do { |server| server.free };
        s.quit;
        SystemClock.sched(1, {
            if(passed) { "SCSHADER_HELP_EXAMPLE_OK".postln; 0.exit } { 1.exit }; nil
        });
    };
};
thisProcess.nowExecutingPath = nil;
Routine({
    try {
        var deadline, ready = false, function;
        function = cleanup.compile;
        assert.value(function.notNil, "Cleanup failed to compile");
        function = setup.compile;
        assert.value(function.notNil, "Setup failed to compile");
        function.value;
        deadline = Main.elapsedTime + 12;
        while({ ready.not and: { Main.elapsedTime < deadline } }, {
            ready = if(mode == "definition") { d.notNil } {
                v.notNil and: { v.isRunning and: {
                    switch(mode,
                        "server", { true },
                        "window", { w.notNil },
                        "graph", { g.notNil and: { g.nodes.size == 2 } },
                        "pattern", { p.notNil and: { p.isPlaying } },
                        "analysis", { ~scShaderAnalysis.notNil and: {
                            ~scShaderAnalysis[\amplitude].value > 0.001 } },
                        "fft", { ~scShaderFFT.notNil and: {
                            ~scShaderFFT.buffer.asArray.maxItem > 0.0001 } },
                        { x.notNil and: { x.isRunning } }
                    )
                } }
            };
            0.02.wait;
        });
        assert.value(ready, "Example did not become ready");
        if(mode != "definition") {
            v.status { |server, value| status = value };
            deadline = Main.elapsedTime + 2;
            while({ status.isNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(status.notNil, "Missing status");
            frames = status[\frameIndex];
            0.8.wait;
            status = nil;
            v.status { |server, value| status = value };
            deadline = Main.elapsedTime + 2;
            while({ status.isNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(status.notNil and: { status[\frameIndex] > frames }, "Frames stopped");
            assert.value(status[\suppressedDiagnostics] == 0, "Suppressed diagnostics");
            assert.value(v.lastError.isNil, "Renderer reported an error");
            assert.value(ShaderServer.default.rendererPath == "/path/to/stale-renderer",
                "Example unexpectedly mutated the default controller");
            if(x.notNil) { assert.value(x.lastError.isNil, "Shader error") };
            if(mode == "typed") { assert.value(x.value(\enabled), "Typed setup did not finish") };
            if(mode == "bus") { assert.value(b.value > 0.7, "Bus did not update") };
            if(mode == "buffer") { assert.value(b.asArray.maxItem == 1, "Buffer did not fill") };
        };
        cleanup.interpret;
        if(mode != "definition") {
            deadline = Main.elapsedTime + 3;
            while({ v.pid.notNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(v.pid.isNil and: { v.isRunning.not }, "Cleanup left renderer running");
        };
        if(mode == "pattern") { assert.value(p.isPlaying.not, "Pattern still playing") };
        if(mode == "analysis") { assert.value(~scShaderAnalysis.synth.isNil, "Analysis leaked") };
        if(mode == "fft") { assert.value(~scShaderFFT.isFreed, "FFT stream leaked") };
        finish.value(true);
    } { |error| error.reportError; finish.value(false) };
}).play(SystemClock);
SystemClock.sched(20, { finish.value(false); nil });
)
'''.replace("SETUP_CODE", json.dumps(setup)).replace("CLEANUP_CODE", json.dumps(cleanup)).replace("MODE_NAME", json.dumps(mode))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--extension", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--sclang", default=find_sclang())
    parser.add_argument("--sclang-config", type=Path)
    parser.add_argument("--working-directory", type=Path, help="child launch directory, e.g. the SuperCollider installation")
    add_audio_arguments(parser)
    args = parser.parse_args()
    if not args.sclang or args.output_dir.exists():
        parser.error("sclang is required and output directory must be new")
    if args.audio_input_channels is not None and args.audio_input_channels < 0:
        parser.error("audio input channels must be non-negative")
    if args.sclang_config and not args.sclang_config.is_file():
        parser.error("sclang configuration does not exist")
    if args.working_directory and not args.working_directory.is_dir():
        parser.error("working directory does not exist")
    help_root = args.extension / "HelpSource"
    cases = []
    guide = runnable_blocks((help_root / "Guides/SCShader.schelp").read_text(encoding="utf-8"))
    if len(guide) != 12:
        raise ValueError("expected all six guide setup/cleanup pairs")
    for index, mode in enumerate(GUIDE_MODES):
        cases.append((f"guide-{mode}", mode, *guide[index * 2:index * 2 + 2]))
    for name, mode in CLASS_MODES.items():
        pair = runnable_blocks((help_root / f"Classes/{name}.schelp").read_text(encoding="utf-8"))
        if len(pair) != 2:
            raise ValueError(f"{name}: expected a setup/cleanup pair")
        cases.append((name, mode, *pair))
    covered = {"SCShader", *CLASS_MODES}
    for page in help_root.rglob("*.schelp"):
        if page.stem not in covered and runnable_blocks(page.read_text(encoding="utf-8")):
            raise ValueError(f"runnable page lacks an execution case: {page.name}")
    readme = re.findall(r"```supercollider\n(.*?)```", (args.extension / "README.md").read_text(encoding="utf-8"), re.S)
    readme = runnable_blocks("\n".join("code::\n" + block + "\n::\n" for block in readme))
    if len(readme) != 4:
        raise ValueError("expected both README setup/cleanup pairs")
    for index, mode in enumerate(("server", "pattern")):
        cases.append((f"readme-{mode}", mode, *readme[index * 2:index * 2 + 2]))
    check_ports_free()
    args.output_dir.mkdir(parents=True)
    state = args.output_dir / "state"
    state.mkdir()
    record = {"passed": False, "cases": {}, "blocks_executed": 0}
    for name, mode, setup, cleanup in cases:
        check_ports_free(wait_seconds=3)
        script, log = args.output_dir / f"{name}.scd", args.output_dir / f"{name}.log"
        script.write_text(sc_fixture(setup, cleanup, mode), encoding="utf-8")
        wrapper = args.output_dir / f"{name}-runner.scd"
        wrapper.write_text(isolated_script(script, state, args.audio_output_device,
                                          args.audio_input_channels), encoding="utf-8")
        with log.open("w") as output:
            command = [args.sclang, "-D"]
            if args.sclang_config:
                command.extend(["-l", str(args.sclang_config.resolve())])
            command.append(str(wrapper.resolve()))
            process = spawn_owned(command, cwd=args.working_directory, stdout=output, stderr=subprocess.STDOUT)
            try:
                process.wait(timeout=25)
            except subprocess.TimeoutExpired:
                pass
            finally:
                stop_process_tree(process)
        content = log.read_text(encoding="utf-8", errors="replace")
        passed = (process.returncode == 0 and "SCSHADER_HELP_EXAMPLE_OK" in content
                  and not re.search(r"ERROR:|FAILURE IN SERVER|Exception|SCShader .*error E_|WARNING: ShaderServer", content))
        record["cases"][name] = {"passed": passed, "sha256": hashlib.sha256((setup + cleanup).encode()).hexdigest()}
        record["blocks_executed"] += 2 if passed else 0
        print(f"{'PASS' if passed else 'FAIL'} {name}: {log}", flush=True)
        if not passed:
            print(content[-3000:], flush=True)
        record["passed"] = len(record["cases"]) == len(cases) and all(case["passed"] for case in record["cases"].values())
        (args.output_dir / "run.json").write_text(json.dumps(record, indent=2) + "\n")
    return 0 if record["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
