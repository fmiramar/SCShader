# Windows setup and hybrid graphics

SCShader uses a separate native x64 renderer on Windows. Its default graphics API
is Direct3D 12. Vulkan is an explicit alternative and requires its own driver and
test results. See the [Windows result record](platform-results/2026-09-28-windows-x64.md)
for the tested scope and remaining checks.

For desktop interaction coverage and the physical-input example, see
[Windows interaction checks](WINDOWS_INTERACTION_CHECKS.md).

Version 0.0.17 explicitly uses FXC for its D3D12 shader subset. Automatic DXC
discovery can load the old compiler bundled with SuperCollider/Qt when launched
from the IDE's working directory. The locked wgpu 30 requires DXC 1.8.2502 or
newer, while the tested SC 3.14 installation carries DXC 1.5. Selecting FXC avoids
that dependency and keeps IDE/terminal launches consistent. Shader model 6
features are outside the current D3D12 subset. See the
[wgpu compiler API](https://docs.rs/wgpu/30.0.0/wgpu/enum.Dx12Compiler.html).

## Build and install

Install x64 SuperCollider, Python 3.11+, rustup, and Visual Studio's Desktop
development with C++ workload including the Windows SDK. From this project's
`SCShader` directory in PowerShell:

```powershell
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
cargo fetch --manifest-path renderer/Cargo.toml --locked --target x86_64-pc-windows-msvc
& ./tools/package_windows.ps1 -Architecture x64
```

The package script accepts `-Python 'C:/path/to/python.exe'` when Python is not on
PATH. Use a normal Python distribution; an embedded interpreter with isolated
module search paths may not import the adjacent build tools. PowerShell 5.1 is
supported; PowerShell 7 is not required. Existing stage/archive output is retained:
move it to a backup location before packaging the same version again.

Extract the ZIP from `dist/`, then pass its inner `SCShader` folder to the installer:

```powershell
& ./tools/install_windows.ps1 -PackageDir './extracted/fmiramar-SCShader-0.0.17-windows-x64/SCShader'
```

Adjust that path to the extracted archive (CI and local handoffs may have an owner
prefix). The installer checks the target, version, and renderer checksum, installs
to the Windows SuperCollider user Extensions directory, and preserves any prior
SCShader installation in `SCShader-backups` beside `Extensions`. Stop an existing
SCShader renderer before replacing it. Other extensions are untouched. If your
SuperCollider uses a custom extension location, pass `-ExtensionsDir` with the
value of `Platform.userExtensionDir`.

Recompile the SuperCollider class library. Only install the packaged extension;
the source checkout contains test classes, build caches, and development files.

If PowerShell displays `>>`, it is waiting for an unfinished command. Press Ctrl+C
to return to a normal `PS` prompt, then paste the complete command on one line.
Relative `./tools/` paths require the project directory as the current location;
alternatively use absolute, quoted script and package paths.

SCShader is the project name, not a class. After Language > Recompile Class
Library, look up `ShaderServer`, `ShaderWindow`, or `Shader`; the project guide
is `Guides/SCShader`. Visual-only examples do not require an audio server.

## Choose NVIDIA or Intel

The default preference is `high-performance`, which usually prefers a discrete
GPU. A preference is a hint, not a vendor guarantee. `--adapter` selects a unique,
case-insensitive name substring among GPUs that can present to the renderer window
on the selected backend. Zero or multiple matches produce an error listing the
available names; there is no automatic substitution of another GPU.

Run this in SuperCollider on an NVIDIA/Intel laptop:

```supercollider
(
v = ShaderServer.new(\windowsShader);
v.rendererArgs_(["--backend", "dx12", "--adapter", "NVIDIA"]);
// To use integrated graphics, replace the preceding line with:
// v.rendererArgs_(["--backend", "dx12", "--adapter", "Intel"]);
v.waitForBoot { |server|
    [server.version, server.backend, server.device].postln;
    server.status { |receiver, status| status.postln };
};
)
```

Close this renderer before testing the other GPU:

```supercollider
(
v.free;
)
```

To request a power preference without fixing a GPU name, use
`["--power-preference", "low-power"]`, `high-performance`, or `none` in
`rendererArgs_` before boot. An explicit `--adapter` takes precedence over the
power hint. Adapter selection survives logical-device recovery. Startup logs
report the selected name, backend, device type, and available driver information;
the OSC handshake reports the name and backend as before.

This uses wgpu 30's [adapter enumeration](https://docs.rs/wgpu/30.0.0/wgpu/struct.Instance.html#method.enumerate_adapters),
[surface compatibility check](https://docs.rs/wgpu/30.0.0/wgpu/struct.Adapter.html#method.is_surface_supported),
and [power preference](https://docs.rs/wgpu/30.0.0/wgpu/enum.PowerPreference.html).
It does not change Windows-wide graphics preferences or reset the host GPU.

## Short checks on each GPU

Set `$Renderer` to the installed `renderer/scshader-renderer.exe`. Run these
serially with fresh output folders:

```powershell
python tools/run_sc_checks.py --renderer "$Renderer" --output-dir build/windows-sc-01
python tools/run_acceptance.py --renderer "$Renderer" --backend dx12 --adapter NVIDIA --seconds 10 --modes traffic trivial reload feedback --output-dir build/windows-nvidia-01
python tools/run_acceptance.py --renderer "$Renderer" --backend dx12 --adapter Intel --seconds 10 --modes traffic trivial reload feedback --output-dir build/windows-intel-01
```

The acceptance records retain the requested GPU and verify the reported name.
One adapter's pass does not qualify the other. These short checks do not qualify
sleep/wake, mixed-DPI displays, driver resets, or long sessions. Use the existing
[Windows platform plan](plans/WINDOWS_X64.md) for remaining verification.

The SC/help runners accept `--audio-output-device` and `--audio-input-channels 0`
if the default audio-device pair cannot open. These override only private test
servers, not Windows or IDE settings. Select a device reported by SuperCollider;
an audio boot failure is not a successful analysis/FFT test.

Use `--working-directory` with the SuperCollider installation folder when running
SC/help checks to cover the same compiler-DLL discovery environment as the IDE.
If a diagnostic says a message was sent to `nil`, recreate its variable before
using it; class-library recompilation clears interpreter variables such as `v`.
