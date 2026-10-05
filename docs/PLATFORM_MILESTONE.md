# Milestone P1 — native desktop compatibility first

Decision recorded 2026-09-25 at the user's request. Baseline: development 0.0.16.
This supersedes the older ordering that placed additional Mac-only stress work
and notice cleanup before native platform qualification. It does not remove any
final-release gates or claim that another OS has been tested. The first Linux
release scope is intentionally narrow: CachyOS x86-64 GNU with Hyprland and
Vulkan. Other distributions, compositors, native Xorg, Linux arm64 and Linux
GL remain outside that release. Windows x64 and macOS Apple Silicon/Intel remain
release targets; this restriction applies only to Linux.

## Objective and priority

2026-10-01 distribution decision: implement notice/source delivery now; defer
runtime provisioning/clean-machine qualification and signing/notarization/download
trust to a later stable release. Current development users install the documented
prerequisites themselves. Notice assembly/source packaging is implemented; see
[NOTICE_REVIEW.md](NOTICE_REVIEW.md) for the explicit provenance limitations and
[RELEASE.md](RELEASE.md) for the requirements. These deferred tracks are not passes
and do not change native regression or final-duration gates.

Build, install, exercise, and fix the existing SCShader implementation on Windows
x64, Linux x64, and macOS Apple Silicon, while retaining the Intel Mac regression
baseline. Find backend, lifecycle, path, packaging, display, and driver differences
before freezing a release candidate. A working Mac build cannot establish these
properties on other operating systems or architectures.

| Track | Starting evidence | Next required evidence | Priority |
| --- | --- | --- | --- |
| Windows x64 / D3D12 | Installed 0.0.18 desktop package and interaction checks pass; RTX 3060 one-hour traffic passes but its eight-hour attempt fails on `E_QUEUE_FULL`; RX 580 has a separate eight-hour pass. A traced RTX run hit the RSS guard at 15 minutes under Nsight; a matched WPR-only 15-minute control passed | Resolve/retest the RTX queue-service failure and account for Nsight's apparent RSS overhead, plus listening, mixed DPI/sleep-wake and other-platform regression | Desktop long-run gate incomplete |
| CachyOS x86-64 / Hyprland / Vulkan | 0.0.18 package, Wayland/Xwayland interactions, acceptance, SC/help, stress and recovery pass | Manual input/appearance/listening and long release gates | Short Linux gate passed |
| macOS arm64 / Metal | Native 0.0.18 Apple M5 short suites plus 19 native interaction stages and user input confirmation pass; SC normal-close correction passes 26 installed checks | Listening/sleep-wake observations and Intel/shared-platform regression | Short and interaction gates passed |
| macOS x64 / Metal | Local 0.0.16 short suites and installed docs passed | Rerun affected checks after shared changes; retain memory workaround | Critical regression protection |
| Linux arm64 / Vulkan | Packaging target exists, no native claim | Optional native arm64 follow-up using the Linux plan | Not a first x64 desktop gate |
| Windows Vulkan / Linux GL | Explicit alternate selection implemented | Separate native results, limits, or clear experimental label | Optional compatibility paths |

Within the three new primary tracks, use whichever native machine is available
first. Do not claim Windows arm64, universal macOS binaries, or Linux musl support.
Keep Xwayland distinct from native Xorg and native Wayland.

## Deliverables and exit gates

- [x] Save the priority decision and portable handoff instructions.
- [x] Prepare shared and per-platform agentic implementation/test plans.
- [x] Windows x64 native linked package and critical automated short tests pass;
      manual display/input and untested hardware remain explicitly limited.
- [x] CachyOS x86-64 / Hyprland native linked package and critical short tests pass,
      with separate Wayland/Xwayland coverage recorded for 0.0.18.
- [x] Linux 0.0.18 resize correction passes native interaction and critical-suite
      regression in the selected CachyOS/Hyprland/Vulkan scope; manual and long
      release gates remain separate.
- [x] macOS arm64 native linked package and critical short tests pass;
      additional display/input/lifecycle and untested hardware remain limited.
- [ ] Cross-platform fixes retain macOS x64 critical regression passes.
- [ ] Each target has a completed result record, exact binary/source hashes,
      runtime dependency findings, and limitations; no unexplained critical failure.
