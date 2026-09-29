# Desktop implementation and native-test handoff

Active priority: [native platform compatibility first](PLATFORM_MILESTONE.md).
Use the [shared agentic plan](plans/COMMON.md) plus the specific
[Windows](plans/WINDOWS_X64.md), [Linux](plans/LINUX.md),
[Apple Silicon](plans/MACOS_ARM64.md), or [Intel regression](plans/MACOS_X64.md)
plan for ordered tasks, prerequisites, evidence, and exit gates. A portable
source handoff starts at [START_HERE.md](../START_HERE.md).

Version 0.0.15 prepares Windows and Linux implementation for testing on those
machines. Cross-target compilation is not a linked package, GPU/runtime pass,
driver-reset qualification, or final-release acceptance.

## Implemented selection

| Platform | `--backend auto` | Explicit alternatives | Display selection |
| --- | --- | --- | --- |
| macOS | Metal | `metal` | Native AppKit |
| Windows x64 | Direct3D 12 | `dx12`, `vulkan` | Native Windows |
| Linux | Vulkan | `vulkan`, `gl` | `auto`, `x11`, `wayland` |

## Recorded native hardware coverage

| Platform and tested OS/session | Graphics path | Adapter families exercised | Result scope |
| --- | --- | --- | --- |
| Windows x64 | Direct3D 12 | NVIDIA discrete and Intel integrated | Short acceptance and native interaction checks; see the [Windows result](platform-results/2026-09-28-windows-x64.md). |
| CachyOS Linux x86-64 under Hyprland | Vulkan on Wayland and Xwayland | NVIDIA discrete and Intel integrated | 0.0.18 native interaction, acceptance, and installed SuperCollider checks; see the [Linux result](platform-results/2026-09-29-linux-interactions.md). |
| macOS Apple Silicon and Intel | Metal | Not yet tested | Native build and hardware qualification remain open. |

The Windows and Linux results describe the recorded hybrid-graphics test system;
they do not establish compatibility with every GPU model, driver, Linux
distribution, or desktop session. Exact versions and per-run evidence are in the
linked platform records.

Automatic backend selection means the platform default above, not silent
fallback to a different graphics API. An unsupported or unavailable requested
backend fails clearly. Linux OpenGL is an explicit compatibility experiment;
its actual adapter limits and shader support still need native testing.

Hybrid graphics: `--adapter NAME` requires a unique case-insensitive GPU-name
substring among presentation-compatible adapters on the selected backend.
`--power-preference high-performance|low-power|none` is a hint when no adapter is
specified; the default remains high-performance. Both choices persist through
logical-device recovery. See [Windows setup](WINDOWS.md) for NVIDIA/Intel examples
and separate per-adapter acceptance commands.
`--window-system` forces an X11/Wayland connection only on Linux. An unavailable
forced display fails; automatic selection delegates to winit. The selected
backend and display connection are retained across logical-device recovery.

Wayland does not expose a primary monitor. Fullscreen without an explicit
monitor now leaves the choice to the compositor. Global window position requests
may be ignored on Wayland; do not require X11-style positioning there. Record
Xwayland separately from a native Xorg session when testing X11.

