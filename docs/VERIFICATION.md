# Verification

## Windows desktop RTX 3060 traffic soaks — 2026-10-04

On the exact 0.0.18 D3D12 renderer (`cc782174...a5ae34`), RTX 3060 traffic passes
3,600.001 seconds with 3,599,919 updates, 81 sender-skipped ticks, 38.211 ms
maximum lag and bounded RSS. The subsequent eight-hour attempt fails after
2,482.785 seconds on `E_QUEUE_FULL`, dropping one continuous update; sender lag
peaks at 83.276 ms and RSS remains within limits. This reproduces the desktop
queue-drop symptom, does not qualify eight hours, and does not establish a root
cause. Four ten-second modes also pass on RTX 3060 and RX 580; formatting,
Clippy, Rust/Python tests, scheduling/runtime stress and logical recovery pass.
Investigate and fix the queue-service issue before another long soak. See the
[full result and exact evidence](platform-results/2026-10-04-windows-rtx3060-soaks.md).

The 2026-10-05 traced display-route controls pass for 60 seconds on both GPUs
across the RTX-driven left output and two Radeon-driven outputs. Two RTX/DX12
30-minute traffic runs also pass: one on each manufacturer's output. A requested
45-minute follow-up stops at 830.779 seconds because the sender's maximum lag
reaches 704.582 ms, above its 250 ms guard. That run records NVIDIA RTX 3060 and
no renderer queue-full event; it is a harness failure and does not reproduce or
resolve the earlier fault. The eight-hour gate remains open. See the
[traced diagnostics and limitations](platform-results/2026-10-04-windows-rtx3060-soaks.md).

The later WPT/Nsight diagnostic records a usable 60-second GPU-event smoke, but
the combined RTX-left run stops after 900.082 seconds on the 128 MiB RSS-growth
guard; sender lag peaks at 26.078 ms and no renderer queue-full event is recorded.
A matched WPR-only 900-second control passes with 2.508 ms maximum lag and
stable RSS. This contrast implicates Nsight in-process instrumentation but is
not a root-cause finding. Both runs are below one-hour qualification. The WPR
ETLs report zero lost buffers/events. See the
[capture diagnostic and trace limits](platform-results/2026-10-04-windows-rtx3060-soaks.md).

The follow-up compact WPR scheduler profile passes a 60-second smoke and a
45-minute RTX 3060 / D3D12 diagnostic, covering the previous 2,482.785-second
queue-failure point without reproducing it. The 45-minute run sends 2,700,000
updates, skips none, peaks at 6.706 ms sender lag, and records no sampled queue
drops or runtime renderer slow phase; WPR reports zero lost buffers/events.
The 4.1 GB ETL includes system-wide CPU context switches and DxgKrnl activity.
The harness now records lag checkpoints with UTC timestamps and the Python
sender PID. Python unittest discovery passes 135 tests. This is not a one-hour
qualification or a root-cause finding. See the
[scheduler capture and limits](platform-results/2026-10-04-windows-rtx3060-soaks.md).

The subsequent one-hour RTX-left WPR run enabled renderer timing tracing and
passes 3,600.001 seconds on the exact 0.0.18/D3D12 binary. It sends 3,599,992
updates with 8 skipped ticks, 22.755 ms maximum sender lag and stable RSS. No
queue-full event or post-startup phase above the trace's 50 ms threshold is
recorded; startup initialization took 417.195 ms. WPR reports zero lost
buffers/events. A two-second slice at the analogous historic failure time has
regular ReadyThread/Present event spacing, but cannot establish thread dispatch
latency or a root cause. xperf reports an Invalid Event decode warning during
extraction. This qualifies one-hour traffic for this route and binary only; the
intermittent queue-service issue and eight-hour gate remain open. [Exact result
and limitations](platform-results/2026-10-04-windows-rtx3060-soaks.md).

## Windows laptop eight-hour traffic checks — 2026-10-04

Both adapters independently pass the standard validator for 28,800 seconds:
Intel 28,799,902 updates / 19.310 ms maximum sender lag; NVIDIA 28,799,804 updates /
22.435 ms maximum lag. Sender-skipped ticks were 98/196, while all 2,881 CSV
samples per GPU show zero renderer queue drops or scheduled rejections. Memory
remained within the original limits. All candidate hashes match; renderer
`df4d0eff...dacea`, corrected harness identical to `a4cf5ac`, Python 3.12.11, D3D12.
The per-adapter eight-hour flags are true; the supervisor's stale overall false
flag is documented without rewriting raw evidence. Scope is traffic only on the
recorded laptop configuration. [Full evaluation](platform-results/2026-10-04-windows-laptop-eight-hour.md).

## Windows laptop one-hour traffic checks — 2026-10-03

Both user-authorized serial runs pass the standard acceptance validator:
Intel 3600.000730 seconds / 3,599,980 updates; NVIDIA 3600.000170 seconds /
3,599,978 updates. Maximum sender lag was 20.202/18.969 ms, skipped sender ticks
19/20, and sampled renderer drops/rejections zero. Memory stayed within bounds.
Harness `bfa86e6`, Python 3.12.11, unchanged renderer `df4d0eff...dacea`, D3D12.
These qualify one-hour traffic on each selected laptop adapter, not eight hours
or the desktop RTX failure. [Exact evidence and scope](platform-results/2026-10-03-windows-laptop-one-hour.md).

## Desktop/laptop sender-clock investigation — 2026-10-03

On Python 3.12.11, a 30-second scheduler-only control confirms 15/16 ms clock
steps. Each laptop GPU passes 60-second traffic diagnostics with both the original
clock and `perf_counter()`; maximum bursts decrease from 16 to four. After the
harness correction, both GPUs pass all four ten-second modes on the unchanged
`df4d0eff...dacea` renderer. No long-duration or desktop rerun is claimed, and
the >250 ms sender stall remains unexplained. The new harness records clock and
Python versions. See the [full comparison](platform-results/2026-10-03-windows-soak-comparison.md).

Follow-up on `d1f9842`: an unattended ten-second preflight and two 600-second
traffic runs pass on Intel/NVIDIA, with 599,998/599,999 updates, zero skipped
updates and zero sampled queue drops. Maximum lag is 15.847/14.087 ms. A 60-second
scheduler-only control completes with 3.171 ms maximum lag. Windows system
profiling policy could not be enabled; renderer/sender traces are preserved.
No historical stall reproduced, and these passes do not qualify longer durations.

## Windows dual-GPU soak results and timing diagnostics — 2026-10-03

The three October 2 eight-hour attempts failed on incoming OSC queue overflow
after 1144.529, 952.906 and 894.918 seconds. Optional timing/display-power tracing
then captured a 256 ms command-service gap during a short failure, with the
display on and sender lag below 4 ms. The repeat passes four ten-second modes;
the 30-minute diagnostic was interrupted at the user's request, with partial
samples through 230.015 seconds. The interrupted diagnostic supplies no long-run
pass, and no RTX root-cause fix is claimed. Pre-commit Rust formatting/Clippy,
61 Rust tests and all 132 Python tests pass. Exact hashes, negative evidence and resume
instructions are in [the investigation](platform-results/2026-10-03-windows-soak.md).