- [x] Notice/signing gaps and untested hardware are explicitly handed to the
      final-release milestone, without representing development packages as cleared.
      See the [distribution handoff](RELEASE.md#remaining-distribution-gates--2026-09-30);
      notice assembly/source delivery is implemented with disclosed provenance
      caveats; runtime provisioning and signing qualification are deferred to stable.

Plans: [shared procedure](plans/COMMON.md), [Windows](plans/WINDOWS_X64.md),
[Linux](plans/LINUX.md), [Apple Silicon](plans/MACOS_ARM64.md),
[Intel regression](plans/MACOS_X64.md). Record results using
[the evidence template](plans/RESULT_TEMPLATE.md). A checklist tick needs an actual
result, not the existence of implementation code or a future test command.

## Work after P1

1. Finish remaining short interaction/stress coverage and authorized clean-checkout
   CI checks. Preserve the October 1 notice/source policy and explicit stable-release
   deferral of runtime provisioning and signing qualification.
2. Freeze a candidate per supported platform. Schedule its exact-binary one-hour
   validation with the user; do not inherit the historical 0.0.14 result for 0.0.16.
3. The user runs the eight-hour test on the completed final candidate as the last
   release validation step. It is not queued or launched by this milestone.
4. Sync reviewed development source, tests, and documentation to the public GitHub
   `main` branch so the same candidate can be tested on other computers. This
   standing authorization does not cover release tags, binary release uploads,
   hosted workflow dispatches, or final release publication; those need explicit
   authorization.

Scene restoration, multiple windows, unrestricted graph DAGs, richer textures,
video/camera, compute, and shared memory remain later scope, not prerequisites
for testing the implemented desktop feature set.

## Session log

- 2026-10-05 desktop RTX capture diagnostic: a custom WPR DxgKrnl GPU
  present/scheduling profile and Nsight D3D12 smoke produced usable traces with
  zero ETW loss. The requested 45-minute RTX-left run stopped after 900.082
  seconds when Nsight-instrumented renderer RSS exceeded the 128 MiB growth
  guard by 0.9 MiB; sender lag stayed below 27 ms and no `E_QUEUE_FULL` occurred.
  A matched WPR-only 900-second control passed with zero skipped updates, 2.508
  ms maximum sender lag and stable RSS. This points to profiler overhead but
  does not establish a renderer leak or resolve the earlier queue failure. The
  eight-hour gate remains unqualified. See the
  [capture diagnostic](platform-results/2026-10-04-windows-rtx3060-soaks.md).

- 2026-10-05 desktop RTX 3060 diagnostics: traced 60-second adapter/output
  controls pass, and traced 30-minute traffic passes on both the RTX-driven left
  display and Radeon-driven primary. The next 45-minute run stops after
  830.779 seconds because sender lag reaches 704.582 ms, above the 250 ms test
  guard. Renderer logs confirm NVIDIA RTX 3060 and no `E_QUEUE_FULL`; this is a
  separate sender stall and does not reproduce or resolve the recurring queue
  failure. The eight-hour gate remains unqualified. See the
  [traced results](platform-results/2026-10-04-windows-rtx3060-soaks.md).

- 2026-10-04 desktop RTX 3060 soaks: the exact 0.0.18/D3D12 binary passes one
  hour of traffic (3,599,919 updates), then fails its eight-hour attempt after
  2,482.785 seconds on `E_QUEUE_FULL` with one continuous update dropped. Four
  short modes pass on RTX 3060 and RX 580; static tests, scheduling/runtime
  stress and logical recovery also pass. The recurring RTX queue-drop symptom
  remains unexplained, so another long soak should wait for investigation and a
  fix. See the [result](platform-results/2026-10-04-windows-rtx3060-soaks.md).

- 2026-10-04 laptop eight-hour checks: Intel UHD 630 and GTX 1050 Ti Max-Q both
  pass 28,800 seconds of D3D12 traffic on the unchanged renderer and corrected
  sender. Independent evaluation confirms the individual passes, bounded memory
  and zero sampled queue drops. The local supervisor has a stale overall flag;
  raw evidence is preserved and the discrepancy explained. Desktop RTX, other
  modes/platforms and final release gates remain open. See the
  [eight-hour results](platform-results/2026-10-04-windows-laptop-eight-hour.md).

- 2026-10-03 laptop one-hour checks: both Intel UHD 630 and GTX 1050 Ti Max-Q
  pass 3,600 seconds of D3D12 traffic with the corrected sender. Automatic serial
  execution finished at 23:15:50 local time. No repeated assistant polling; zero
  sampled renderer queue drops. Eight-hour and desktop investigation gates remain
  open. See the [one-hour results](platform-results/2026-10-03-windows-laptop-one-hour.md).

- 2026-10-03 unattended laptop follow-up: Intel and NVIDIA each pass 600 seconds
  of traced D3D12 traffic with zero skipped updates or sampled queue drops. The
  scheduler-only control completes. System profiling was unavailable, the old
  >250 ms stall did not reproduce, and long-duration/desktop gates remain open.
  Details are in the [comparison](platform-results/2026-10-03-windows-soak-comparison.md).

- 2026-10-03 desktop/laptop comparison: confirmed a Python 3.12 Windows sender
  pacing defect and switched the soak clock to `perf_counter()`. Four one-minute
  diagnostics and eight corrected ten-second mode checks pass on Intel UHD 630
  and GTX 1050 Ti Max-Q. Rare sender stalls and desktop RTX queue-service gaps
  remain open; display ownership must be controlled in the next desktop matrix.
  See the [comparison](platform-results/2026-10-03-windows-soak-comparison.md).

- 2026-10-03 Windows: investigate the three failed RTX / D3D12 eight-hour attempts
  (incoming OSC overflow after 15–19 minutes). Optional timing and Windows power
  tracing reproduced an event-loop gap in a short run; four repeated short modes
  pass. The user paused the longer diagnostic after about four minutes for a model
  switch and display-power adjustment. Short passes do not clear this long-run
  failure; see [the investigation](platform-results/2026-10-03-windows-soak.md).
- 2026-10-03 Windows follow-up: user requested sequential eight-hour RTX 3060 and
  RX 580 traffic tests after setting AC display-off and sleep to Never; physical
  monitors were to be switched off. On the same packaged renderer, RTX 3060/DX12
  failed after 107 seconds on `E_QUEUE_FULL`, while RX 580/DX12 passed 28,800 seconds
  with 28.8 million updates and no drops. The RTX long-run failure remains open;
  see the [Windows record](platform-results/2026-10-03-windows-soak.md).
- 2026-10-03 Windows hybrid-GPU follow-up: the user requested eight-hour D3D12
  traffic runs on Intel UHD 630 and NVIDIA GTX 1050 Ti Max-Q with development
  renderer `0.0.18` (source `a15b6be`, SHA-256
  `df4d0eff7dee8490482902800ee60df6329539aba2892fde3c05015ad11dacea`). Both
  four-mode short preflights pass. The Intel run fails after 20,199.89 seconds
  on 311.996 ms sender lag; the NVIDIA run fails after 15.781 seconds on
  436.999 ms sender lag. Neither qualifies for eight hours. The sampled incoming
  queue-drop count stays zero and renderer frames progress until each harness
  abort; the sender scheduling cause remains unresolved. Raw JSON, CSV and logs
  remain under ignored local `build/eight-hour-sequence-20261003-135026/` and are
  not part of this commit. See the
  [laptop soak record](platform-results/2026-10-03-windows-uhd630-gtx1050ti-soaks.md).

- 2026-09-30 Windows desktop: native 0.0.18 short checks pass on RTX 3060 and
  RX 580. The extended Windows harness passes 21 stages per GPU across three
  displays, with independent native geometry/DPI and GPU allocation evidence.
  All 59 Rust tests and 116 Python tests pass; 26 isolated installed SC checks,
  18 help demos, RX 580 stress/recovery and package path checks pass. Default
  standalone SC configuration has an unrelated Quark `Document` error; normal
  IDE compile/lifecycle checks pass. The user confirms window/input checks across
  all three displays; listening, sleep/wake and release gates remain open. See
  [the result](platform-results/2026-09-30-windows-dual-gpu.md).

- 2026-09-30: the user authorized recurring development sync to public GitHub
  `main`. Commit `3e8f022` was pushed with the macOS follow-up source, tests, tools,
  and documentation. This is a development checkpoint, not a tagged release or
  release-asset upload. Handoff rules now describe clone/pull/push on each machine.

- 2026-09-30 interaction follow-up: 19 native macOS window stages pass on Apple
  M5/Metal, including native state, Retina/GPU allocation changes, synthetic keys,
  resize/focus events and native close. The user confirms physical input and
  Escape from fullscreen. Normal native close exposed a misleading SC `E_BOOT`
  warning; the focused correction and restart/early-exit regressions pass.
  Python: 104 pass, four Windows skips. The strict notice recheck still finds ten
  missing texts, and Gatekeeper assessment rejects the ad-hoc binary. Explicit
  release handoff is recorded; no signing, long run or publication is claimed.

- 2026-09-30: native Apple Silicon 0.0.18 build/package and recovery-hook build
  pass with pinned Rust 1.97.1. Rust formatting, strict Clippy, 59 Rust tests,
  and 91 Python tests (four Windows skips) pass. All 82 installed files match
  the package. User-run evidence confirms four ten-second Apple M5/Metal modes,
  18 help demos and all 25 installed SC checks. The SC suite was rerun with
  complete console evidence after the session sandbox restriction was lifted.
  Found and fixed a test gap: ten-second feedback runs previously performed no
  resizes. The harness now requires confirmed size changes and valid framebuffer
  scaling; the native ten-second rerun confirms three resizes at 2x Retina scale.
  Scheduling, 80 resource cycles, short floods and exact-pixel logical recovery
  pass with complete evidence. Continuous traffic did not fill the incoming
  queue. Local UDP and SC launch now work directly in the agent environment.
  See the [macOS result](platform-results/2026-09-30-macos-arm64.md). Additional
  display/input/lifecycle checks, Intel regression and long release gates remain.
- 2026-09-29: full 0.0.18 Wayland compositor and Xwayland OSC interactions pass
  on NVIDIA and Intel after correcting checker coordinate/option parsing.
  Python: 84 pass, four Windows skips. After the display-environment correction,
  all 16 ten-second acceptance modes and both 25-check SC suites pass. The
  explicit 150 ms Pattern lead time makes all 18 help demos pass on both displays.
  The follow-up passes all help/SCDoc and scheduling checks. Stress exposed a
  startup memory-baseline mismatch; the checker now waits for a stable window
  and records geometry, with exact leak checks intact.
  Python: 84 pass, four Windows skips. The stabilized 80-cycle resource/flood
  stress, logical Vulkan recovery and path-with-spaces/non-ASCII checks then pass
  on Intel; the candidate is installed with 82 verified files. Manual and long
  release gates remain. See
  the [current result](platform-results/2026-09-29-linux-interactions.md).
- 2026-09-28 resume: fixed ignored synchronous winit resize results and unchanged-size
  target recreation; added a Hyprland runner with GPU allocation regression checks
  and failure-stage evidence. First native NVIDIA/Wayland run reached the first
  resize but timed out. The runner now settles the Wayland floating configure
  before OSC resize checks. See the [resume record](platform-results/2026-09-28-linux-resize.md).
- 2026-09-28: user confirmed the Windows IDE example works and selected Linux
  next after Windows interaction checks. Both GPUs pass native resize/reload,
  fullscreen/borders and OS minimize/restore at 125% scale. Physical interaction
  observations remain pending; the source kit includes the
  [Linux handoff](LINUX_HANDOFF.md). Mac regression remains open for later.
- 2026-09-28: native CachyOS x64 Linux package, installation, Vulkan adapter
  selection, Wayland/Xwayland SC/help checks, scheduling/resource/flood checks,
  and logical-device recovery passed on Intel and NVIDIA. The Linux packager now
  uses Python's standard ZIP writer because this host has no external `zip` tool.
  A Hyprland-owned-window helper remains incomplete; see the Linux result record.
- 2026-09-28: reproduced the IDE's incompatible-DXC launch failure and fixed it
  with explicit FXC. Duplicate ready replies no longer reconfigure the window.
  Candidate SC/help checks and Windows Job Object cleanup pass. The verified ZIP
  is installed, with all file hashes matching and installed SC/SCDoc rechecks
  passing. NVIDIA/Intel short modes and NVIDIA stress/recovery pass; see the
  [0.0.17 Windows record](platform-results/2026-09-28-windows-x64.md).
- 2026-09-27: native Windows package and separate NVIDIA/Intel D3D12 short modes
  pass. Integration exposed startup bus lateness, a user IDE boot failure and test
  cleanup gaps. P1 remains open; see the
  [Windows record](platform-results/2026-09-27-windows-x64.md).
- 2026-09-25: chose platform implementation/compatibility before further long
  Mac-only tests; prepared native handoff plans and a portable source kit.
- 2026-09-25: audited local Git. Current implementation is not fully committed;
  the handoff therefore captures working-tree files, including untracked source.
  No staging, commit, authentication, or publication was requested/performed.
  See [the audit](HANDOFF_GIT_AUDIT.md). Native target results remain pending.
- 2026-09-25: source-kit inventory includes 136 project files plus the original
  plan. Fresh extraction passed all 49 Python tests, Rust formatting, and offline
  locked manifest inspection. ZIP contents/checksums are verified; no new native
  GPU or duration qualification is claimed. See [VERIFICATION.md](VERIFICATION.md).
