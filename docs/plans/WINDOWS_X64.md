# Windows x64 agentic plan

Status: the 0.0.18 desktop package passes RTX 3060/RX 580 short D3D12 modes and
three-display interactions, user window/input checks, isolated SC/help checks,
and RX 580 stress/recovery. The RTX 3060 passes one hour of traffic but fails an
eight-hour attempt on `E_QUEUE_FULL`; investigate and fix that queue-service issue
before repeating the long soak. Follow-up tracing passes 30-minute traffic on
both the RTX-driven and Radeon-driven displays, but a 45-minute diagnostic ends
at 830.779 seconds on the sender-lag guard without reproducing `E_QUEUE_FULL`.
A later WPR-only scheduler capture passes 45 minutes through the historic
queue-failure point with no sender pause or queue drop; this does not establish
a fix. The harness now records UTC lag checkpoints and sender PID for future
trace correlation. The root cause remains unknown. See the
[latest desktop soak result](../platform-results/2026-10-04-windows-rtx3060-soaks.md).
Listening, mixed DPI, sleep/wake and remaining release gates are tracked in the
[desktop result](../platform-results/2026-09-30-windows-dual-gpu.md).
The earlier 0.0.17 IDE fixes and NVIDIA/Intel laptop result are retained in the
[earlier Windows record](../platform-results/2026-09-28-windows-x64.md).
Follow [COMMON.md](COMMON.md) as the shared execution plan.

## W1. Establish a native MSVC environment

- Use an interactive Windows x64 desktop with a real GPU/driver supporting D3D12,
  native x64 SuperCollider, Python 3.11+, PowerShell, Git if desired, and rustup.
- Install Visual Studio C++ build tools and Windows SDK. Open an x64 developer
  PowerShell so the linker/SDK are discoverable. Do not substitute MinGW or a
  Windows arm64 emulation result for this target.
- Inspect `rustc -vV`, `cargo --version`, `python --version`, and
  `$env:PROCESSOR_ARCHITECTURE`. Required Rust host: `x86_64-pc-windows-msvc`.
  Toolchain comes from `rust-toolchain.toml`; do not upgrade the lockfile.
- Use a writable source path containing spaces for one run. Record Windows build,
  SC architecture/version, adapter, driver, and display/DPI configuration.

## W2. Native build, packaging, and installation

Run shared static checks first. Stop after any nonzero external exit code.

```powershell
$Target = 'x86_64-pc-windows-msvc'
$env:SCSHADER_ARTIFACT_PREFIX = 'fmiramar'
python tools/package_support.py --target $Target --check-host
if ($LASTEXITCODE -ne 0) { throw "Native host check failed" }
& ./tools/package_windows.ps1 -Architecture x64
if ($LASTEXITCODE -ne 0) { throw "Packaging failed" }
```

Review `dist/fmiramar-SCShader-0.0.18-windows-x64.zip` and its checksum (use the
current VERSION if implementation changes it). Verify PE32+ AMD64, `--version`,
`build-info.json`, and `dumpbin /dependents` from the developer environment. Record
actual DLL imports and whether a redistributable is required; do not assume a
development machine proves a clean runtime installation works.

Install the extracted inner extension using COMMON step 2. Set `$Extension` to
that installed folder, `$Renderer = Join-Path $Extension 'renderer/scshader-renderer.exe'`,
and `$Sclang` to the installed sclang executable. Test discovery without overriding
`ShaderServer.default` to a dummy path. Run all SC and help checks from COMMON.

## W3. Native backend and platform behavior

```powershell
python tools/run_acceptance.py --renderer "$Renderer" --backend dx12 --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/windows-dx12-01
if ($LASTEXITCODE -ne 0) { throw "D3D12 acceptance failed" }
```

Run COMMON scheduling/resource stress and the separate `gpu-test-hooks` recovery
build for `$Target`. Its executable ends in
`build/recovery-target/x86_64-pc-windows-msvc/debug/scshader-renderer.exe`.
Use a separate new output directory for an **optional** Vulkan acceptance run
with `--backend vulkan`; an unavailable Vulkan driver is not a D3D12 failure.

Critical Windows investigations:

1. SC process creation and `.exe` discovery with spaces/non-ASCII paths; quotes
   must not become part of the filename or a shell-injection opportunity.
2. Boot, quit, timeout, immediate reboot, SC recompilation, and renderer crash.
   Verify Job Object cleanup of owned test processes, including descendants after
   parent exit. The SC class also retains PID-scoped `taskkill /T` as a quit
   timeout fallback for its child. Never kill by image name.
3. D3D12 shader compilation/reflection, matrix layouts, buffer writes, graph order,
   texture formats/readback, overlay isolation, resize and logical-device recovery.
4. Mixed-DPI monitor moves, borderless fullscreen, focus/cursor/input coordinates,
   minimize/restore, sleep/wake, display-rate changes. Record missing hardware.
5. Missing runtime DLL, unsupported adapter/API, and OS download warning behavior.
   Do not disable security protections or claim the development ZIP is signed.

## W4. Exit gate

Native package + D3D12 critical tests + SC/help examples + process cleanup must
pass, with hashes and dependency findings in a Windows result record. Vulkan and
extra GPU vendors get separate coverage/limitations. Native Windows arm64 remains
outside this plan. Merge shared fixes and request Intel/other-target regression;
do not publish or start long-duration validation automatically.