The user resumed testing with sequential eight-hour runs on RTX 3060 and RX 580,
using the same packaged 0.0.18 renderer. AC display-off and sleep were set to Never.
RTX failed on a queue drop after 107 seconds; RX 580 passed 28,800 seconds, sending
28.8 million updates with no drops. The exact renderer passes the traffic-only
eight-hour check on RX 580; it does not qualify the changed diagnostic renderer
or the full four-mode suite. The RTX 3060 failure remains open. Review also fixed
the sender's high-rate catch-up cap without changing the cap at 1,000 Hz, the
99% rate requirement or renderer-drop failure checks. See the
[Windows soak record](platform-results/2026-10-03-windows-soak.md).

### Intel UHD 630 and NVIDIA GTX 1050 Ti Max-Q laptop follow-up — 2026-10-03

Both adapters passed the four-mode, ten-second D3D12 preflight. On the same
development renderer, the Intel eight-hour traffic attempt ran 20,199.89 seconds
before the harness aborted after a 311.996 ms sender-lag spike. It sent 20,199,532
updates, advanced from frame 6 to 1,212,005, recorded zero sampled queue drops and peaked at
333,020 KiB RSS. The NVIDIA attempt aborted after 15.781 seconds when sender lag
reached 436.999 ms; it sent 13,652 updates and recorded zero queue drops. Neither
run qualifies as an eight-hour check. The renderer reported the intended adapter
in each run; no queue overflow, device loss, or other renderer diagnostic was
recorded beyond the harness's intentional malformed-packet probe. The sender-lag
guard and exact outcomes are in the
[laptop soak record](platform-results/2026-10-03-windows-uhd630-gtx1050ti-soaks.md).

## Notice and corresponding-source packaging — 2026-10-02

Development 0.0.18, starting at `2c790b1`; renderer, SC classes, dependency pins and
version metadata are unchanged. Implementation/review began October 1 and the
final package checks resumed October 2. Runtime provisioning and signing work
are explicitly deferred to a later stable release, not marked passed.

- **Pass:** all five locked/offline `--distribution --require-texts` notice gates:
  Windows x64 123 packages; Linux x64/arm64 152 each; macOS x64/arm64 132 each;
  zero missing texts/declarations. These are metadata checks, not native builds.
- **Pass:** 125 Python tests before sandboxing; the final managed-sandbox rerun
  discovers 125, passes 124 and skips only `test_symlink_rejected` because symlink
  creation needs privileges. No failures/errors. Added nine regressions cover
  source-code false positives, explicit recovery provenance, embedded excerpts,
  stale review rejection, complete source packaging, offline vendor configuration,
  source drift, archive tampering and overwrite protection.
- **Pass:** native Windows packaging with Rust/Cargo 1.97.1, MSVC 14.44,
  SDK 10.0.26100.0 and Python 3.14.0. Final inspection prefix:
  `fmiramar-license-check02`. The source ZIP has 16,160 files and 281 vendored
  crates, including all locked target dependencies. Source manifest, ZIP checksums,
  full GPL text, Rust standard-library notices and source/binary/lock linkage pass.
- **Pass:** extract the first inspection source ZIP, resolve all 124 Windows
  packages (renderer included) from its vendor directory using an initially empty
  `CARGO_HOME`, then `cargo build --release --locked --offline` with a fresh target
  directory and native MSVC. The resulting renderer reports 0.0.18. Compiler/SDK
  were already installed; this tests offline Cargo sources, not clean-machine
  runtime provisioning. Rebuilt hash differs from the original; no byte-identical
  build claim is made. Runtime/vendor sources are unchanged in the final ZIP.
- **Pass:** both shell packagers pass `bash -n`; `git diff --check` is clean.
  CI now invokes them through `bash`, fixing the previous Unix package-job
  permission-denied failure. Branch CI will validate the committed revision;
  local results do not imply hosted jobs or native macOS/Linux builds passed.
- **Negative evidence retained:** an initial filename whitelist omitted valid
  notices (`NOTICES.md`, `COPYING.LIB`, version-suffixed Apache files). Comparing
  against the original inventory caught this; the final scanner retains every
  original notice except the four `copying.rs` false positives. The first ZIP
  is retained as inspection evidence, not the final notice package.
- **Not rerun:** GPU/audio/SC interaction suites and long-duration tests, since
  no runtime source changed. No installed extension was replaced, release tagged,
  binary uploaded, signing identity changed or upstream message sent.

| Artifact | SHA-256 |
| --- | --- |
| Final inspection binary ZIP | `1e44fdc5e005b83c95f87a6be1ca036eeae896cca8bd12eb8b39133043156711` |
| Final inspection corresponding-source ZIP | `d8208d6f3af33b32c851568c0b677cce1c8e94f56e83e70fca5a9ebcd1b33886` |
| Packaged production renderer (unchanged) | `b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff` |
| Offline rebuilt renderer | `70e066b8ca1674b8e5e77e7d8b68225390d3f9d2ab6e667b7e367962cd954d06` |
| Cargo.lock (unchanged) | `99477baa12a417c9c2aee715b1bc27cb59bb40035f7b4855480ef07e5fe762f7` |

Local/ignored evidence is under `build/license-review-20261001/`,
`build/license-review-20261002/`, `stage/` and `dist/`. The inspection source
snapshot precedes this final verification documentation. See
[NOTICE_REVIEW.md](NOTICE_REVIEW.md) for declaration-backed dispatch recovery and
the preserved Apple SDK context; passing these checks is not legal certification.

## Windows desktop with two GPUs and three displays — 2026-09-30

The 0.0.18 production renderer
`b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff`
passes four explicitly ten-second D3D12 modes on both RTX 3060 and RX 580.
Each GPU also passes 21 native window stages, including all three display moves,
fullscreen coverage and return to windowed mode. Native DPI/client/framebuffer
measurements and GPU allocation checks confirm the geometry changes. All three
outputs are currently 1920x1080 at 100% scale, nominal 59 Hz; mixed DPI is not run.
The user confirms the installed interaction example passes visible animation,
mouse/key controls, three-display dragging, resizing, minimize/restore, reload
status and cleanup. This manual session did not separately record its adapter.
Audio listening/analysis/FFT observations and sleep/wake remain unreported.

Formatting, strict Clippy and all 59 Rust tests pass. Eight new monitor/ownership
regressions bring Python to 116 passing tests. The native package has 83 installed
files with matching hashes and zero missing license texts across 123 audited
packages; notice review remains separate. All 26 installed SC checks, 13 rendered
SCDoc pages and 18 help demos/36 blocks pass using a dedicated class configuration.
Normal IDE compile/lifecycle passes; an unrelated Quark's standalone `Document`
error is preserved as negative evidence. Five SC checks also pass from a package
path containing spaces and non-ASCII characters, launched from the SC directory.

Default scheduling bounds pass. RX 580 resource/flood stress and feature-gated
logical recovery pass, including exact pixels and second-loss exit 70. Continuous
flood traffic did not saturate the incoming queue. The complete record, package
and recovery hashes, measured stress figures, original policy/Quark failures and
remaining manual/release gates are in the
[desktop result](platform-results/2026-09-30-windows-dual-gpu.md).
Raw evidence remains in `build/platform-tests/windows-dual-gpu-20260930/`.

## macOS native interaction and normal-close follow-up — 2026-09-30

The native Apple M5/Metal interaction runner passes all 19 stages in
`build/platform-tests/macos-arm64-interactions-final/`: resize/reload with native
geometry and GPU allocation checks, Retina mapping, fullscreen/decorations,
synthetic F/Escape delivery, resize/focus events, stats, minimize, restore/front,
and native close with exit 0. The production hash remains
`8409c5457b5694d344cf249850e7bbb43ca04d620e35bdd9c99891f111980f21`.
Thirteen ownership/evidence regressions bring Python to 108 discovered,
104 passed and four Windows-only skips (`macos-followup-python-final.log`).

