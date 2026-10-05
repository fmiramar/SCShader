# Implementation checkpoint

Updated 2026-10-04. Current development version: **0.0.18**. Not a final v0.1.0 release.

Desktop RTX 3060 follow-up: the exact 0.0.18 D3D12 renderer passes one hour of
traffic, then fails its eight-hour attempt after 2,482.785 seconds on
`E_QUEUE_FULL` (one dropped continuous update). The RTX desktop long-run issue
remains open; do not repeat the eight-hour soak on this unchanged binary before
investigating the queue-service failure. See the
[RTX 3060 results](platform-results/2026-10-04-windows-rtx3060-soaks.md).

Laptop eight-hour follow-up: **Intel UHD 630 and GTX 1050 Ti Max-Q both pass**
28,800 seconds of D3D12 traffic on the unchanged laptop renderer and corrected
sender. They sent 28,799,902 / 28,799,804 updates, with zero sampled renderer drops,
bounded memory and maximum sender lag of 19.310 / 22.435 ms. Independent evaluation
confirmed each adapter's eight-hour qualification; the local supervisor's overall
flag was stale and is explained in the result. These are traffic-only passes.
Desktop RTX failures, other modes and final release gates remain separate. See the
[eight-hour results](platform-results/2026-10-04-windows-laptop-eight-hour.md).

Laptop one-hour follow-up: **Intel UHD 630 and GTX 1050 Ti Max-Q both pass**
3,600 seconds of D3D12 traffic with the corrected sender and unchanged renderer.
Maximum sender lag was 20.202/18.969 ms, with 19/20 sender-skipped ticks and zero
sampled renderer queue drops. The old stalls did not reproduce. Eight-hour
qualification and the desktop RTX cause remain open. See the
[one-hour results](platform-results/2026-10-03-windows-laptop-one-hour.md).

Desktop/laptop comparison (2026-10-03): the RTX failure is a renderer queue-service
gap; both laptop failures tripped the sender-lag guard. Python 3.12's coarse Windows
monotonic clock was confirmed to batch 1 kHz traffic into bursts of 16. Switching
the soak clock to `perf_counter()` reduced maximum bursts to four in paired
one-minute diagnostics on each laptop GPU. Both adapters then passed all four
ten-second modes with the corrected harness. The original >250 ms stalls did not
reproduce, and desktop display routing remains a hypothesis requiring its hardware.
See the [comparison and remaining checks](platform-results/2026-10-03-windows-soak-comparison.md).

The subsequent unattended ten-minute traffic diagnostics pass on both laptop
GPUs with zero skipped updates or sampled queue drops. Maximum sender lag was
15.847 ms (Intel) and 14.087 ms (NVIDIA); the no-renderer control completed too.
The old stall did not reproduce. Windows system profiling was unavailable, so
application traces supply the new timing evidence; long-duration gates remain open.

Windows hybrid-GPU soak follow-up (2026-10-03): the user's Intel UHD 630 and
NVIDIA GTX 1050 Ti Max-Q each passed four ten-second D3D12 preflight modes. The
eight-hour traffic attempts then failed on the harness's 250 ms sender-lag guard:
Intel after 20,199.89 seconds (maximum lag 311.996 ms), NVIDIA after 15.781 seconds
(436.999 ms). Neither qualifies as an eight-hour check. Renderer frames continued,
sampled queue drops remained zero, and RSS stayed within limits before each abort;
the sender scheduling cause remains unresolved. See the
[laptop soak record](platform-results/2026-10-03-windows-uhd630-gtx1050ti-soaks.md).

Windows soak investigation (2026-10-03): three earlier RTX 3060 / D3D12
eight-hour runs failed after 15–19 minutes on incoming-queue overflow. Optional
timing tracing reproduced a 256 ms command-service gap while the sender stayed
on pace. A repeat four-mode short check passed, then the user paused to switch
models and adjust display power. Follow-up eight-hour tests used the same packaged
renderer on both GPUs, with AC display-off and sleep set to Never. RTX 3060/DX12
failed again on `E_QUEUE_FULL` after 107 seconds; RX 580/DX12 completed 28,800
seconds with 28.8 million updates and no queue drops. The RTX long-run failure
remains open. The pass qualifies that packaged binary and traffic mode only.
Pre-commit review corrected the test sender's high-rate catch-up cap; Rust
formatting/Clippy, 61 Rust tests and all 132 Python tests pass. See
[the investigation](platform-results/2026-10-03-windows-soak.md).

