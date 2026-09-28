# Local Git audit for the source handoff

> Historical note: this audit records the originating workspace as inspected on
> 2026-09-25. It does not describe the later Windows source-kit extraction. The
> current Windows project had no `.git` directory or parent Git repository when
> received; that archive intentionally excludes Git history.

Audited 2026-09-25. This is a read-only audit, not a commit or publication.

## Current Windows source-kit copy

On 2026-09-28, the user requested a local history for the Windows source-kit copy
and explicitly excluded web publication. That copy starts without the Mac
workspace history described below. A new standalone Git repository belongs inside
`SCShader/`; its initial commit can preserve the complete current source as a
baseline, after which subsequent changes on this shared folder can be tracked.
This does not reconstruct the order of prior edits. The portable source ZIP still
omits `.git`; use this shared checkout to retain the new history across the
Windows/Linux dual boot. Configure the user's chosen author identity before the
initial local commit. No remote, push, tag or release is needed.

## Finding

**The latest work is not fully committed.** SCShader lives inside the larger
workspace repository, not its own initialized repository. The workspace HEAD is
`4ecad92` (2026-08-20, `Update NovelOscUGens release documentation`); earlier commits
include `5745bfb` (`Add recent SuperCollider UGen port work`) and `89db243`
(`Initial workspace snapshot`). Nothing was staged at the start of this audit.

Before adding this handoff, Git tracked 21 files inside `SCShader/` and the adjacent
original implementation plan. Numerous tracked implementation files were modified;
most later classes, help, tests, platform tooling, and documentation were untracked.
Therefore `git archive HEAD`, a checkout of HEAD, or a patch containing only
tracked-file diffs would omit substantial current work.

The source kit is built from an explicit source-file inventory of the working
tree, not HEAD. It includes untracked source and new handoff plans, preserves the
original plan beside the project, excludes generated/private state, and provides
per-file hashes plus an archive checksum. It preserves current implementation but
does **not** provide historical commits, branches, installed binaries, or old logs.

## Other workspace findings (untouched)

- SCPhysicalModels has unrelated modified/untracked work.
- FMMatrixUGens and its plan are untracked in the workspace repository.
- The ambisonics and ChowDSP nested repositories report dirty state.
- An unrelated root-level test script is untracked.
- `git submodule status` fails because the existing GardenHose
  `VIRTUAL_TUBE_DELAY-EFFECT-` gitlink has no corresponding `.gitmodules` mapping.
  This is outside SCShader. It affects whole-workspace recursive Git operations,
  not the self-contained SCShader source archive; it was not repaired here.

No unrelated files were staged, committed, reverted, or packaged. No remotes were
contacted, and no authentication, workflow dispatch, tag, push, or release was made.

## Remaining Git work

If the user wants a local checkpoint commit, request/confirm that separately,
inspect repository-root `git status --short`, review the SCShader diff and all new
files, verify the approved author identity, and stage only `07-SCshaders/` source.
Do not use `git add .` at the workspace root or commit build outputs. An approved
local commit does not authorize pushing it. Other projects' missing commits and
the unrelated gitlink metadata need their own scoped decisions.

On a different computer, initialize a standalone repository only if requested,
inside the extracted `SCShader/` directory. Preserve the source ZIP/checksum as the
transfer baseline and keep the adjacent historical plan with the handoff. See
[START_HERE.md](../START_HERE.md) for returning changes safely.