The user confirms physical mouse, keyboard, focus, fullscreen, reload and stats
behavior. Escape was tested explicitly after F entered fullscreen and passed.
The posted log exposed a real normal-close warning: the old SC class treated
exit 0 after ready as `E_BOOT`. The new `window_close_smoke` fails against the
previous install (`macos-window-close-before/`), then passes with the correction.
It checks shader invalidation, restart and continued rejection of exit 0 before
handshake. Focused candidate checks and all 26 installed checks pass, including
13 warning-free SCDoc pages. Complete console evidence is in
`build/platform-tests/macos-window-close-installed-sc.console.log`.

The initial interaction runs `-01` and `-03` retain harness failure evidence:
temporary AppKit window disappearance during fullscreen, and a restore operation
that did not itself activate the app. The corrected checks retain ownership and
explicitly exercise window-front before requiring focus gain. No renderer change
or backend fallback was needed. The first package audit also found two staged
Finder metadata files; the macOS packager now excludes those and resource metadata.

The locked arm64 notice audit still fails with ten missing texts. The ad-hoc
binary's signature integrity passes, while Gatekeeper assessment rejects it;
the local executable has no quarantine attribute. No transferred-package result
or distribution clearance is claimed. Listening/sleep-wake observations and
external/Intel hardware, signing/notices and long-duration gates remain separate.
See [the result](platform-results/2026-09-30-macos-arm64.md) for package and backup
identities and [the procedure](MACOS_INTERACTION_CHECKS.md) for remaining checks.

## macOS Apple Silicon build and short-runtime checkpoint — 2026-09-30

The pinned Rust 1.97.1 toolchain and locked dependencies are available. Formatting,
warnings-as-errors Clippy, all 59 Rust tests, and 91 Python tests (95 discovered,
four Windows-only skips) pass. The native `aarch64-apple-darwin` 0.0.18 production
package and separate recovery-hook build succeed. The executable is arm64, links
Apple system libraries only, and declares macOS 11.0 in its build load command.
The package audit still lacks ten third-party license texts. The 82-file package
is installed in the normal Extensions folder; all 82 file hashes match the ZIP.

User-run logs confirm four ten-second Metal modes on Apple M5 and 18 help demos
(36 blocks). After the session sandbox was lifted, all 25 installed SC checks
were rerun directly with harness exit 0 and complete PASS console evidence in
`build/platform-tests/macos-arm64-sc-02.console.log`. SCDoc rendered 13 pages with
zero warnings/errors. Scheduling bounds, 80 resource cycles and short floods pass.
Continuous traffic offered 511,744 commands without filling the incoming queue;
do not label that result saturation coverage. The isolated recovery harness now
passes all exact-pixel/resource/control/schedule assertions and confirms the
expected second-loss exit 70, with both console and renderer logs saved.

The short feedback test previously scheduled its first resize at 30 seconds,
so its ten-second pass contained zero resizes. The corrected harness uses a
2.5-second cadence for a ten-second run, confirms each requested logical size
through window metrics, checks framebuffer/scale consistency, and rejects missing
confirmations. Seven regression tests cover the changed behavior, including a
simulated timed loop and ignored resize requests. The corrected native Metal
run confirms three resizes at 2x Retina scale, alternating 1280x720/2560x1440
and 960x540/1920x1080 logical/framebuffer sizes. Existing long-soak cadence
remains 30 seconds.

The earlier agent environment denied local UDP and CPU feature reads; this is
resolved after enabling full session access. UDP bind/send/receive, NEON feature
query and `sclang -v` now pass. Native execution reports `sysctl.proc_translated=0`.
Negative evidence is retained. The
[remaining-check runner](../tools/check_macos_remaining.sh) completed successfully
under `build/platform-tests/macos-arm64-remaining-lpdK9W/`, with unchanged
production/recovery executable hashes. Additional display/input/lifecycle,
Intel regression and long-duration gates remain open. See the
[macOS result](platform-results/2026-09-30-macos-arm64.md) for hashes and limits.

## Linux interaction and scaling follow-up - 2026-09-29

The exact production 0.0.18 renderer passed the full native Wayland compositor
and Xwayland OSC interaction suites on both NVIDIA and Intel. Earlier Xwayland NVIDIA checks exposed a test
coordinate mismatch: 900x500 framebuffer/compositor dimensions were compared
against 600x333 renderer logical dimensions at 150% scaling. The corrected
checker distinguishes these spaces and reads the compositor's Xwayland scaling
configuration, accepting both boolean and legacy integer option responses.
Renderer code and executable hash are unchanged.

All 23 interaction harness tests pass, including scaled Xwayland OSC/compositor
paths, refused requests, stale compositor geometry and stale GPU allocations.
The complete Python suite discovers 88 tests: 84 pass and four native-Windows
checks skip. Evidence: `linux-continue-20260929-vovfUJ/python-final.log` under
`build/platform-tests/`. The native interaction matrix passes in
`linux-finish-20260929-021852/native-20260929-060354/`. A later run at
`linux-finish-20260929-021852/native-20260929-060950/` passes all 16 ten-second
GPU/display/mode acceptance combinations and both 25-check installed SC suites.
Preserving `DISPLAY` fixed NVIDIA discovery during explicitly selected Wayland
acceptance. The updated guide and README use an explicit 150 ms Pattern lead time
after an earlier 60.980 ms late event. The stabilized follow-up passes help/SCDoc
on both displays, scheduling limits, stress, logical recovery and path checks;
the candidate remains installed with 82 verified files.
See the
[detailed result](platform-results/2026-09-29-linux-interactions.md).

Follow-up `native-tail-20260929-062528/` passes updated SCDoc and all 18 help
demos/36 blocks on each display, plus short scheduling limits. Resource/flood
stress initially failed its exact memory baseline comparison; startup window
resizing was identified from the allocation sizes. The checker now waits for
stable dimensions and records them, preserving exact leak detection and rejecting
later size changes. The stabilized rerun
`native-tail-20260929-063408/` passes 80 resource cycles, protocol/command/
continuous floods, logical Vulkan recovery, and path checks from a directory with
spaces and non-ASCII text. The full Python suite is 88 discovered: 84 pass and
four native-Windows checks skip (`linux-continue-20260929-vovfUJ/python-stable-window.log`).
The selected CachyOS/Hyprland Linux short gate is complete; manual desktop/audio
observations and long release checks remain.

## Linux resize follow-up - 2026-09-28

Development 0.0.18 handles winit's synchronous physical-size result and avoids
clearing GPU targets on unchanged-size notifications. Formatting, locked/offline
warnings-as-errors Clippy, and all 59 Rust tests pass. Python: 75 discovered,
71 pass and four native-Windows skips, including 13 new interaction-harness
tests. These test strict window ownership, IPC/error evidence and detection of
OS metrics changing while GPU target allocations remain stale.

The user's interactive desktop has native IPC and Vulkan device access. The first
user-run 0.0.18 NVIDIA/Wayland check launched the release binary and drew frames,
but timed out at its first client resize. The updated run verified compositor-driven
resize, GPU allocation changes and frame progress, then confirmed the first
SCShader OSC resize is refused in this Hyprland Wayland session. Native Wayland is
recorded with compositor-driven resizing; Xwayland remains the OSC resize path.
The installed 0.0.17 checkpoint is retained, and its prior passing suites do not
qualify the changed executable. See the
[resume result](platform-results/2026-09-28-linux-resize.md) and
[interaction instructions](LINUX_INTERACTION_CHECKS.md).

