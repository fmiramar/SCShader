# Continue SCShader on another computer

Original handoff date: 2026-09-25. Current development version: **0.0.18**,
including the Linux resize correction of 2026-09-28. Not a final release.
The next milestone is native platform compatibility, before further extended
stress tests and final-release validation.

Active platform: **macOS Apple Silicon**, selected by the user on 2026-09-30.
The native 0.0.18 package, four ten-second Metal modes, confirmed Retina resizing,
scheduling/resource stress, logical recovery and installed SC/help checks pass
on Apple M5. Evidence is recorded in the
[Apple Silicon result](docs/platform-results/2026-09-30-macos-arm64.md).
The later follow-up passes 19 native window/input stages and user interaction
checks. A corrected normal-close warning in `ShaderServer` passes all 26 installed
SC checks; the renderer is unchanged. Resume the
[Apple Silicon plan](docs/plans/MACOS_ARM64.md) and
[interaction checklist](docs/MACOS_INTERACTION_CHECKS.md): analysis/FFT listening,
sleep/wake and unavailable hardware remain separate. The earlier agent sandbox
blocker is resolved; local UDP and SuperCollider work directly in this session.

Previous active platform: **Linux**. The 0.0.17 native checkpoint passed its
short suites. The
[0.0.18 resize follow-up](docs/platform-results/2026-09-28-linux-resize.md) and
[current Linux checks](docs/platform-results/2026-09-29-linux-interactions.md):
0.0.18 GPU interactions, short acceptance, SC/help, scheduling, stress, recovery
and path suites pass for the selected CachyOS/Hyprland scope. Manual desktop
observations and long release gates remain. The
[Linux handoff](docs/LINUX_HANDOFF.md) preserves the
earlier Windows change summary. Intel Mac regression remains a separate open track.

## What this transfer contains

The source archive includes the current working files, including uncommitted and
untracked implementation, classes, shader assets, help, tests, packaging scripts,
CI configuration, the pinned Rust toolchain/lockfile, and platform plans. Its layout:

```text
SCShader-handoff/
  HANDOFF_MANIFEST.json
  SC_Shader_Interface_Agentic_Implementation_Plan.md
  SCShader/
    START_HERE.md
    AGENTS.md
    renderer/  Classes/  HelpSource/  tests/  tools/  docs/  ...
```

The original plan stays beside `SCShader/` so its relative links still work.
The manifest hashes every transferred source file. No `.git` history, compiled
renderer, Rust dependency cache, old test logs, or installed extension is included.
This is not an offline SDK: initially obtain Rust, its pinned toolchain/dependencies,
Python 3.11+, SuperCollider, and the receiving platform's build tools and drivers.
Do not place this entire source tree in SuperCollider's Extensions directory.

## First session

1. Verify the ZIP against its adjacent `.sha256` file (`shasum -a 256` on macOS,
   `sha256sum` on Linux, or `Get-FileHash -Algorithm SHA256` on Windows). Extract to
   a fresh writable folder, not over an existing project or installed extension.
2. Open the inner `SCShader/` directory as the agent/editor working directory.
   Read [AGENTS.md](AGENTS.md), [STATUS.md](docs/STATUS.md), and the
   [platform milestone](docs/PLATFORM_MILESTONE.md).
3. Read the [shared execution plan](docs/plans/COMMON.md), then the matching plan:
   [Windows x64](docs/plans/WINDOWS_X64.md),
   [Linux x64 / optional arm64](docs/plans/LINUX.md),
   [macOS Apple Silicon](docs/plans/MACOS_ARM64.md), or
   [macOS Intel regression](docs/plans/MACOS_X64.md).
4. Verify the archive's internal manifest from this directory (use `python` on
   Windows): `python3 tools/package_source.py --verify /path/to/the-source.zip`.
   This checks archive content; make changes only after preserving the original ZIP.
5. Follow the selected plan: native preflight, build, package, install, SC/help
   checks, short GPU tests, and hardware-specific checks. Fix demonstrated issues
   and record evidence. Do not start a long soak or publish anything automatically.

All plan commands run from the inner `SCShader/` directory. On Windows, verify
each native command's exit code before moving on; PowerShell does not always stop
after a failing external executable. Shell scripts can be invoked with `bash`
even if the ZIP extractor did not restore executable bits.

## Version control and returning changes

See the [Git audit](docs/HANDOFF_GIT_AUDIT.md) for history provenance. The
originating Mac workspace had changes that were uncommitted/untracked at the time
of its 2026-09-25 audit. This Windows source-kit extraction arrived without its
`.git` history; its new local repository begins with the current source-kit state
and cannot recreate the earlier edit sequence.

`SCShader/` is the standalone repository root for the shared Windows/Linux folder.
The source ZIP remains a portable snapshot and intentionally excludes `.git`;
using the same dual-boot folder retains the local history. Set the user's approved
author identity. Do not add a remote, copy another workspace's `.git`, push or tag.
The `.github` workflow assumes this project root.

Return a scoped patch/commits against that baseline plus the platform result record,
or another source handoff ZIP if no local repository was initialized. Include new
files: plain `git diff` omits untracked files. Review/merge on return rather than
overwriting newer work on another machine. Keep raw hardware logs in a separate
reviewed evidence archive if needed; never include credentials or private paths
in the publishable summary.

## Prompt for the next agent

> Continue SCShader from this source handoff (current version in VERSION). Read AGENTS.md, START_HERE.md,
> docs/PLATFORM_MILESTONE.md, docs/plans/COMMON.md, and this computer's platform
> plan. Native platform compatibility is the next priority. Inspect current Git
> state, identify OS/architecture/backend, implement only reproducible missing
> platform work, build/package/install locally, and run the documented short checks.
> Record exact evidence and remaining blockers. Preserve existing changes; no
> publication, remote writes, dependency upgrades, or long tests without approval.

For architecture and the remaining release work, read
[IMPLEMENTATION_REPORT.md](docs/IMPLEMENTATION_REPORT.md). Historical log paths in
that report refer to the originating machine's ignored evidence, not files promised
by this source kit. Repeat the relevant checks on the receiving hardware.
