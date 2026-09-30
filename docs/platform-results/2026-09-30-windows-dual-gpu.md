# Windows x64: RTX 3060, RX 580 and three displays — 2026-09-30

Development **0.0.18**. The automated short checks pass on this desktop.
The user also confirms the installed IDE window/input checks pass. Listening,
sleep/wake and final release gates remain open. This result does not inherit the
earlier Windows laptop's user observations.

## Identity

- Starting source: clean `main` at `c90c9e9f0c54217d9ac42a7c77b870d4c4969a86`.
  This follow-up adds monitor verification and its tests/documentation. Renderer,
  SC classes, dependency pins and version metadata are unchanged.
- Native Windows 11 Pro x64, OS build `10.0.26200`; Intel Core i5-10600K.
- Rust/Cargo 1.97.1, host/target `x86_64-pc-windows-msvc`, pinned Clippy/rustfmt;
  Python 3.14.0; Visual Studio 2022 MSVC 14.44 and Windows SDK 10.0.26100.0.
- SuperCollider 3.14.1; both `sclang` and `scsynth` verified as x64 PE binaries.
  Owned audio tests use WASAPI / Realtek HD Audio 2nd output, 48 kHz, 64-frame
  driver blocks and zero hardware input channels.
- NVIDIA GeForce RTX 3060, Windows driver `32.0.15.9636`; Radeon RX 580 Series,
  driver `30.0.13023.4001`. Both explicitly selected and reported **Dx12** hardware
  adapters during their acceptance and monitor runs.
- Three active Dell displays: SE2419HR, S2421HN and U2419H. All measured windows
  report 96 DPI / 100% scale. Native display modes are 1920×1080, with integer
  nominal refresh reported as 59 Hz by `EnumDisplaySettingsW`.

| Desktop position | Windows display | Output adapter |
| --- | --- | --- |
| Left, primary: `(0, 0)` | `DISPLAY5` | Radeon RX 580 |
| Middle: `(1920, 0)` | `DISPLAY1` | RTX 3060 |
| Right: `(3840, 0)` | `DISPLAY7` | Radeon RX 580 |

Both selected rendering adapters were exercised on all three outputs, including
outputs driven by the other GPU. This verifies continued frames, native geometry
and OSC/GPU allocation state. The separate user check below confirms visible
output and input, without claiming a manually selected adapter on each run.

Package: `fmiramar-SCShader-0.0.18-windows-x64.zip`.

| Artifact | SHA-256 |
| --- | --- |
| ZIP | `7916a5fcc428024d5419f07fd8a2e4735cfed1b9b580a4c9dc972cbe9dd8fd67` |
| Installed production renderer | `b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff` |
| Separate debug recovery renderer | `29e6767b1ca55a8261ca7705190409f4b40240b4d8db3981067a763d7701889c` |
| Cargo.lock | `99477baa12a417c9c2aee715b1bc27cb59bb40035f7b4855480ef07e5fe762f7` |

The production build uses ordinary release features. The recovery executable
uses `gpu-test-hooks` and is kept outside the installed package. All 83 installed
file hashes match the freshly extracted ZIP. No previous SCShader installation
was present on this desktop. DLL imports include Windows system APIs,
`VCRUNTIME140.dll` and the Universal CRT; a machine without developer tools has
not been used to qualify redistribution requirements.

## Checks and evidence

Raw evidence is local and ignored under
`build/platform-tests/windows-dual-gpu-20260930/`. Paths below are relative to it.