The saved 2026-09-29 checkpoint expanded the harness to 16 regression tests,
including explicit resize-driver scope and refusal without fallback. The full
Python suite discovered 78 tests: 74 passed, four native-Windows skips. At that
checkpoint no full 0.0.18 hardware interaction or installed SC suite had passed;
the later 2026-09-29 results are recorded above.

## Windows IDE corrections - 2026-09-28

The Linux source handoff inventory includes 152 source files plus its manifest.
The first archive passes CRC/content/permission checks. A fresh extraction passes
61 Python tests with one Windows symlink-privilege skip, Rust formatting, and
locked/offline Cargo manifest inspection. These are transfer checks on Windows,
not native Linux results. The revised r5 archive carries the later user feedback
and clarified manual example; preserve its checksum adjacent to the ZIP. It
contains 152 source files.

Follow-up interaction checks on the same installed binary pass on NVIDIA and
Intel: four resizes during watched reload, fullscreen and borders, actual Windows
minimize/restore, fresh frame progress and clean close. One active display reports
125% scale and 1920x1080 fullscreen. The new manual example's setup/callbacks pass
against installed classes. The user reports the window/input actions worked;
the R discussion concerned unchanged appearance and the key-repeat flag, not a
reported renderer error. The user confirmed `ShaderFFTTexture` worked perfectly
in the IDE; ShaderAnalysis listening and display/sleep observations remain pending.
See [the checklist](WINDOWS_INTERACTION_CHECKS.md). Python remains 61
passes and one privilege-dependent skip (`python-handoff-17.log`). No production
renderer or class code changed in this follow-up.

Development 0.0.17 reproduces and fixes the IDE launch failure caused by an older
DXC DLL in SuperCollider's working directory. Explicit FXC selection passes from
that directory. The SC boot handshake now ignores duplicate ready replies instead
of replaying window configuration; the failing bus-startup check passes.

All 25 SC checks have passing evidence across the candidate suite and one focused
recheck of the corrected asynchronous typed-example test. All 18 help/README
demos and their cleanups pass (36 blocks), and SCDoc renders all 13 pages without
warnings/errors. Tests use a candidate extension path with spaces/non-ASCII text,
the SC installation as working directory, and isolated test audio/cache settings.
Rust formatting, warnings-as-errors Clippy, and all 59 Rust tests pass. Python:
61 passes, one symlink-privilege skip, including native descendant cleanup tests.

Both NVIDIA and Intel pass all four ten-second D3D12 modes. NVIDIA also passes
future-scheduling bounds, 80 resource cycles, two-second floods after correcting
probe placement, and exact-pixel logical-device recovery. Earlier flood failures,
the correction and measurement limits are preserved in the
[0.0.17 Windows record](platform-results/2026-09-28-windows-x64.md).
The 0.0.17 ZIP passes integrity/hash checks; its extracted copy also passes class
compile, lifecycle, all four runnable example files and all 13 SCDoc pages from
SuperCollider's working directory (`sc-zip-17-recheck/` in the evidence folder).
The user installed that package with a 0.0.16 backup outside Extensions. All 73
installed files match the ZIP. Installed compile, handshake, lifecycle, bus
mapping, four example files and all 13 SCDoc pages pass in `sc-installed-17/`.
No new long-duration or other-platform qualification is claimed.

## Native Windows development checks - 2026-09-27

The release ZIP is PE32+ AMD64 and is installed locally. Four 10-second D3D12
modes pass separately on NVIDIA GTX 1050 Ti Max-Q and Intel UHD 630. Rust: 59
tests pass; Python: 60 discovered, 59 pass, one privilege-dependent skip. All 13
installed help pages render without warnings/errors with an isolated class
configuration. Of 24 SC checks, 23 have passing evidence across initial/focused
runs; startup bus mapping still fails a late-event assertion. Installed help
execution is 17/18 demos, with `ShaderAnalysis` failing on the same diagnostic.
The user IDE boot failure, stronger timeout cleanup, stress/recovery and long-run
gates remain open. Exact hashes and failed attempts are in the
[Windows result record](platform-results/2026-09-27-windows-x64.md).

## Portable source handoff and platform milestone — 2026-09-25

Saved the user's platform-first priority in `PLATFORM_MILESTONE.md`, with shared
and per-platform plans for Windows x64, Linux x64/optional arm64, macOS arm64,
and macOS x64 regression. No receiving-machine test is claimed by writing a plan.

- All **49 Python tests** pass, including eight source-package tests for untracked
  source/hidden CI inclusion, generated-state exclusion, source completeness,
  symlink/unreviewed-file rejection, deterministic output, no overwrite, safe
  archive paths, and content/inventory verification.
- A preliminary source archive was extracted into a fresh directory outside the
  workspace. All 49 Python tests, `cargo fmt -- --check`, and
  `cargo metadata --locked --offline --no-deps --format-version 1` pass there.
  Manifest inspection is not a clean native link or GPU/runtime qualification.
- ZIP inventory matches all **136 tracked and untracked SCShader source files**
  plus the adjacent original implementation plan. Internal per-file SHA-256 and
  permission checks pass; the source packager also writes an external ZIP checksum.
- Relative links in all nine new handoff/milestone/audit/plan documents resolve.
  Both POSIX binary packagers pass `bash -n`; packaging now also carries the root
  handoff/agent Markdown files referenced by the updated docs. Native Windows
  PowerShell execution remains pending on Windows.
- `git diff --check` passes for `07-SCshaders/`. The read-only Git audit found
  uncommitted/untracked implementation and unrelated workspace changes; nothing
  was staged, committed, authenticated, pushed, tagged, or published.

The source transfer contains no compiled renderer, Cargo dependency cache, Git
history, or historical raw test logs. It is a development handoff, not an offline
SDK or installable release. No Rust/SC runtime code or help-example code changed;
no native GPU suites, one-hour, or eight-hour test was rerun for this packaging task.


## Installed-example correction — 0.0.16 documentation update

User-reported failures on 2026-09-25 exposed a coverage gap: SCDoc had rendered
all pages, but the example runner executed only the first guide block, plus the
four example-file setup blocks with document paths substituted. That did not
verify the other help snippets or copy/paste execution. Placeholder renderer/
shader paths, undeclared audio variables, and missing graph setup were real
documentation defects, not evidence of a missing installed renderer.

The guide, ten class pages, and README now supply complete installed-asset demos
with cleanup. Each visual demo uses a fresh controller, so an old bad path on
the default controller cannot poison it. Audio examples use looping
`ExampleFiles.child`, synchronized buffer/SynthDef setup, an optional commented
live-input source, quiet monitoring, and cleanup of their own resources. The FFT
demo includes an original spectrum-bar shader. Examples 01–04 no longer require
the path of the currently executing document.

- `tools/check_help_examples.py` passed all **18 demos and 18 cleanup blocks**
  verbatim: six guide, ten class-help, and two README pairs. Each ran in a fresh
  language process with a deliberately invalid path on the default controller,
  no document path, installed assets, and private audio-server ownership.
- Checks covered shader readiness, frame progress, nonzero analysis/FFT data,
  Pattern/graph/bus/buffer outcomes, renderer exit, and cleanup diagnostics. Logs
  and exact code hashes are in `build/help-examples/2026-09-25-cleanup/run.json`.