Distribution follow-up: notice assembly now has explicit license choices,
hash-pinned provenance/recovery records and zero missing texts for all packaging
targets. Packages include full GPL text, dependency/Rust standard-library notices
and a matching vendored source archive linked by binary/source hashes. See
[NOTICE_REVIEW.md](NOTICE_REVIEW.md) for dispatch and Apple SDK provenance caveats;
this is not legal certification. At the user's request, runtime provisioning/
clean-machine and signing/notarization qualification are deferred to a later stable
release; current users install the exact documented prerequisites themselves.
No runtime code, dependency pins or version metadata changed in this follow-up.
Verification: all five notice-review gates, Windows paired ZIP checks and an
offline vendored-source rebuild pass. Python: 125 tests discovered, 124 pass and
one sandbox privilege skip on the final rerun; all 125 passed before sandboxing.
See [VERIFICATION.md](VERIFICATION.md) for exact artifact hashes and limitations.

Windows desktop follow-up: RTX 3060 and RX 580 each pass four ten-second D3D12
modes and 21 native window stages, including moves/fullscreen/restoration across
three 1920x1080 displays. All currently use 100% scale. The pinned native package
is installed with 83 matching files. All 59 Rust tests, strict Clippy, and 116
Python tests pass. All 26 installed SC checks and 18 help demos pass in a dedicated
class configuration; the normal IDE class context passes compile/lifecycle checks.
An existing `adclib`/standalone-`Document` error is preserved separately.
RX 580 resource/flood stress, exact-pixel logical recovery, default scheduling
and spaces/non-ASCII package checks pass. The monitor harness now verifies native
geometry/DPI and GPU allocation on each display. The user confirms physical
window/input checks pass across all three displays. Listening, mixed DPI,
sleep/wake and long release gates remain open. See the
[desktop record](platform-results/2026-09-30-windows-dual-gpu.md).

Multi-computer development uses the public GitHub repository's `main` branch.
Reviewed source, test, and documentation commits are authorized for routine push
to `origin/main`; each computer authenticates locally before pushing. The macOS
follow-up is in development checkpoint `3e8f022` on `main`. Release tags, binary
release uploads, hosted workflow dispatches, and final release publication still
require separate authorization. See [START_HERE.md](../START_HERE.md).

