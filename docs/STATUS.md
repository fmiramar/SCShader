# Implementation checkpoint

Updated 2026-09-28. Current development version: **0.0.17**. Not a final v0.1.0 release.

Windows checkpoint (2026-09-28): development 0.0.17 fixes the reproduced IDE
startup failure by selecting FXC explicitly, avoiding an incompatible DXC DLL
in SuperCollider's working directory. Duplicate ready replies no longer replay
window configuration. All 25 SC checks have passing evidence across the candidate
suite and a focused asynchronous-example recheck. All 18 help demos and 13 SCDoc
pages pass from a candidate path containing spaces and non-ASCII characters,
launched from SuperCollider's directory. Windows test trees now use kill-on-close
Job Objects; Python has 61 passes and one privilege-dependent skip.
Rust formatting, Clippy and all 59 Rust tests pass. Both GPUs pass four ten-second
D3D12 modes. Scheduling bounds, 80 resource cycles, two-second floods and exact
pixel logical-device recovery pass on NVIDIA. The flood runner now sends health
probes before catch-up bursts, retaining the offered load and health thresholds;
the earlier missing-reply failures remain recorded. The user installed the
verified 0.0.17 ZIP in the normal Extensions folder, with 0.0.16 backed up outside
Extensions. All 73 installed files match the extracted package. Installed class
compile, handshake, lifecycle, bus startup, four runnable examples and all 13
SCDoc pages pass from SuperCollider's working directory. Recompile the user's
open IDE class library before using the update. Manual display/input checks,
other native platforms and separately scheduled long runs remain outside this
automated Windows result.
Follow-up: native window interaction checks now pass on both GPUs, including four
resizes with watched reload, fullscreen/border toggles, OS minimize/restore and
clean close at 125% scale (one active display, 1920x1080 fullscreen). The manual
IDE interaction example is validated. The user reports window/input actions
worked; unchanged appearance after R is expected and its reload acknowledgement
is being confirmed separately. The user confirmed `ShaderFFTTexture` worked
perfectly in the IDE. `ShaderAnalysis` listening, sleep/wake and mixed-monitor
observations remain unreported; see [the checklist](WINDOWS_INTERACTION_CHECKS.md).
The user selected **Linux next after step 1**, with Mac regression deferred.
The native Linux x64 checkpoint now passes on the receiving CachyOS desktop;
see [the Linux result](platform-results/2026-09-28-linux-x64.md) and
[Linux setup](LINUX.md). Native Xorg, compositor-driven interaction, and Mac
regression remain open.
The latest source handoff is `fmiramar-SCShader-0.0.17-source-linux-handoff-2026-09-28-r5.zip`;
verify it using the adjacent `.sha256` file.
See [the current Windows record](platform-results/2026-09-28-windows-x64.md),
[the preserved 0.0.16 results](platform-results/2026-09-27-windows-x64.md), and
[Windows setup](WINDOWS.md). P1 remains open; no long-run claim is added.

The [implementation report](IMPLEMENTATION_REPORT.md) explains the architecture,
technology choices, completed milestones, remaining work, and why it matters.

Next milestone (2026-09-25 decision): **native desktop compatibility first**.
Prioritize Windows x64, Linux x64, and macOS Apple Silicon builds, installations,
short GPU/SC checks, and platform fixes; retain Intel Mac regression coverage.
See [PLATFORM_MILESTONE.md](PLATFORM_MILESTONE.md) and its per-platform agentic plans.
This ordering comes before further extended Mac-only stress work and final long
tests. Native target results are still pending. [START_HERE.md](../START_HERE.md)
explains the portable source handoff; [the Git audit](HANDOFF_GIT_AUDIT.md) records
that current work is not fully committed. Nothing is published by this handoff.

Installed-example correction (2026-09-25): user reports exposed placeholder paths
and prerequisite-only snippets that earlier checks missed. The guide, ten class
pages, README, and example files 01–04 now use installed assets and fresh
controllers. All 18 help/README demos and their cleanup blocks pass verbatim
execution (36 blocks), including analysis/FFT audio and graph setup. All four
example-file setup blocks pass without an active document path. SCDoc rendered
all 13 pages again without warnings/errors. The native renderer is unchanged;
this fixes documentation/assets/test coverage, not a new renderer version.