- The first run stopped after audio-server shutdown had not yet released its
  private port. Its analysis log also exposed scheduled updates arriving after
  the shader was freed. Cleanup now stops producers and clears future work first;
  the harness waits briefly for port release and rejects cleanup diagnostics.
  The incomplete initial run is not a pass.
- The four standalone example setup blocks passed unchanged with a nil document
  path and stale default override. The packaged typed example also passed its
  value roundtrip. All **13 help pages** reindexed/rendered without SCDoc warnings
  or errors. All **41 Python tests** pass.

Installed docs/examples and the new shader fixture were updated locally. The
native renderer remains SHA-256
`f9a16109461cac1d8c56ea698fd53820d6013866d5739ae9f29f1d231ec82d0b`.
No native Rust/class implementation changed, no new long run was started, and
no public release was made. Reopen an already-open help page to load corrected
content; run one demo at a time and its cleanup before the next.

## Resource churn and diagnostic flood hardening — 0.0.16

Local macOS x64 / Metal checks on 2026-09-25:

- The saved 0.0.15 baseline passed 80 resource-churn cycles but **failed** both
  two-second error floods: 3,999 protocol replies and 3,998 invalid-command
  replies. Records remain in `build/stress/0.0.15-baseline/run.json`.
- The final 0.0.16 candidate passed all four short modes in
  `build/stress/0.0.16-final/run.json`. Eighty create/bind/render/free cycles
  returned shader/image/buffer counts and owned-texture bytes exactly to baseline.
  RSS was 43,848 KiB after warmup and 45,028 KiB at completion (1,180 KiB growth).
- Each error flood offered 3,998 packets and produced **59 replies and 59 logged
  diagnostics**, with 3,939 suppressed. Frame-health sample gaps stayed below
  0.202 seconds; pongs/status and post-flood controls remained responsive.
- The valid-update mode offered 511,616 updates in 128-member bundles over about
  two seconds. No incoming drops, errors, or diagnostic suppression were observed;
  RSS rose from 41,932 to 45,476 KiB and live frame-health gaps stayed below
  0.202 seconds. This is offered-load evidence, not executed-command accounting
  or native queue-saturation evidence; kernel UDP loss is not measured.
- An earlier run in `build/stress/0.0.16-incoming-saturation/run.json` remains
  **failed** because its assertion required saturation and no drops occurred,
  despite healthy rendering at 511,872 offered updates. The harness now records
  observed saturation separately; `--require-saturation` retains the strict
  requirement. Deterministic full-channel tests separately verify whole-bundle
  rejection, exact drop counts, sampled diagnostics, and admission after draining.
- All 55 Rust unit tests, all-target/all-feature warnings-as-errors Clippy, and
  37 Python tests passed. Unit coverage includes shared-thread diagnostic bounds,
  refill/idle-credit limits, critical-error bypass, UTF-8 truncation, queue-full
  admission, and failure of ordinary soaks when diagnostics are suppressed.
- All-target/all-feature cross-target checks again passed for Windows x64 MSVC
  and Linux x64 GNU. They do not link or execute native Windows/Linux binaries.
- Live future-scheduler stress and logical-device recovery passed. GPU readback
  checked exact restored pixels/resources, ramps/clocks/schedules, overlay
  isolation, surface recreation, and second-loss exit 70. This is not a physical
  GPU reset; test hooks are absent from the ordinary package.
- Four **three-second** Metal smoke modes passed under
  `build/soak/0.0.16-short/`: traffic, trivial, watched reload, and feedback.
  This duration exercised two watched reloads but no timed resize; it is not a
  resize-stress or one-hour result.
- All 24 installed SC checks passed using the packaged executable/examples.
  SCDoc indexed and rendered all 12 class pages and the project guide without
  warnings/errors, including a rerun after the final guide update. The previous
  extension installation was backed up before replacement.

The ordinary tested/installed macOS x64 renderer has SHA-256
`f9a16109461cac1d8c56ea698fd53820d6013866d5739ae9f29f1d231ec82d0b`.
Its explicit target, x86_64 Mach-O header, and system-only dynamic dependencies
were checked. The local development bundle retains the unreviewed notice audit:
ten macOS full-text gaps remain. No public release, native Windows/Linux runtime
pass, or new long-duration qualification is claimed; no long test is queued.

## Windows/Linux implementation preparation — 0.0.15

Local implementation date: 2026-09-24. Native Windows/Linux runtime tests are
explicitly deferred to other machines; see [PLATFORMS.md](PLATFORMS.md).

- `cargo check --locked --offline --all-targets --all-features` passed with
  explicit `x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu` targets on the
  macOS host. These checks compile/type-check platform code without linking or
  executing Windows/Linux binaries. Linux includes both X11 and Wayland paths.
- All 49 renderer unit tests and warnings-as-errors all-target/all-feature Clippy
  passed locally. All 34 Python tests passed, including mocked Windows cleanup,
  OS/Rust-host rejection, ELF/PE/Mach-O header checks, and backend/display evidence.
- An actual attempted Linux target check on macOS failed as intended before
  package staging. The native macOS package passed target/header/architecture
  checks and includes portable `build-info.json` and its unreviewed notice audit.
- CLI version/help passed without a window. Unsupported macOS D3D12 and explicit
  Wayland choices failed clearly without launching a renderer window.
- All 24 installed SC checks passed using the packaged renderer and packaged
  example paths. The 12 class pages and project guide indexed/rendered without
  SCDoc warnings or errors. The older installation was backed up before updating.
- Four explicit-Metal **three-second** acceptance-runner checks passed (traffic,
  trivial, watched reload, feedback/resize), with requested backend/display saved
  in `build/soak/0.0.15-platform-metal-short/`. These are short orchestration and
  regression checks, not one-hour qualification.
- Feature-gated logical-device recovery again passed exact GPU pixel, graph,
  cached-resource, ramp/clock/schedule, overlay-isolation, surface-recreation, and
  second-loss exit-70 checks. This is local Metal evidence, not D3D12/Vulkan
  device-reset evidence. Test hooks are absent from the ordinary packaged build.

The ordinary macOS x64 renderer used above has SHA-256
`fac7e4a620b0af9f79ba397debc96ac57aea342a2558cfff910c96b24669a430`.

No Windows/Linux linked archive, native GPU pass, hosted CI pass, or one-hour
0.0.15 qualification is claimed. The pinned 0.0.14 one-hour result below remains
historical evidence for that earlier binary only. No long test is queued.

## Diagnostics/recovery and revised validation policy — 0.0.14

Local macOS x64 / Metal checks on 2026-09-24:

- All 46 renderer unit tests passed with ordinary/all-feature builds; all-target
  Clippy passed with warnings denied. The current Python harness/audit suite has
  24 passing tests, including one-hour defaults, explicit eight-hour opt-in, and
  pinned upstream notice validation.
- All 23 installed SC checks passed after fixing a ShaderServer help-section
  ordering error and rerunning SCDoc. All 12 class pages and the guide rendered
  without SCDoc warnings/errors. The buffer smoke now transfers 16,384 values.
- Native overlay readback verified glyph, blended background, and untouched
  outside pixels. Recovery injection verified presentation-surface recreation,
  exact restored graph pixels, cached image/data/source, typed controls and ramps,
  clock/frame continuity, retained scheduled work, and second-loss exit code 70.
  These are logical-device tests, not physical driver-reset qualification.
- Release scheduling-limit stress passed. A separate overlay-enabled **60-second**
  traffic smoke passed with 60,000 updates and 60 pongs/status samples. RSS was
  42,516 -> 44,408 KiB, with a 44,088 KiB post-warmup baseline.