macOS interaction follow-up: all 19 native window/input stages and the user's
physical mouse/key/focus/fullscreen/reload checks pass. A reported Escape concern
was resolved by testing it while fullscreen. A genuine normal-close issue was
reproduced and fixed: `ShaderServer` now accepts exit 0 after a successful boot,
while still rejecting early exits. All 26 installed SC checks and 104 Python
tests (108 discovered, four Windows skips) pass. The renderer hash is unchanged.
The macOS packager now excludes Finder/resource metadata after two `.DS_Store`
files were found in a development archive. Analysis/FFT listening and sleep/wake
remain pending; the user has only the built-in display. Intel/shared-platform
regression remains open. That September 30 strict recheck found ten missing texts;
the local ad-hoc binary verifies but fails Gatekeeper assessment. See the
[interaction procedure](MACOS_INTERACTION_CHECKS.md),
[Apple Silicon record](platform-results/2026-09-30-macos-arm64.md) and
[distribution handoff](RELEASE.md#remaining-distribution-gates--2026-09-30).

Deferred runtime observation: while switching between SCIDE and the renderer GUI
during the first `ShaderAnalysis` example, the user saw repeated `E_LATE_EVENT`
warnings, roughly 0.8–1.02 seconds late and applied immediately. No crash was
reported. Preserve and reproduce this before treating analysis interaction as
clean. The user reports that `ShaderFFTTexture` ran smoothly and showed only
normal renderer-ready messages; see the ignored evidence record under
`build/platform-tests/macos-arm64-analysis-late-events-20260930/`.

macOS Apple Silicon resumed 2026-09-30: the pinned Rust 1.97.1 toolchain,
locked dependencies, native arm64 production package, and recovery-hook build
are ready. Rust formatting, strict Clippy, 59 Rust tests, and 91 Python tests
(95 discovered, four Windows-only skips) pass. All 82 installed files match the
package. Four ten-second Metal modes on Apple M5 and all 18 help demos pass.
With the agent sandbox restriction lifted, all 25 installed SC checks were rerun
successfully with complete console evidence. Scheduling, 80 resource cycles,
short floods and logical-device recovery with exact GPU pixels also pass.
The old feedback mode performed zero resizes in ten seconds; the corrected
harness now confirms three actual resizes at 2x Retina scale. Continuous traffic
did not saturate the incoming queue; this is load evidence. Local UDP, CPU feature
queries and `sclang` now work here. Additional native display/input/lifecycle
coverage, Intel regression and long-release gates remain open.
See the [Apple Silicon result](platform-results/2026-09-30-macos-arm64.md).

Linux release scope: CachyOS x86-64 GNU under Hyprland with Vulkan. Xwayland is
covered only as an in-session compatibility path; other distributions,
compositors, native Xorg, arm64 and GL are deferred. Windows x64 and macOS
Apple Silicon/Intel remain in the project release plan.

Linux resume: code inspection found that the renderer discarded synchronous
`winit` resize results, which Wayland may apply without a later resize event.
0.0.18 applies that physical size through the existing surface/input path and
avoids recreating GPU targets for unchanged sizes. A Hyprland interaction runner
records failure stages and checks GPU allocation changes as well as native window
metrics. Static checks pass: 59 Rust tests; 84 Python tests and four Windows-only
skips (88 discovered). Both NVIDIA and Intel now pass the full native Wayland
compositor-resize and Xwayland OSC-resize interaction suites: four resize/reload
cycles, GPU allocation changes, fullscreen, decoration flags and clean close.
Earlier strict Wayland OSC requests were refused, so the accepted Wayland scope
uses compositor-driven resizing. The Xwayland checker now handles logical DPI
versus framebuffer dimensions and both Hyprland scaling-option response formats.
After correcting the wrapper's display environment, all 16 short GPU acceptance
combinations and both 25-check installed SC suites pass. After giving the Pattern
demo an explicit 150 ms lead time, all 18 help demos and SCDoc pass on both
displays. Short scheduling checks pass. The stabilized resource/flood rerun passes
80 resource cycles, protocol/command/continuous floods with bounded diagnostics,
and queue saturation. Logical Vulkan recovery and the path-with-spaces/non-ASCII
SC checks also pass on Intel. The candidate package is installed with 82 verified
files; manual desktop/input/audio observations and long-duration release gates
remain separate.
See the [current result](platform-results/2026-09-29-linux-interactions.md), the
[resume record](platform-results/2026-09-28-linux-resize.md) and
[interaction procedure](LINUX_INTERACTION_CHECKS.md). Earlier native passes below
qualify their recorded 0.0.17 binary, not this changed candidate.

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
The native Linux checkpoint is scoped to the receiving CachyOS desktop; see
[the Linux result](platform-results/2026-09-28-linux-x64.md) and [Linux setup](LINUX.md).
The 0.0.18 CachyOS/Hyprland short qualification is complete; native Xorg and
other Linux distributions are outside this first-release scope. Manual
input/appearance/listening, one-hour validation and final release sign-off remain
open. Mac regression remains open.
The resumed source snapshot is
`fmiramar-SCShader-0.0.18-source-linux-resume-2026-09-29-r8.zip`; verify its adjacent
`.sha256` file. The original 0.0.17 r5 Windows-to-Linux transfer remains preserved.
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
tests. The selected Linux short target is now qualified; macOS Apple Silicon
has a native build/runtime checkpoint with remaining gates above.
[START_HERE.md](../START_HERE.md)
explains the GitHub clone/pull workflow and the optional portable source ZIP. The
[Git audit](HANDOFF_GIT_AUDIT.md) is historical and describes the 2025 source-kit
state; it does not describe the current GitHub checkout.

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
- Keep notice/source delivery tied to the exact locked graph and binary, with
  the provenance limitations in [NOTICE_REVIEW.md](NOTICE_REVIEW.md). All current
  packaging targets pass the reviewed notice-assembly gate; graph/text changes
  require renewed review. Runtime provisioning/clean-machine qualification and
  signing/notarization are deferred to stable under the user's October 1 decision.
- Only after the acceptance gates: prepare the v0.1.0 tag/release with explicit
  publication authorization. No public release has been made in this session.

Scene restoration, independent multiple windows, general DAGs, richer texture
bindings, video/camera input, compute, and shared memory are later scope. They are
not silently treated as completed by the current narrow implementations.
