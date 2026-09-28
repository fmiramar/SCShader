# Stability and overload verification

Use the exact candidate executable throughout. Examples below use the ordinary
`cargo build --release` output; platform packagers instead build under
`renderer/target/<target-triple>/release/`, with `.exe` on Windows. Substitute that
path or the installed package executable. [PLATFORMS.md](PLATFORMS.md) provides
short Windows/X11/Wayland commands and backend/display overrides. Packaging and
cross-target checks do not qualify as native runtime or long-duration evidence.

Use Python 3.11 or newer and a native desktop session with a working GPU. These
tools use only the Python standard library. Unit checks do not open a window:

```sh
python3 -m unittest discover -s tests/python -v
cargo test --manifest-path renderer/Cargo.toml --locked
```

Installed documentation has a separate short execution check:

```sh
python3 tools/check_help_examples.py --extension /path/to/installed/SCShader \
  --output-dir build/help-examples/candidate
```

Substitute the actual extension directory and run with an interactive desktop.
This runs all six guide demos, ten class-help demos, and both README demos plus
their cleanup blocks verbatim, in separate language processes. It checks shader
readiness/frame progress, nonzero analysis/FFT data, resource-specific outcomes,
cleanup, and absence of compiler/renderer errors. A stale placeholder is left on
the default controller deliberately; examples must use their own fresh controller.
It refuses occupied renderer/audio test ports, uses a quiet private audio server,
and never rewrites example paths or relies on an active document filename. SCDoc
indexing/rendering is a separate check, not proof that examples execute.

Build the release renderer first. The live queue stress opens and owns one
renderer, refuses an occupied port, and stops only its child:

```sh
cargo build --manifest-path renderer/Cargo.toml --release --locked
python3 tools/stress_schedule.py --renderer renderer/target/release/scshader-renderer
```

It fills the future queue by command count and payload bytes independently,
checks rejection counters and both bounds, clears the queue, schedules new work,
and checks continuing frames/pongs. It also tests malformed-bundle all-or-none
parsing, inherited nested deadlines, excessive depth, and excessive lookahead.
Payload packets stay below macOS's usual 9216-byte UDP ceiling. This deliberately
tests scheduler saturation without also saturating the incoming packet queue.

## Short resource and incoming-flood checks

```sh
python3 tools/stress_runtime.py --renderer /path/to/candidate/scshader-renderer \
  --output-dir build/stress/candidate-runtime
```

This owns a fresh child per mode, saves its binary hash/platform and a JSON
result even on failure, and keeps logs. The default resource test repeats 80
cycles of two shaders, a bound image and Float32 buffer, and a selected graph.
It renders before/after freeing resources, reuses IDs, requires live counts and
owned-texture bytes to return exactly to baseline, and enforces 512 MiB absolute
RSS / 128 MiB growth after ten warmup cycles. These are bounded regressions,
not proof that a driver cache will remain flat indefinitely.

Three two-second traffic cases follow: malformed packets at 2000/sec, invalid
resource commands at 2000/sec, and 2000 valid bundles/sec containing 128 continuous
updates each (256,000 offered updates/sec). Offered traffic is not a measurement
of successfully received/executed commands; kernel UDP loss is not counted. They
check live pongs/status, frame progress while traffic is active, memory bounds,
and successful controls afterward. The error floods also require bounded reply
and stderr diagnostic counts plus a nonzero suppression counter. Modes, duration
and cycles can be selected within short-test bounds; no mode qualifies as an
hour/eight-hour run. These tests do not exhaust every UDP/driver/resource limit.

The continuous case records whether incoming-queue drops actually occurred. A
responsive run without drops is valid load evidence, not saturation evidence.
Use `--modes continuous --require-saturation` to require observed drops; that
variant fails if the host handles the offered load without filling the queue.
Deterministic Rust tests separately fill a one-slot channel, verify whole-bundle
rejection/counting, then drain it and verify admission resumes.

## Measured soaks

Acceptance policy (2026-09-24): the intended session length is at most one hour.
Current development validation is one measured **60-minute traffic run** on the
exact candidate, plus short overload, reload, feedback/resize, and recovery checks.
The user will perform an **eight-hour test as the last final-release sign-off**
after implementation and other release checks. It is not a development blocker
and must not start automatically. The former four-by-eight-hour matrix is an
optional extended qualification, not the current required workflow.

Run each scenario separately, on the exact candidate binary. Choose a new evidence
basename per run: existing files are refused, and parent directories are created.
The default duration is **60 minutes**, not a shortened simulation:

```sh
python3 tools/soak_uniforms.py \
  --renderer renderer/target/release/scshader-renderer --port 57166 \
  --mode traffic --minutes 60 --rate 1000 \
  --csv build/soak/traffic-60m.csv --json build/soak/traffic-60m.json \
  --renderer-log build/soak/traffic-60m-renderer.log
```

Add `--overlay` for a distinct diagnostics-enabled run. It verifies that the
overlay remains enabled and records that choice in the result; default runs
leave the panel disabled. Do not conflate a short overlay smoke with long-run
qualification of its additional GPU pass.

Use `--seconds 60` for a one-minute harness smoke; its result explicitly records
60 seconds and is **not** a completed one-hour or eight-hour acceptance run.
Use at least two seconds for meaningful status/frame-progress evidence. Available
scenarios (duration is chosen independently):

| Mode | Workload |
| --- | --- |
| `trivial` | Built-in animation with status/ping monitoring. |
| `traffic` | 1000 amount updates/sec by default; fails if the generator stalls or sends fewer than 99% of the requested updates. |
| `reload` | A temporary original shader edited about once/sec; each edit requires a watched-reload acknowledgement. |
| `feedback` | Prior-frame feedback at 0.92, alternating window size every 30 seconds. |