The implementation uses the locked [wgpu instance/display API](https://docs.rs/wgpu/30.0.0/wgpu/struct.InstanceDescriptor.html)
and winit's [Wayland](https://docs.rs/winit/0.30.13/winit/platform/wayland/index.html)
and [X11](https://docs.rs/winit/0.30.13/winit/platform/x11/index.html) event-loop
extensions. No additional runtime library was added to the locked Rust graph.

## Native builds later

Use an interactive desktop, Python 3.11+, SuperCollider, and the pinned Rust
toolchain. Windows uses the x64 MSVC toolchain, Visual Studio C++ build tools and
Windows SDK. Linux uses the native GNU toolchain and system linker. Install a
GPU driver providing the requested API and the desktop's X11/Wayland libraries.
Do not treat an SSH-only/headless session as a presentation test.

From the SCShader source directory:

```sh
# Linux x64 (Linux arm64 can use arm64 on a native arm64 GNU host).
tools/package_linux.sh x64
```

```powershell
# Windows x64, in PowerShell with the MSVC build environment available.
pwsh -File tools/package_windows.ps1 -Architecture x64
```

Packagers now check the operating system, native CPU and Rust host triple before
staging; build with an explicit target/output directory; and inspect ELF, PE32+,
or Mach-O headers before labeling the artifact. `build-info.json` records target,
version, renderer/lockfile hashes, and that packaging does not establish runtime
validation. This prevents a Mac executable from being labeled as a Linux x64
package. The macOS packager retains its additional `lipo` check.

Hosted CI is configured to run Rust/Python unit tests on Windows and Linux and
build native packages. A manual workflow dispatch can build inspection artifacts
without creating a release tag. It has not been run as part of this local implementation.
The Linux CI image is Ubuntu 24.04; older distribution compatibility is not yet
established. Native handoff must inspect ELF dependencies/required GLIBC versions
and Windows DLL imports before deciding on runtime redistribution requirements.

Extract the generated archive and install its `SCShader` folder in the normal
SuperCollider Extensions directory. Recompile the class library. Discovery checks
an explicit path, the folder containing the installed classes (including Quark
layouts), the conventional user extension, `SCSHADER_RENDERER`, then `PATH`.
Windows paths use `.exe`; command arguments remain separate, including spaces.

## Short native checks

Keep the source checkout for the test harnesses and installed package for SC
classes/help. Replace `--sclang` with the actual executable if it is not on PATH.
Every output directory must be new. These commands intentionally run only short
checks; they do not launch the one-hour or eight-hour gates.

```sh
# Linux: class/help/runtime checks, then separate display paths.
renderer="$PWD/renderer/target/x86_64-unknown-linux-gnu/release/scshader-renderer"
python3 tools/run_sc_checks.py --renderer "$renderer"
python3 tools/run_acceptance.py --renderer "$renderer" --backend vulkan \
  --window-system x11 --seconds 10 --modes traffic trivial reload feedback \
  --output-dir build/platform-tests/linux-x11
python3 tools/run_acceptance.py --renderer "$renderer" --backend vulkan \
  --window-system wayland --seconds 10 --modes traffic trivial reload feedback \
  --output-dir build/platform-tests/linux-wayland
```

```powershell
# Windows: the default backend is D3D12; Vulkan is a separate optional check.
$Renderer = (Resolve-Path 'renderer/target/x86_64-pc-windows-msvc/release/scshader-renderer.exe').Path
python tools/run_sc_checks.py --renderer "$Renderer"
python tools/run_acceptance.py --renderer "$Renderer" --backend dx12 --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/windows-dx12
python tools/run_acceptance.py --renderer "$Renderer" --backend vulkan --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/windows-vulkan
```

The acceptance runner preserves the requested backend/display in JSON and checks
the reported graphics API for explicit requests. Forced display selection is
implemented in the renderer; retain its console log and record the desktop
session/compositor yourself. The SC runner supports `--renderer` and `--sclang`
paths with spaces and uses platform-specific owned-process cleanup. Windows
validation children start suspended, join a kill-on-close Job Object, then
resume; descendants remain owned even after the parent exits. Native regression
tests cover both live and exited parents. See Microsoft's
[Job Objects documentation](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects).
Pass `--working-directory` to the SC/help runners to repeat launch checks from
the SuperCollider installation directory and exercise its DLL search environment.

Also check manually:

- Launch/quit/reboot from both SC IDE and command-line sclang; install paths with
  spaces and non-ASCII characters; missing executable and driver diagnostics.
- Visible shader output, controls/ramps, images, graph/feedback, FFT/waveform
  streams, watched reload, and overlay exclusion from feedback.
- Windowed/fullscreen, cursor/input/focus, resize/minimize/restore, mixed-DPI
  monitors, moving between displays, and sleep/wake.
- Logical-device recovery using the existing feature-gated recovery harness;
  separately record real hardware/driver resets, not as equivalent evidence.
- Run `tools/stress_schedule.py --renderer <native-executable>` and preserve its
  output. Check sustained load, memory and driver behavior on available GPUs.
- Run `python tools/stress_runtime.py --renderer <native-executable> --output-dir
  build/platform-tests/runtime-stress` (use `python3` on Linux). Version 0.0.16
  adds bounded resource churn and two-second incoming/error floods using the
  platform's default backend/display; see [STABILITY.md](STABILITY.md) for bounds
  and the distinction between offered traffic and observed queue saturation.

After the native short checks, schedule a measured one-hour run on the exact
candidate when appropriate. The user's eight-hour test remains the final-release
sign-off and is never started automatically by these instructions.
