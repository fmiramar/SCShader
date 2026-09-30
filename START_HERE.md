# Continue SCShader on another computer

Updated 2026-09-30. Current development version: **0.0.18**. This is not a final
release. The canonical shared source is the public GitHub repository
[`fmiramar/SCShader`](https://github.com/fmiramar/SCShader), with development
work synced directly through `main`. Read-only cloning is public; authenticate
separately on each computer before pushing.

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

The Git repository contains the current source, tests, packaging scripts, plans,
and project history. Clone it for normal work across computers. A source archive
is an optional portable snapshot for cases where Git access is unavailable; it
has no `.git` history and must not replace a newer checkout. The archive layout is:

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
The archive manifest hashes transferred source files. It excludes `.git`, compiled
renderers, Rust dependency caches, old test logs, and installed extensions. It is
not an offline SDK: obtain Rust, the pinned dependencies, Python 3.11+,
SuperCollider, and the receiving platform's build tools and drivers. Do not place
the project checkout in SuperCollider's Extensions directory.

## First session

1. Clone the repository into a fresh writable project folder:

   ```sh
   git clone https://github.com/fmiramar/SCShader.git
   cd SCShader
   git switch main
   git rev-parse --short HEAD
   ```

   The public repository can be cloned without authentication. Confirm the
   reported commit is the current `origin/main` tip before testing.
2. Read [AGENTS.md](AGENTS.md), [STATUS.md](docs/STATUS.md), and the
   [platform milestone](docs/PLATFORM_MILESTONE.md).
3. Read the [shared execution plan](docs/plans/COMMON.md), then the matching plan:
   [Windows x64](docs/plans/WINDOWS_X64.md),
   [Linux x64 / optional arm64](docs/plans/LINUX.md),
   [macOS Apple Silicon](docs/plans/MACOS_ARM64.md), or
   [macOS Intel regression](docs/plans/MACOS_X64.md).
4. Follow the selected plan: native preflight, build, package, install, SC/help
   checks, short GPU tests, and hardware-specific checks. Fix demonstrated issues
   and record evidence. Do not start a long soak or create a release automatically.
5. If you received a source ZIP instead, verify its adjacent checksum and internal
   manifest (`python3 tools/package_source.py --verify <archive.zip>`; use `python`
   on Windows), then extract it to a fresh folder. ZIP extraction is a fallback
   source snapshot, not the normal multi-computer sync path.

All plan commands run from the inner `SCShader/` directory. On Windows, verify
each native command's exit code before moving on; PowerShell does not always stop
after a failing external executable. Shell scripts can be invoked with `bash`
even if the ZIP extractor did not restore executable bits.

## Version control across computers

Use this repository's `main` branch as the shared development line. Before each
session in an existing checkout, run `git pull --ff-only origin main`; do not work
from a stale copy or copy `.git` between machines. Before syncing, inspect
`git status`, review the complete staged diff, and add only project source, tests,
and documentation. Keep builds and raw test logs under ignored `build/`.

Each computer needs its own GitHub CLI authentication to push. Public clone and
fetch work without an account login. On a computer that will push, run:

```sh
gh auth login --hostname github.com --git-protocol https --web
git config user.name "Fellipe M. Martins"
git config user.email "54965070+fmiramar@users.noreply.github.com"
```

The Git identity commands are local to this checkout. Then commit and push
reviewed development changes to `main`. Resolve a non-fast-forward rejection by
fetching and reviewing the other machine's commit before integrating; do not force
push. Include a concise result record with exact source commit and renderer hash,
but do not commit private paths, credentials, or raw machine logs.

This standing workflow covers public development commits on `main`. Version tags,
binary release uploads, manual hosted workflow dispatches, and final release
publication remain separate actions requiring explicit authorization. A push to
`main` may run the configured branch CI automatically. The `.github` workflow can
publish when a version tag is pushed, so never create one as a sync shortcut.

## Prompt for the next agent

> Continue SCShader from the current `main` checkout (current version in VERSION). Read AGENTS.md, START_HERE.md,
> docs/PLATFORM_MILESTONE.md, docs/plans/COMMON.md, and this computer's platform
> plan. Native platform compatibility is the next priority. Inspect current Git
> state, identify OS/architecture/backend, implement only reproducible missing
> platform work, build/package/install locally, and run the documented short checks.
> Record exact evidence and remaining blockers. Preserve existing changes. Commit
> and push reviewed source, test, and documentation changes to public `main` for
> multi-computer sync. Do not create version tags, upload release assets, dispatch
> hosted workflows, upgrade dependencies, or start long tests without separate
> authorization/scheduling.

For architecture and the remaining release work, read
[IMPLEMENTATION_REPORT.md](docs/IMPLEMENTATION_REPORT.md). Historical log paths in
that report refer to the originating machine's ignored evidence, not files promised
by this source kit. Repeat the relevant checks on the receiving hardware.