Run acceptance scenarios in isolation from compilation, profilers, and other
GPU/SC stress jobs. Concurrent workloads are useful separate stress tests, but
must be recorded as such; a queue drop still fails either run.

The serial runner defaults to **one traffic scenario lasting one hour**:

```sh
python3 tools/run_acceptance.py --renderer renderer/target/release/scshader-renderer \
  --output-dir build/soak/candidate-one-hour
```

Short auxiliary checks can be run separately:

```sh
python3 tools/run_acceptance.py --renderer renderer/target/release/scshader-renderer \
  --output-dir build/soak/candidate-short-checks --seconds 60 --modes trivial reload feedback
```

Only when the final candidate is ready, the user runs the final sign-off explicitly:

```sh
python3 tools/run_acceptance.py --renderer renderer/target/release/scshader-renderer \
  --output-dir build/soak/final-candidate-eight-hour --final-eight-hour
```

The final command runs traffic for eight hours, not four eight-hour scenarios.
`--modes` can select additional scenarios if deliberately desired. Keep the
desktop session awake and leave the owned verification window open. The runner
checks the binary hash before each stage, uses fixed traffic/memory bounds, and
stops at the first failure. Each mode saves CSV, JSON, console, and renderer logs
without needing interactive-agent polling. `run.json` records active work and
only sets `passed`/duration-qualification flags after completion; a one-hour
pass never sets `qualifies_as_eight_hour_check`. The suite flag additionally
requires all four modes. A terminated session or incomplete journal is not success.
Use `--seconds 3` and a fresh directory for a short orchestration smoke. Do not
rebuild/replace the selected executable between stages.

The harness first checks one intentional malformed-packet diagnostic. Any later
renderer diagnostic, diagnostic suppression, queue rejection/drop, child exit, missing reply, or lack of
frame-counter progress for more than five seconds fails the run. Fresh pongs
alone cannot hide a stalled render loop. Default process-memory limits are
512 MiB absolute RSS and 128 MiB growth over the first sample after a 30-second
warmup; explicit options can change these, and JSON records the chosen limits.
Missing RSS samples fail when monitoring a PID. Attachment to an already-running
renderer is supported by omitting `--renderer`; supply `--pid` for memory evidence.
Without a PID, JSON says memory was not monitored. `--quit` only stops an attached
renderer after success; an owned child is always cleaned up.

CSV streams observations during the run. JSON includes success/failure, actual
elapsed duration, counts, memory summary, backend/platform, and the SHA-256 of an
owned binary. Interrupted/failed runs are not passes. A machine sleep, crash,
session termination, or insufficient evidence never counts as elapsed acceptance.
Keep raw logs with release evidence; summarize only completed results in
[VERIFICATION.md](VERIFICATION.md).

Frame-counter progress establishes submitted rendering, not correct pixels or
display scan-out. Sent-update counts are not individual acknowledgements, and
application drop counters do not measure OS-level UDP loss. RSS is not GPU VRAM.
Manual visual checks, real driver-reset qualification,
native platform/display testing, and broader resource/resize stress remain separate
gates beyond the bounded churn checks above.
The implemented logical-device/surface recovery and overlay readback checks are
documented in [RECOVERY.md](RECOVERY.md); their test hooks are not shipped.

## Allocation diagnostics

An optional `gpu-counters` Cargo feature logs wgpu's live GPU-resource counters
every 600 submitted frames. It is off in ordinary release packages:

```sh
cargo build --manifest-path renderer/Cargo.toml --features gpu-counters --locked
```

Only in that diagnostic build, `SCSHADER_DIAGNOSTIC_WAIT=1` also waits (up to two
seconds) for GPU completion after each presentation. This changes performance
and is an isolation experiment, not a default runtime workaround. Rebuild without
the feature before producing ordinary release measurements/packages.

On macOS, the original `tools/metal_memory_probe.m` isolates offscreen Metal
rendering from Rust, wgpu, OSC, and window presentation. It renders 20,001 frames
with two passes by default, waits for the last command buffer per frame, reports
its own RSS, and aborts above 512 MiB:

```sh
mkdir -p build
clang -fobjc-arc -Wall -Wextra -Werror -framework Foundation -framework Metal \
  tools/metal_memory_probe.m -o build/metal_memory_probe
build/metal_memory_probe
build/metal_memory_probe single-pass
build/metal_memory_probe unretained
build/metal_memory_probe split-buffers signal-event completion-handler labels
```

The optional flags independently compare retained references, one/two passes,
separate command buffers, shared-event signaling, completion callbacks, and labels.
They do not modify the renderer. The corresponding offscreen wgpu comparison
uses existing dependencies and `ps` for RSS on macOS/Linux:

```sh
cargo build --manifest-path renderer/Cargo.toml --example memory_probe --locked
renderer/target/debug/examples/memory_probe
renderer/target/debug/examples/memory_probe single-pass
renderer/target/debug/examples/memory_probe separate-submissions upload recreate-bind-group
cargo run --manifest-path renderer/Cargo.toml --example pass_order_check --locked
```

The memory probe has the same 20,001-frame/512 MiB diagnostic bound and an explicit
per-frame GPU wait. Successful exit means the bounded experiment finished, **not**
that memory growth or long-soak acceptance passed; compare the printed trend.
`pass_order_check` validates exact graph/feedback/output pixels for both batched
and split submissions, with no CPU waits between its 40 feedback frames.

Profilers can suspend a process. A soak interrupted by allocation inspection may
correctly fail its five-second health deadline; such a run is diagnostic evidence,
not acceptance evidence. Do not increase health/memory thresholds to hide growth.