- The four-mode runner smoke used **three seconds per mode**, not eight hours.
  It is orchestration evidence only and does not qualify as a long test.

The ordinary renderer used for release checks has SHA-256
`703fc35633acb2be48647c5e33ad037d3d303eb36b3e7740630742154160230d`.
This exact candidate passed the isolated one-hour traffic check: **3600.005
seconds**, 3,600,000 updates at 1,000/second, 3,598 pongs/status samples, and frame
progress from 0 to 215,979. RSS was 37,384 -> 54,572 KiB, with a 43,252 KiB
30-second baseline (11.1 MiB growth, below the 128 MiB growth / 512 MiB absolute
limits). There were no queue drops or unexpected diagnostics. The completed
record is `build/soak/0.0.14-one-hour-current/run.json`.

The subsequent three **60-second** auxiliary checks passed against the same hash:
trivial rendering, 59 acknowledged watched reloads, and feedback with one resize.
Each received 60 pongs/status samples and continued rendering; peak RSS was below
45,000 KiB. Records are under `build/soak/0.0.14-short-current/`. These do not
claim eight-hour qualification, cross-platform verification, or physical resets.

The macOS x64 development archive was rebuilt with the same renderer SHA-256 as
the one-hour test. ZIP integrity, portable SHA-256 checksum, x86_64 architecture,
and system-only dynamic library links passed. It includes the locked dependency
audit, explicitly unreviewed, and was installed in the user Extensions directory
after backing up the prior installation. Installed class compilation, lifecycle,
typed example, general examples, and overlay/status checks passed. All 13 SCDoc
pages were indexed/rendered again without warnings or errors.

Four target metadata audits passed provenance/hash validation. Strict text-presence
checks pass for Linux/Windows x64 but intentionally fail for both macOS targets:
ten upstream full texts are still missing. This is not a license review or native
verification of the other platforms. See [NOTICE_AUDIT.md](NOTICE_AUDIT.md).

Policy update agreed 2026-09-24: current validation is one measured hour plus
short stress/recovery checks, matching the intended maximum session length.
The user will perform the eight-hour test as the last step on the completed
final-release candidate. Historical references below to a mandatory eight-hour
matrix describe the earlier policy and do not block ongoing development.

## Bounded scheduling and stability tooling — 0.0.13

Verified on macOS x64 / Metal; local test date 2026-09-23 (some logs use UTC
2026-09-24). This remains a development milestone, not final-release acceptance.

- Rust format, warnings-as-errors Clippy with all targets/features, ordinary
  debug/release builds, and all 43 Rust tests passed. The optional diagnostic
  feature also compiles and passes the suite. All 12 Python harness/audit tests pass.
- All 21 default installed SuperCollider checks pass, including the new
  `scheduling_limits_smoke`, the typed controls and restart regressions, all actual
  example blocks, audio analysis/streams, and SCDoc. All 12 class pages and the
  project guide indexed/rendered without SCDoc warnings or errors.
- The ordinary release binary passed live future-scheduler saturation by both
  command count and payload bytes, whole-packet admission/rejection, queue clearing
  and reuse, malformed-bundle no-partial-parse execution, nested timing inheritance,
  depth/lookahead rejection, and continued frames/pongs after recovery.
- Three separate **60-second** release runs passed: 59 acknowledged watched
  reloads, feedback with a resize at 30 seconds, and idle rendering. The initial
  traffic smoke sent 60,000 updates in 60 seconds. These are short harness smokes,
  not completed eight-hour gates.
- The pre-workaround ordinary release binary used for those short modes and the
  first queue stress has SHA-256
  `0dcc2f88ec91d117f8e3ae946c30281c83c7ae71588c315b30e5884f65185ac7`.

The real 60-minute/1000-update-per-second gate **failed after 410.03 seconds**
against that binary with unchanged 512 MiB absolute / 128 MiB post-warmup
RSS-growth bounds. RSS reached 185,124 KiB from a 53,088 KiB warmup baseline
(128.9 MiB growth). There were 410,027 sent updates, 410 pongs/status replies,
and frame progress from 6 to 24,562. The harness stopped its owned child and
recorded the failure under ignored `build/soak/0.0.13-traffic-60m.*`.
No eight-hour or cross-platform runtime pass is claimed here; the updated
workaround's one-hour traffic result is recorded below.

Short runs exposed visible-frame RSS growth even without OSC load. Autorelease
and explicit-GPU-wait comparisons did not resolve it; the independent native
Metal probe stayed small through 20,001 frames. One allocation-profile run failed
the health deadline because the profiler paused it. See
[MEMORY_INVESTIGATION.md](MEMORY_INVESTIGATION.md) for measured distinctions,
counter evidence, and the still-open investigation; do not treat a short `PASS`
as proof of long-duration memory stability.

The Metal multipass workaround was then implemented and the 43 Rust tests,
  all-target/all-feature Clippy, 12 Python tests, all 21 SC checks, and release
scheduler stress passed again. The updated ordinary release renderer has SHA-256
`f122da822fee4522a01019f35ab091ea984e5fb8f571c37289d135255c9aec09`.
An independent GPU readback regression verified exact pixels after 40 graph/
feedback frames with batched and split submissions. The reduced 20,001-frame
wgpu comparison stayed small with split submissions even with per-frame uploads
and new bind groups (13,240 → 15,764 KiB), while batched two-pass submissions
grew from 13,328 to 126,088 KiB.

Its first one-hour attempt overlapped other GPU checks and failed after 70.34
seconds on a reported queue drop (`0.0.13-metal-split-traffic-60m.*`), not on RSS.
This is retained as negative overload evidence; overlap is not proof of cause.
The isolated one-hour retry **passed** in 3600.00 seconds with 3,600,000 updates,
215,984 advancing frames, 3,598 pongs/status samples, no queue drops, and RSS
42,360 → 56,256 KiB (12.3 MiB growth from its 30-second baseline). It used the
updated binary hash above and is recorded under
`build/soak/0.0.13-metal-split-isolated-60m.*`.

This completes the one-hour traffic gate. The four eight-hour modes remain open.

The initial 0.0.13 local archive was checksum/architecture-checked and installed. Packaged
lifecycle, typed-control example, general examples, and SCDoc checks passed. An
initial packaged-example invocation supplied the examples directory where the
test expects the extension root; rerunning with the correct root passed without
a source change. This packaging evidence does not override the failed soak gate.

The rebuilt workaround archive has the same renderer hash as the isolated run,
passed ZIP/checksum/x86_64 and system-library checks, and was installed in the
user extension directory. Installed SCDoc was indexed/rendered again without
warnings or errors. No new packaged-renderer process was started during the
isolated gate; the earlier all-21 run used the updated source/debug renderer.

The offline notice-audit addition brings the Python suite to **12 passing tests**.
The strict macOS x64 audit correctly returned failure after writing its inventory:
132 dependencies, 22 without bundled notice text. This is a documented remaining
release task, not a passed licensing review; see [NOTICE_AUDIT.md](NOTICE_AUDIT.md).

## Typed controls — 0.0.12

Verified 2026-09-23 on macOS x64 with the Metal backend.

- Rust format, Clippy with warnings denied, debug/release builds, and 36 unit
  tests passed. New cases cover WGSL/GLSL reflection, struct offsets, mat3 padding,
  exact integer blobs, malformed payloads, type mismatches, smoothstep, extreme
  Float32 endpoints, retargeting, and preservation across changed layouts.