Short resource-churn and incoming-flood checks exposed unbounded diagnostic
replies/logging in 0.0.15. Version 0.0.16 shares a burst-20, 20/second budget
across receive/render threads and senders, caps UTF-8 message bodies, and appends
`suppressedDiagnostics` to status. Critical GPU loss/fatal diagnostics bypass the
limit; ordinary soaks fail on suppression so hidden errors cannot count as passes.
Eighty resource cycles returned counts and owned-texture bytes to baseline.
Both two-second error floods fell from roughly 4,000 diagnostics to 59, while
frames and replies remained live. Valid bundles offered 511,616 updates without
observed incoming-queue drops; deterministic unit tests cover full-queue rejection
and recovery separately. See [STABILITY.md](STABILITY.md).

Windows/Linux preparation is implemented: explicit GPU/display selection,
Wayland-compatible default fullscreen, backend/display retention during recovery,
portable executable discovery and test paths, OS/CPU/Rust/binary-format packaging
guards, and native-test instructions. Windows x64 MSVC and Linux x64 GNU pass
cross-target compilation of all targets/features on macOS. Linux x64 now also
has a native linked package, installation, Vulkan Wayland/Xwayland checks on both
GPUs, SC/help checks, bounded stress, and logical recovery evidence; see the
dated result record. Native Xorg, other distributions, and native macOS arm64
runtime testing remain open; see [PLATFORMS.md](PLATFORMS.md).

Local 0.0.16 regression checks pass: 55 Rust tests, 49 Python tests,
warnings-as-errors Clippy, all 24 installed SC checks, 13 warning-free rendered
help pages, four three-second Metal modes, live future-scheduler stress, and
exact-pixel logical-device recovery. The macOS x64 development package is
installed locally, with the prior installation backed up. Native Windows/Linux
linked builds, GPU behavior and hosted CI are not yet verified. No new
long-duration result is claimed.

Source-handoff checks (2026-09-25): the Python count now includes eight portable
archive tests. A fresh source-ZIP extraction passed all 49 Python tests, Rust
formatting, and locked/offline Cargo manifest inspection. The inventory covers
all 136 current project source files plus the original implementation plan;
generated artifacts and Git metadata are excluded. No renderer/class code changed
for this handoff, and no native platform or long-run result was added.

Implemented and covered by local tests: renderer/server lifecycle, timed OSC and
Patterns, one window/input events, static images/feedback, visual buses/audio
analysis, Float32/FFT/waveform streams, linear multipass graphs, constrained
GLSL/Shadertoy compatibility, and architecture-specific packaging scripts.

The latest milestone replaces the amount-only control restriction with real
Naga reflection for a bounded flat uniform block, all ten planned scalar/vector/
matrix types, typed set/get, and renderer-side step/linear/smooth interpolation.
Reload watches all live shaders and preserves matching controls and active ramps.
Renderer restarts now guard against stale shutdown timers, defer overlapping boot
requests, invalidate old Shader handles, and reset the renderer-clock estimate.
Future work now has command-count, payload-byte, and lookahead bounds, whole-packet
validation/admission, atomic bundle dispatch, overload counters, and explicit
schedule clearing. The strict soak harness records measured duration, fresh replies,
frame progress, and process memory; short smokes do not satisfy long-run gates.
Version 0.0.14 adds the optional stats overlay, one controlled GPU-device restart
with CPU snapshots and explicit feedback-history reset, surface recreation, image
placeholders, and expanded status. Native Metal pixel readback covers overlay
drawing/isolation and recovery; logical-device injection is not a physical reset.
See [RECOVERY.md](RECOVERY.md) for the contract and test limits.
See [CONTROLS.md](CONTROLS.md) and [VERIFICATION.md](VERIFICATION.md) for contracts
and measured evidence. Historical milestone checklists are not evidence that
every final-release requirement has been satisfied.

## Remaining final-release work

