# Linux handoff after Windows 0.0.17

The user selected native Linux testing as the next platform after Windows
interaction checks. Windows `ShaderFFTTexture` was also confirmed working in the
IDE. macOS regression and native Apple Silicon qualification
remain open for later; no Linux runtime result is implied by this source kit.

## Receiving the source

Transfer `fmiramar-SCShader-0.0.17-source-linux-handoff-2026-09-28-r5.zip` from
`dist/` and verify it with its adjacent `.sha256` file. On Linux, verify the checksum and extract into a fresh
writable directory. Open the inner `SCShader-handoff/SCShader/` as the project.
Do not copy the whole handoff into SuperCollider Extensions. The adjacent original
implementation plan and internal per-file hash manifest travel with the source.

From that inner source directory, verify the archive with:

```sh
python3 tools/package_source.py --verify /path/to/the-transferred-source.zip
```

The kit includes current source, tests, docs, platform plans and build scripts.
It excludes Windows binaries, installed extensions, caches, raw logs, `.git`,
personal machine settings and generated build outputs. It is not an offline SDK.
The Windows working folder has no Git metadata; do not treat the historical Mac
workspace audit as this folder's current Git state.

## Changes carried from Windows

- Explicit FXC selection for D3D12 avoids loading the incompatible DXC bundled
  with the tested SuperCollider installation. Linux continues to default to Vulkan.
- Duplicate OSC ready replies no longer replay SC window state. This shared
  lifecycle change needs native Linux and later Mac regression coverage.
- Explicit adapter selection and corrected platform/executable detection retain
  their strict matching behavior; no silent graphics-backend fallback.
- Verification runners support explicit sclang configuration, writable isolated
  caches, chosen audio devices and launch directories. Native Windows Job Objects
  are conditional; Linux retains owned process-group cleanup.
- Source packaging excludes AppleDouble metadata and verifies normalized archive
  paths. Test fixtures no longer depend on POSIX `touch` for reload.
- Short flood probes precede catch-up bursts; load/health assertions are unchanged.
- `check_windows_interactions.py` is Windows-only. The manual interaction example
  uses the ordinary cross-platform SC API and can be exercised on Linux.

No dependency versions were upgraded. Keep Rust 1.97.1 and `renderer/Cargo.lock`.
The [Windows record](platform-results/2026-09-28-windows-x64.md) preserves passed
checks, earlier failures, fixes, executable hashes and remaining limits. The
Metal per-pass submission workaround remains in place.

## First Linux session

Read [AGENTS.md](../AGENTS.md), [STATUS.md](STATUS.md),
[COMMON.md](plans/COMMON.md), and the [Linux plan](plans/LINUX.md).
Identify the actual distribution, architecture, Rust host, SuperCollider, audio
device/backend, GPU/driver and desktop session before choosing package commands.
Use a real desktop; WSL/software rendering does not establish native Linux GPU
compatibility. Install distribution-specific prerequisites from observed needs.

Follow the existing Linux plan: static checks, native linked package, ELF/runtime
dependency inspection, clean backed-up extension installation, SC/help checks,
then explicit ten-second Vulkan modes. Record native Xorg, Xwayland and Wayland
separately. Run only sessions available on that machine and mark missing paths.
Add short scheduling/resource stress and a separate hook-enabled recovery build;
ship only the ordinary production build. Record a fresh dated Linux result.

Do not rerun Windows tests under emulation as Linux qualification. Windows binary
hashes are provenance for those Windows results, not expected Linux hashes.
One-hour validation requires scheduling; the user's eight-hour sign-off remains
the last final-release step. No hosted CI, publication or GPU driver reset is
authorized by this handoff.

## Resume prompt

> Continue SCShader 0.0.17 on this native Linux desktop. Windows implementation
> and automated short checks passed; read docs/LINUX_HANDOFF.md and the recorded
> Windows manual-check status before proceeding. Follow AGENTS.md,
> docs/plans/COMMON.md and docs/plans/LINUX.md. Preserve source and prior installs,
> use the pinned toolchain/lockfile, build and install locally, and run the short
> native SC/help/Vulkan checks. Diagnose failures without weakening assertions.
> Keep Xorg, Xwayland and Wayland results distinct. Save exact hashes and evidence.
> No long soak, dependency upgrade or publication without the required authorization.