- The installed SC classes pass all 20 default checks in
  `python3 tools/run_sc_checks.py`. These include the existing lifecycle, timing,
  Pattern, texture/feedback, window, graph, compatibility, audio-analysis and stream
  checks, plus typed-control unit/live tests, restart regression, and actual
  runnable example/guide blocks.
- Live typed-control tests round-trip all ten types, signed extrema and the full
  unsigned range; verify independent instances, timestamped updates, scalar/vector
  glides, and retained values after explicit reload.
- A non-selected resource reports an invalid watched edit without losing its
  controls, then recovers from a valid type-changing edit with refreshed reflection.
- Restart tests verify that old shutdown timers cannot terminate replacement
  children, boot requests during quit are deferred, old Shader handles are freed,
  and clock estimates reset. Consecutive examples exposed the stale-timer bug;
  it was corrected and the sequence rerun successfully.
- A separate 32-event timing benchmark on the local display measured application
  error mean 10.46 ms, p95 11.66 ms, and maximum 13.01 ms. This is not GPU scan-out
  timing or evidence for another display/refresh rate.
- SCDoc reindexed the installed extension and rendered all 12 class pages and the
  project guide without warnings or errors.
- The owner-prefixed macOS x64 development archive passed checksum and zip-integrity
  checks; its Mach-O contains only x86_64 and links only system libraries/frameworks.
  The complete bundle was installed; lifecycle, packaged example, and installed
  SCDoc checks passed against it. Nothing was uploaded to GitHub.

The initial lifecycle regression failed because it hard-coded version 0.0.11;
the test now compares against `VERSION`. Initial unsigned tests also exposed
sclang's signed 32-bit Integer boundary; upper-half uint values now use exact
integral Floats and binary OSC payloads. Both regressions were rerun successfully.

These are bounded development checks, not the mandatory 60-minute/8-hour soak
evidence. No Linux, Windows, Apple Silicon, hosted-CI, signing, or public-release
result is implied. See [STATUS.md](STATUS.md) for remaining acceptance gates.

## Historical evidence

Verified 2026-08-20 on macOS 15.7.2, x86_64, using the Metal backend on an AMD Radeon RX 580.

## Completed checks

- `cargo build --release --locked`: passed.
- `cargo fmt -- --check`: passed after formatting.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 5 protocol tests passed.
- Real 1280×720 Metal window and built-in WGSL animation: passed.
- Versioned ready/ping/pong/uniform/quit round trip: passed.
- Malformed OSC datagram: returned `E_PROTOCOL`; renderer remained alive.
- Ten-second traffic smoke at 1000 uniform updates/s: 10,000 updates, 10 pong replies, one expected malformed-packet diagnostic; RSS was 41.2 MiB at the start and final sample.
- Separate steady-state process sample after 16 seconds: 5.0% CPU and 42,384 KiB RSS. This is reasonable for the trivial continuously redrawn shader on the test machine, but is not a cross-platform performance result.
- Invalid WGSL fixture: returned `E_SHADER_VALIDATE`, retained a live window using the reported built-in fallback, accepted another 200 uniform updates, and quit cleanly.

## Milestone B checks

Verified 2026-09-12 on the same macOS/Metal system.

- `cargo fmt -- --check`: passed.
- `cargo test --locked`: six protocol tests passed, including `/scshader/v1/status` parsing.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `ShaderServer` class-library compilation: passed.
- `ShaderServer` lifecycle smoke: launched the local renderer, completed v1 ready handshake, received a ping/pong and a status reply, then sent a clean quit command. The renderer reported Metal and the AMD Radeon RX 580.
- `ShaderServer` process-exit smoke: a deliberately failing child changed the controller out of booting/running state and reported `E_BOOT`.

The lifecycle smoke uses an explicitly configured development executable. Locating a packaged executable is implemented but release packaging itself remains a later milestone.

## Milestone C checks

Verified 2026-09-12 on the same macOS/Metal system.

- `cargo test --locked`: eight protocol tests passed, including shader create parsing and positive resource-ID validation.
- `cargo clippy --all-targets -- -D warnings`: passed.
- SuperCollider class-library compilation: passed with `ShaderServer`, `ShaderDef`, and `Shader` installed.
- Shader lifecycle smoke: created a file-backed ShaderDef/Shader resource, confirmed the compact `amount: float` reflection, triggered automatic hot reload by touching the unchanged WGSL fixture, requested an explicit reload, and freed the resource.
- Invalid WGSL creation returned `E_SHADER_COMPILE` with a renderer diagnostic while the preceding active shader remained live, then the renderer quit cleanly.

This proves the current one-active-pipeline lifecycle. It does not establish general WGSL reflection, simultaneous shader passes, vector/matrix uniforms, or cross-platform behavior.

## Milestone D checks

Verified 2026-09-12 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings`: passed. The 13 tests cover protocol parsing, OSC timetag conversion, invalid historical timetags, and timestamp/packet ordering in the renderer scheduler.
- SuperCollider class-library compilation and SCDoc rendering: passed for `ShaderServer`, `ShaderDef`, `Shader`, and the SCShader guide.
- Timing smoke: took four ping/pong clock samples, observed a future bundle in `scheduledEventCount`, applied a bundled uniform-plus-marker event, and verified a deliberately past bundle is applied immediately with `E_LATE_EVENT`.
- Timing benchmark: 32 regularly scheduled uniform-plus-marker bundles measured a mean renderer event-application error of 10.08 ms, median 10.06 ms, p95 10.28 ms, p99 10.73 ms, and maximum 10.73 ms. These are renderer-clock application measurements on the current Metal/vsync setup, not GPU presentation, display scan-out, sample-accurate visual timing, or a cross-machine performance claim.
- Existing controller lifecycle and shader-resource lifecycle smokes: passed. The invalid-WGSL diagnostic in the latter is expected and preserves the last valid pipeline.

The supplied benchmark should also be run on a 120 Hz display where available. No 120 Hz or cross-platform result has been recorded here.

## Milestone E checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings`: passed. The 17 tests include startup window argument validation plus parsing for window resize and boolean controls.
- SuperCollider class-library compilation: passed with `ShaderWindow` installed alongside the controller and shader-resource classes.
- SCDoc indexing and rendering: passed without warnings for `ShaderServer`, `ShaderWindow`, `ShaderDef`, `Shader`, and the SCShader guide.
- Native-window smoke: launched a 640×360 Metal renderer window, applied runtime title, logical position, borderless, fullscreen-off, VSync, cursor, and 800×450 resize commands, then received positive logical/framebuffer metrics and a clean quit.
- The same smoke enabled input and verified the renderer-to-language mouse, button, key, resize, and focus callback routing with representative protocol events. It does not simulate physical user input, multi-monitor fullscreen, non-1.0 HiDPI scaling, or another operating system.
- Existing lifecycle regression smoke: passed with the final 0.0.5 renderer, including ready, ping/pong, status, and clean process shutdown.

The current implementation supports one renderer-owned window per `ShaderServer`. Startup monitor selection is implemented for fullscreen creation, but independent multiple windows and runtime monitor switching remain unimplemented.

## Milestone F checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings`: passed. The renderer version is 0.0.6.
- SuperCollider class-library compilation: passed with the registered `\\shader` Event type present in `Event.eventTypes`.
- SCDoc indexing and rendering: passed without warnings for all five SCShader help pages.
- Pattern smoke: created a live named shader and its `ShaderWindow`, used a finite `Pbind` to schedule three `amount` updates at 150 ms latency, observed a future renderer queue entry, and confirmed the final language-side value. It then sent an event using that resource's `shaderID`, confirming explicit-ID routing.
- Existing lifecycle regression smoke: passed with the final 0.0.6 renderer, including ready, ping/pong, status, and clean process shutdown.
- The smoke proves language-side event resolution and renderer bundle scheduling on this machine. It does not measure display presentation, establish strict first-event timing for an asynchronously created shader, or implement a separate `Pshader` wrapper.