Validation policy updated 2026-09-24: sessions are intended to last at most one
hour. Current validation uses one hour on the exact candidate plus short stress/
recovery checks. The user owns the eight-hour sign-off as the final release step;
do not launch it automatically or block ongoing implementation on that duration.

The previous 0.0.14 one-hour run passed on 2026-09-24 against the pinned
`703fc35633ac...` binary: 3,600,000 updates in 3600.005 seconds, 215,979 frames,
3,598 pongs/status samples, no queue drops, and 11.1 MiB post-warmup RSS growth.
Evidence is in `build/soak/0.0.14-one-hour-current/run.json`.
All three subsequent one-minute auxiliary modes also passed under
`build/soak/0.0.14-short-current/`: trivial, 59 watched reloads, and feedback/resize.
The local scripts save all results without continual agent polling; keep progress
messages/checks minimal to conserve usage. No eight-hour test is queued.

The matching 0.0.14 macOS x64 development archive remains preserved locally.
Its renderer retains the tested SHA-256. Packaged lifecycle, examples, overlay/
status, and all 13 help-page checks passed. Dependency inventories are included
as unreviewed evidence, not final notice clearance. Nothing has been published.
Versions 0.0.15 and 0.0.16 change the renderer and do not inherit this one-hour result.
No new one-hour or eight-hour test is automatically launched for this milestone.

- Keep the Metal per-pass-submission workaround covered by the remaining long-run gates.
  An offscreen reproduction isolated growth to multipass submissions on the
  tested device; single-pass and split-submission comparisons stayed small, and
  GPU pixel readback verified feedback/output ordering. See [MEMORY_INVESTIGATION.md](MEMORY_INVESTIGATION.md).
- The isolated 60-minute/1000-update-per-second traffic test now passes: 3,600,000
  updates in 3600.00 seconds, 215,978 frames of progress, no queue drops, and RSS
  42,360 -> 56,256 KiB (12.3 MiB growth from the 30-second baseline). Evidence is
  `build/soak/0.0.13-metal-split-isolated-60m.*`, using renderer SHA
  `f122da822fee4522a01019f35ab091ea984e5fb8f571c37289d135255c9aec09`.
- Retain the completed 0.0.14 one-hour traffic and short trivial/reload/feedback,
  overload, and recovery evidence. Reserve the user-run eight-hour check for the
  completed final candidate. Record memory, responsiveness, crashes, and frame
  progress rather than elapsed time alone.
  The pre-workaround 0.0.13 one-hour attempt failed after 410.03 seconds: post-warmup RSS growth
  reached 128.9 MiB. Evidence is under `build/soak/0.0.13-traffic-60m.*`.
  A post-workaround run overlapped other GPU tests and failed on a queue drop;
  that remains separate negative evidence and is not used as the isolated result.
- Continue stress coverage beyond the completed future-queue saturation/recovery,
  bounded resource churn, incoming-load, and diagnostic-rate checks: prolonged
  resize/reload, larger resource workloads, and native incoming saturation.
  The current valid-update flood did not saturate the incoming queue; its full
  admission/drop behavior is covered deterministically, not as a live GPU result.
- Qualify the implemented overlay and recovery path on real driver/device failures
  and other backends. Repeat acceptance if the renderer changes; the completed
  0.0.14 one-hour result qualifies only the exact binary recorded above.
- Build/run the native Linux, Windows, and macOS arm64 packages and hosted CI;
  check multi-monitor/HiDPI and display-rate timing on available hardware.
- Generate and review complete third-party license notices from the locked
  dependency graph, and decide signing/notarization before general distribution.
  The offline [notice audit](NOTICE_AUDIT.md) now validates pinned upstream
  supplements and is included in development packages. macOS text gaps fell from
  22 to ten; Linux/Windows x64 have none, but no target claims completed review.
  Tagged package builds fail while required notice texts remain missing.
- Only after the acceptance gates: prepare the v0.1.0 tag/release with explicit
  publication authorization. No public release has been made in this session.

Scene restoration, independent multiple windows, general DAGs, richer texture
bindings, video/camera input, compute, and shared memory are later scope. They are
not silently treated as completed by the current narrow implementations.