| Gate | Result | Evidence and scope |
| --- | --- | --- |
| Formatting / strict Clippy | pass | `fmt.log`, `clippy.log`; locked all-target/all-feature checks |
| Rust unit tests | pass | `rust-tests.log`: 57 main tests plus two example tests, 59 total |
| Python tests | pass | Initial 108 pass; `python-tests-monitors.log`: 116 pass, including eight new monitor/ownership regressions and native Windows Job Object tests |
| Native package / install | pass | `package-process-policy.log`, `install.log`, `dll-imports.log`; x64 PE, 83 matching installed files |
| Normal IDE class context | pass | `normal-ide-compile_check-console.log`, `normal-ide-lifecycle_smoke-console.log`; normal user class configuration with `sclang -i scqt`, installed discovery and live lifecycle |
| Installed SC suite | pass | `sc-isolated-01/`, console log and `sc-summary.json`; all 26 checks in an isolated class configuration, launched from the SC installation directory |
| SCDoc / runnable help | pass | 13 pages rendered with zero warnings/errors; `help-isolated-01/`: 18 demos and their cleanups, 36 blocks |
| RTX 3060 D3D12 | pass | `dx12-nvidia-01/`: traffic, trivial, reload and feedback, each explicitly 10 seconds |
| RX 580 D3D12 | pass | `dx12-radeon-01/`: same four explicitly 10-second modes |
| Both GPUs across all monitors | pass | `monitors-nvidia-01/`, `monitors-radeon-01/`: 21 native stages per GPU, including windowed/fullscreen/restored states on each display |
| Future scheduler bounds | pass | `schedule-default-01-console.log`: command/payload limits, clearing, rejection and nested timing; default selection, actual adapter not retained by this harness |
| Resource/flood stress | pass | `runtime-default-01/`: RX 580/D3D12, 80 cycles and explicit two-second floods |
| Logical-device recovery | pass | `recovery-default-01-console.log` and renderer log: RX 580/D3D12, exact GPU pixels, graph/cache/control/ramp/clock/schedule restoration and second-loss exit 70 |
| Spaces/non-ASCII package path | pass | `sc-space-path-01/`: compile, lifecycle, bus mapping, packaged examples and SCDoc from an extracted path containing spaces and `ç`, launched from the SC directory |
| Physical window/input checks | pass, user-reported | Installed `09_window_interaction.scd`: visible animation, mouse movement/clicks, F/Escape, R/V, moves across all three displays, resize, minimize/restore, reloads > 0, error = nil and cleanup; adapter not separately captured for this manual session |
| Audio listening / visible analysis and FFT | not run | Automated audio/help checks pass; this desktop's listening observations remain unreported |
| Mixed DPI / different refresh rates | not run | All three connected displays currently share 100% scale and the same reported nominal refresh |
| Sleep/wake / physical driver reset | not run | No sleep or host GPU reset requested |
| Vulkan / other native platforms | not run | This checkpoint is Windows D3D12; shared Python regressions pass but no new Linux/macOS native result is claimed |
| Notice texts | pass | Strict Windows audit: 123 packages, zero missing texts; license review and signing remain open |

Each traffic mode offered 10,000 updates and received ten fresh pong and status
samples over its measured ten seconds. Feedback confirms three real resizes per
GPU. The short acceptance duration does not reach the 30-second RSS warmup and
does not establish a post-warmup memory-growth result.

On RX 580, resource counts and owned texture bytes returned to baseline after
80 cycles; post-warmup process RSS grew by 1,336 KiB. Each malformed/invalid-command
flood produced 59 reported diagnostics and 3,937 counted suppressions while frames
and health replies remained live. Continuous traffic offered 511,744 commands
with no observed queue drops; it did **not** saturate the incoming queue.

## Preserved failures and changes

1. `package.log` records the initial PowerShell `AllSigned` refusal. The reviewed
   local build/install scripts subsequently ran with process-scoped `RemoteSigned`;
   the machine/user execution policy was not changed. Native build tools and the
   pinned Rust toolchain were used without changing dependencies.
2. `sc-compile-01/compile_check.log` records an existing Quark configuration error:
   `adclib` extends the IDE-only `Document` class, which standalone `sclang` excludes.
   SCShader's compile marker was present, but the strict runner correctly failed
   on the error. The normal IDE class context passes compile/lifecycle checks.
   It still emits unrelated path-not-found lines during Quark initialization.
3. Full SC/help checks use a local `sclang-isolated.yaml` that explicitly includes
   the SC core library and the installed SCShader extension, with default include
   paths excluded. The normal user class configuration and other extensions were
   left intact. This scope is part of the result, not a normal-CLI pass claim.
4. The Windows interaction runner previously never moved between displays.
   `--all-monitors` now enumerates displays and checks actual monitor ownership,
   native client size, window DPI, logical position, full monitor coverage,
   frame progress and corresponding GPU texture allocation. Its observer uses
   per-monitor DPI awareness and preserves the failing stage/native sample.
   Physical input and mixed-DPI coverage are reported independently.
5. The existing allocation regression checker moved unchanged to
   `tools/window_checks.py` for Windows/Linux reuse. The new unit tests reject
   stale framebuffer/DPI replies, wrong or partially covered monitors, incorrect
   scaled positions and dead/reused owned windows. No product defect was reproduced
   by this desktop's automated checks.

## Remaining work

- Record this desktop's audio-example listening and visible analysis/FFT
  observations. Follow [the interaction procedure](../WINDOWS_INTERACTION_CHECKS.md).
- Sleep/wake, mixed DPI/refresh settings, optional Vulkan and a machine without
  developer runtimes remain separate coverage. NVIDIA-specific stress/recovery
  is not implied by the RX 580/default-selection stress result.
- The one-hour exact-candidate run is **not run** and needs separate scheduling.
  The final eight-hour sign-off remains reserved for the user.
- Final license review, signing/distribution decisions and shared-platform
  regression remain release gates. This is a development checkpoint.
- At the user's request, GitHub sign-in/sync follows completion of this short-test
  batch. Raw evidence stays local and ignored; development source sync does not
  publish release assets or close the remaining release gates.
