# Milestone P1 — native desktop compatibility first

Decision recorded 2026-09-25 at the user's request. Baseline: development 0.0.16.
This supersedes the older ordering that placed additional Mac-only stress work
and notice cleanup before native platform qualification. It does not remove any
final-release gates or claim that another OS has been tested.

## Objective and priority

Build, install, exercise, and fix the existing SCShader implementation on Windows
x64, Linux x64, and macOS Apple Silicon, while retaining the Intel Mac regression
baseline. Find backend, lifecycle, path, packaging, display, and driver differences
before freezing a release candidate. A working Mac build cannot establish these
properties on other operating systems or architectures.

| Track | Starting evidence | Next required evidence | Priority |
| --- | --- | --- | --- |
| Windows x64 / D3D12 | Verified and installed 0.0.17; IDE/handshake fixes, SC/help, NVIDIA/Intel short modes, NVIDIA stress/recovery pass | Manual display/input coverage and other-platform regression; recorded limits remain | Automated short-test gate passed |
| Linux x64 / Vulkan | Cross-target compile only; native packager exists | GNU link/dependencies, installed SC examples, X11 and Wayland records | Critical for Linux use |
| macOS arm64 / Metal | Explicit native packager path exists | Native arm64 build outside Rosetta, SC integration, Apple GPU runtime | Critical for Apple Silicon support |
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
- [x] Linux x64 native linked package and critical short tests pass, with separate
      Xwayland/Wayland coverage recorded; native Xorg and compositor interaction
      remain outside this checkpoint.
- [ ] macOS arm64 native linked package and critical short tests pass.
- [ ] Cross-platform fixes retain macOS x64 critical regression passes.
- [ ] Each target has a completed result record, exact binary/source hashes,
      runtime dependency findings, and limitations; no unexplained critical failure.
- [ ] Notice/signing gaps and untested hardware are explicitly handed to the
      final-release milestone, without representing development packages as cleared.

Plans: [shared procedure](plans/COMMON.md), [Windows](plans/WINDOWS_X64.md),
[Linux](plans/LINUX.md), [Apple Silicon](plans/MACOS_ARM64.md),
[Intel regression](plans/MACOS_X64.md). Record results using
[the evidence template](plans/RESULT_TEMPLATE.md). A checklist tick needs an actual
result, not the existence of implementation code or a future test command.

## Work after P1

1. Finish remaining short interaction/stress coverage, dependency notice review,
   distribution/signing decisions, and authorized clean-checkout CI checks.
2. Freeze a candidate per supported platform. Schedule its exact-binary one-hour
   validation with the user; do not inherit the historical 0.0.14 result for 0.0.16.
3. The user runs the eight-hour test on the completed final candidate as the last
   release validation step. It is not queued or launched by this milestone.
4. Publish only with explicit authorization. This milestone authorizes no uploads,
   version tags, hosted workflow dispatches, or account authentication.

Scene restoration, multiple windows, unrestricted graph DAGs, richer textures,
video/camera, compute, and shared memory remain later scope, not prerequisites
for testing the implemented desktop feature set.

## Session log

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
