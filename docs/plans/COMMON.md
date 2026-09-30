# Shared agentic execution plan

Read this together with the selected platform plan, not as an automatic script.
Current milestone: [P1](../PLATFORM_MILESTONE.md). All commands start at the
SCShader project root. Run live GPU/audio suites serially in an interactive desktop.

## 0. Recover context and establish a baseline

1. Read `AGENTS.md`, `START_HERE.md`, `docs/STATUS.md`,
   `docs/IMPLEMENTATION_REPORT.md`, `docs/PLATFORMS.md`, and `docs/RECOVERY.md`.
   Consult `protocol/protocol.md` before changing transport or status fields.
2. Inspect local files and repository-root Git status. A source handoff has no
   `.git`; do not mistake an enclosing unrelated repository for this project's
   baseline. Keep the original archive and manifest intact. No publishing is allowed.
3. Copy [RESULT_TEMPLATE.md](RESULT_TEMPLATE.md) to a dated target result under
   `docs/platform-results/`. Record OS version, CPU/process architecture, GPU/driver,
   actual desktop session, SuperCollider version/architecture, Python, Rust host,
   source archive checksum or approved local commit and dirty state. Avoid hostnames.
4. Verify native prerequisites in the platform plan. Missing hardware/display is
   **blocked/not run**, not a failure of a tested renderer and not a pass. Do not
   replace hardware qualification with headless or software rendering silently.

## 1. Static checks and a native linked build

Run the following, stopping to investigate a nonzero exit. On Windows replace
`python3` with `python` and check `$LASTEXITCODE` after each external command.

```sh
cargo fmt --manifest-path renderer/Cargo.toml -- --check
cargo clippy --manifest-path renderer/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path renderer/Cargo.toml --locked --all-targets --all-features
python3 -m unittest discover -s tests/python -v
```

The native package commands are in each platform plan. They use explicit target
triples, verify host and binary headers, produce `stage/` and `dist/`, and include
dependency inventories marked unreviewed. Use `SCSHADER_ARTIFACT_PREFIX=fmiramar`.
Do not bypass a failed host, format, or strict-notice check. Development packaging
does not certify license completeness, signing, runtime, or minimum OS support.

Review `build-info.json`, executable format/architecture, version, and dynamic
dependencies. Hash the exact packaged executable. A cross-target `cargo check`
does not link and cannot replace this step. Keep builds free of `gpu-test-hooks`
except the dedicated recovery build below.

## 2. Install the actual package, then test SC integration

1. Extract the generated binary ZIP into a fresh folder. Locate its inner
   installable `SCShader/` directory (not this development source tree).
2. Determine the user's Extensions directory in SuperCollider. Back up any existing
   SCShader directory outside the scanned Extensions tree, then replace only that
   extension with the extracted package. Do not merge stale class/help files.
3. Recompile the class library. Confirm `ShaderDef.filenameSymbol` resolves inside
   the intended installation and the renderer is discovered without a placeholder
   override. Verify the installed executable hash matches the packaged one.
4. Set shell variables `renderer` and `extension` to the **absolute installed**
   executable and SCShader folder; set `sclang` to the actual SC executable. These
   are receiving-machine values, not portable defaults. Then run:

```sh
python3 tools/run_sc_checks.py --renderer "$renderer" --sclang "$sclang"
python3 tools/check_help_examples.py --extension "$extension" --sclang "$sclang" --output-dir build/platform-tests/help-01
```

PowerShell equivalent, after assigning `$Renderer`, `$Extension`, and `$Sclang`:

```powershell
python tools/run_sc_checks.py --renderer "$Renderer" --sclang "$Sclang"
if ($LASTEXITCODE -ne 0) { throw "SC checks failed" }
python tools/check_help_examples.py --extension "$Extension" --sclang "$Sclang" --output-dir build/platform-tests/help-01
if ($LASTEXITCODE -ne 0) { throw "Help examples failed" }
```

The SC suite currently covers 26 checks including compile, handshake, lifecycle/reboot,
process exit, platform validation, typing, scheduling, Patterns, images/feedback,
graphs, analysis, streamed textures, examples, and SCDoc. The separate help runner
executes 18 help/README demos and their cleanups (36 blocks) from fresh sessions,
including stale-default-path and unsaved-document cases. Expectations may grow
with new tests; never reduce the list to hide a failing platform test.

SCDoc indexing/rendering must have zero warnings/errors. Keep a working audio
device/backend available for audio examples. Do not kill another session to free
ports; close owned demos normally or coordinate with the user. Default live-test
ports include 57140, 57166–57168, and private audio 57312.