## Milestone G checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings`: passed with the static PNG/JPEG/PNM decoder enabled.
- Texture/feedback smoke: decoded the checked-in public-domain 2×2 PNM fixture, bound it as `source`, enabled `previous` feedback at 0.92, resized the renderer window to recreate ping-pong targets, observed one texture in status, then freed it and observed zero textures without a renderer error.

## Milestone H checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --locked`: passed with renderer version 0.0.8. The 17 existing renderer unit tests remain green because ShaderBus batching and ShaderAnalysis are language/audio-server layers that retain the v1 renderer ABI.
- SuperCollider class-library compilation and SCDoc indexing/rendering: passed without warnings for all eight SCShader pages, including new `ShaderBus` and `ShaderAnalysis` help.
- ShaderBus mapping smoke: created a live renderer Shader, mapped a zero-smoothing bus to `amount`, sent 0.73, and confirmed the coalesced mapping updated the live Shader with no renderer error before clean shutdown.
- Analysis smoke: booted an isolated `scsynth` on port 57310, wrote a private audio bus from a 440 Hz sine source, and confirmed amplitude, pitch, confidence, centroid, flatness, onset, and zero-crossing replies. The initial run exposed source/analyser node ordering; placing the analysis Synth at the default-group tail fixed it and the repeatable smoke passed.

This proves the controlled-rate local bridge and language-side 60 Hz smoothing, not GPU-side arbitrary-uniform interpolation, an audio-server polling bridge, long-duration analysis stability, or cross-platform behavior.

## Milestone I checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --locked`: passed with renderer version 0.0.9. The suite now has 20 tests, including big-endian float32 blob parsing, malformed-blob rejection, and rejection of unsupported buffer bindings.
- SuperCollider class-library compilation and SCDoc indexing/rendering: passed without warnings for all 11 SCShader pages, including `ShaderBuffer`, `ShaderFFTTexture`, and `ShaderWaveformTexture`.
- Buffer smoke: created a 128-float resource, transferred a 0 through 1 ramp as an OSC blob, bound it at `spectrum`, observed `bufferCount == 1`, confirmed the language copy, freed it, and observed `bufferCount == 0` with no renderer error.
- Stream smoke: ran a private 440 Hz sine source on a private `scsynth` audio bus, filled a 512-bin FFT-magnitude buffer and a 64-point recent-waveform buffer, bound each through `Shader.setTexture(\\spectrum, ...)`, observed nonzero data, and shut down both renderer and audio server cleanly. The initial extraction used an invalid demand trigger and emitted `Unpack1FFT` buffer warnings; it was replaced by FFT-window triggers with demand groups capped at 32 inputs before the successful run.
- Lifecycle regression: passed against 0.0.9, including the extended status reply with `bufferCount`.

The first data path intentionally transfers complete moderate-size buffers and was not benchmarked over an hour. At the verified 1024-point FFT / 20 Hz setting, 512 Float32 values are about 2 KiB of payload per direction per update (roughly 40 KiB/s before OSC framing), plus a 64-point waveform stream of about 5 KiB/s. This is well below a shared-memory threshold for the tested configuration, so shared memory remains deferred until a larger-stream profile demonstrates a bottleneck. The implementation does not provide FFT phase, mel/chroma, general storage buffers, GPU readback, or cross-platform measurements.

## Milestone J checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --locked`: passed with renderer version 0.0.10. The suite has 22 tests, including valid linear graph parsing and duplicate-node rejection.
- SuperCollider class-library compilation and SCDoc indexing/rendering: passed without warnings for all 12 SCShader pages, including `ShaderGraph`.
- Graph smoke: created two independent live WGSL Shader resources, committed their validated linear chain, rendered it in the local Metal renderer, observed `shaderCount == 2`, and shut down cleanly.
- Lifecycle regression: passed against 0.0.10 with a ready reply, ping/pong, extended status reply, and clean shutdown. `shaderCount` intentionally excludes the renderer's built-in fallback, therefore a newly booted renderer reports zero client Shader resources.

The graph is a deliberately bounded first implementation: a single connected acyclic linear fullscreen chain with an optional terminal prior-frame feedback edge. General DAG routing, branches, typed intermediate textures, independently-sized targets, and arbitrary feedback cycles are not implemented.

## Milestone K checks

Verified 2026-09-13 on the same macOS/Metal system.

- `cargo fmt -- --check`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, and `cargo build --locked`: passed with renderer version 0.0.11. The suite has 24 tests, including protocol handling for explicit GLSL/Shadertoy source types and rejection of unknown types.
- Compatibility smoke: the live renderer compiled and rendered an original raw GLSL 4.50 fragment plus three original Shadertoy-style sources. The corpus exercises time/frame values, resolution, mouse/date declarations, `iChannel0` sampling, bounded loops, derivatives, and common math on Metal without a renderer error.
- SuperCollider compatibility control: `ShaderDef(..., \glsl)` and `ShaderToy(...)` both reached creation acknowledgement with separate resource IDs, compact `amount` reflection, status count four, and clean free/quit.

WGSL remains the native path. The compatibility layer uses wgpu's Naga GLSL frontend, not a regex rewrite; it is limited to the documented GLSL 4.50 fragment contract and a `mainImage` Shadertoy wrapper. The smoke does not establish arbitrary historical GLSL syntax, independent Shadertoy channels, source assets from Shadertoy, physical input simulation, other GPU backends, or cross-platform compatibility.

## Milestone L checks

Verified 2026-09-13 on the same macOS/Metal x64 system.

- The local `tools/package_macos.sh x64` syntax check and release build passed. It created `SCShader-0.0.11-macos-x64.zip` plus a SHA-256 file, containing the extension classes, help, original fixtures, Quark metadata, licensing files, and one x64 renderer executable.
- The archive listing was inspected for the expected `SCShader/renderer/scshader-renderer` layout. The lifecycle smoke then launched that staged executable—not the development binary—and completed the v1 ready, ping/pong, status, and clean-quit round trip.
- `SCShader.quark` declares the source package and has no automatic executable download. `docs/RELEASE.md` describes explicit archive installation, checksum use, and the target-hardware release checklist.
- `.github/workflows/ci.yml` defines Rust format/Clippy/unit tests, a Linux SuperCollider class check, architecture-explicit macOS x64 and arm64 builds, Linux x64, Windows x64, checksummed artifacts prefixed from `github.repository_owner`, and a tag-only release job.

No GitHub release was created in this session. Linux, Windows, macOS arm64, codesigning/notarization, CI execution, GPU runtime behavior outside this local macOS x64 target, and the mandatory long soak remain unverified.

## Long test status

The mandatory 60-minute harness is implemented but the full duration was not run in this implementation session. Run:

```sh
python3 tools/soak_uniforms.py --minutes 60 --rate 1000 --pid RENDERER_PID --csv soak.csv
```

Passing the short smoke does not substitute for the documented 60-minute acceptance run or the later eight-hour soak matrix. Windows, Linux/Wayland, Linux/X11, Apple Silicon, multi-monitor, HiDPI, and cross-platform release packaging are also still unverified.