`run_sc_checks.py` reuses `build/test-logs/`; preserve that directory under a dated
ignored evidence folder before rerunning so negative results are not overwritten.
The other output directories must be new for each run. Do not paste earlier
origin-machine results into a new platform result as if they were rerun.

## 3. Short native acceptance, stress, and recovery

Run the explicit-backend/display commands in the selected platform plan with
`--seconds 10`. Without an explicit duration the acceptance harness defaults to
60 minutes: **do not omit the duration** in this milestone. The four modes check
traffic, trivial rendering, hot reload, and feedback/resize. Preserve JSON and logs.
Record actual adapter/backend, frame progress, fresh replies, RSS, drops,
suppressed diagnostics, and failures. Validate visible output too.

Use the installed production binary for the default-backend stress checks:

```sh
python3 tools/stress_schedule.py --renderer "$renderer"
python3 tools/stress_runtime.py --renderer "$renderer" --cycles 80 --seconds 2 --output-dir build/platform-tests/stress-01
```

On Windows use `python` and `$Renderer`, checking exit codes. Capture scheduling
test console output in the ignored evidence directory. A continuous flood that
never fills the incoming queue is load evidence, not saturation evidence. Keep
the distinction in the result record; deterministic rejection tests are separate.

For logical-device recovery, build a separate native debug executable using the
selected platform's target triple:

```sh
# Set target from the platform plan, then:
cargo build --manifest-path renderer/Cargo.toml --locked --target "$target" --features gpu-test-hooks --target-dir build/recovery-target
```

Run `tools/check_gpu_recovery.py --renderer` with that debug executable and `--log`
pointing to a new file under an existing ignored evidence directory. Windows needs
the `.exe` suffix and PowerShell's `$Target`. This test destroys only its child's
logical wgpu device and checks recovery/pixels/history reset. It does not simulate
a host driver reset. The harness currently uses the platform default backend;
alternate-backend recovery requires an explicit harness extension with tests.
Do not claim all backend/display combinations from one default run.

Reconfirm the installed/releasable executable is the ordinary release build, not
the hook-enabled binary. Do not force a real device/driver reset without separate
approval. If real failures occur naturally, save evidence and diagnose carefully.

## 4. Implement missing platform behavior from evidence

For every failure: record the command/hash and expected/actual behavior, reduce to
a small reproduction, identify whether the defect is in renderer, SC code, test,
packaging, or environment, then make a focused fix and regression test. Relevant
files are `renderer/src/platform.rs`, `main.rs`, `app.rs`, `renderer.rs`,
`Classes/ShaderServer.sc`, `tools/process_tools.py`, and packaging/test tools.
Preserve OSC compatibility, bounded work, and the Metal submission workaround.

Do not weaken invariants, pretend unsupported window-manager behavior succeeded,
or silently choose a different backend. Narrow platform-specific assertions only
when the API really cannot provide the behavior, document why, and retain tests
for actual observable behavior. Rebuild/reinstall after changes, rerun affected
tests and the critical suite, and update the recorded binary hash. Shared fixes
require regression on every available native target, including the Intel baseline.

## 5. Hardware and distribution review

Check window/fullscreen/input/focus, resizing/minimize/restore, HiDPI and mixed-DPI
display moves, different refresh rates, sleep/wake, hot reload while resizing,
audio/FFT/graph output, clean shutdown and abnormal child exit. Use the target plan
for OS-specific cases. Unavailable extra displays/GPUs are explicitly **not run**.
Record tested scope rather than advertising general vendor/OS compatibility.

Inspect minimum runtime dependencies and development-package launch behavior on a
machine without the Rust toolchain where available. Retain licensing/signing gaps
as release blockers. CI build jobs are configured, not evidence of a past hosted
run; this task does not authorize triggering hosted jobs or pushing tags.

## 6. Handoff and stop condition

Complete the result record, summarize fixes and remaining critical/noncritical
items, update `docs/STATUS.md`, `docs/VERIFICATION.md`, and milestone checkboxes
only where evidence supports them. Preserve raw logs under ignored `build/` and
link their relative names; share them separately if required.

This track is complete when a native package installs, critical short checks pass,
platform-specific behavior is documented, and no unexplained critical failure
remains. Hardware gaps require an explicit limitation/support decision; never mark
them passed. Then hand off the source changes and result record for integration.
One-hour scheduling, final notice/signing review, user-run eight-hour sign-off,
and authorized publication remain separate later steps.
