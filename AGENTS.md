# SCShader agent handoff rules

These instructions apply to this standalone project and travel with source
handoffs. Read `START_HERE.md`, `docs/STATUS.md`, and the selected platform plan
before implementing changes. The active milestone is
`docs/PLATFORM_MILESTONE.md`; historical checklists are not completion evidence.

## Scope and safety

- Preserve existing changes. Work only on SCShader and its adjacent original
  implementation plan; do not modify other projects in a larger workspace.
- The user authorizes ongoing development sync to the public
  `https://github.com/fmiramar/SCShader` repository by committing and pushing
  reviewed source, tests, and documentation directly to `main`. This makes those
  commits public. Do not ask again before routine pushes within this scope.
  Pushes to `main` may run the repository's configured branch CI automatically.
  Manual hosted workflow dispatches, release tags, binary release uploads, and
  changing release/version metadata still need separate authorization. The
  existing version-tag workflow can publish a release.
- A Git audit is not permission to stage or commit. Before each sync commit,
  inspect repository-root status and the staged diff; stage only intended project
  source, tests, and documentation. Exclude binaries, logs, build/stage/dist trees,
  caches, and raw test evidence. Use the user's approved author identity, never a
  computer name. Use `fmiramar` for local artifact prefixes.
- Each computer uses its own GitHub CLI login; never copy credentials or keychain
  data between machines. Public clone/fetch needs no login. Authenticate locally
  with `gh auth login --hostname github.com --git-protocol https --web` before
  pushing. Pull `main` before starting work on an existing checkout and push
  development updates to `origin main` after reviewing the exact commit.
- Keep binaries, logs, build/stage/dist trees, caches, and raw test evidence out
  of source commits. Never copy credentials, `.git`, or machine-specific settings
  to another computer. The shared Git repository carries source history; use the
  source ZIP only as an optional portable snapshot. Include reviewed source files
  in commits, including newly created files.
- Back up an existing installed SCShader extension before replacing it. Install
  verification builds in the receiving user's normal SuperCollider Extensions
  directory. Do not delete other extensions or leave duplicate SCShader classes.
- Test processes must be PID-scoped and owned by the test. Never kill unrelated
  SuperCollider/renderer processes by executable name. Run live suites serially.

## Architecture and implementation

- Keep the Rust renderer out of scsynth's audio thread. Preserve the versioned
  OSC contract, bounded queues/diagnostics, and explicit resource ownership.
- Use the pinned Rust toolchain and `Cargo.lock`; do not upgrade dependencies
  merely to get a platform build through. Investigate the smallest failing case.
- Never label a cross-compile, software adapter, Rosetta run, or short smoke as
  native hardware qualification. Record exact target, backend, and executable hash.
- Do not remove assertions or add silent backend fallback to make a test pass.
  State unsupported behavior, fix a reproducible issue, and add regression coverage.
- Do not ship `gpu-test-hooks`. Logical-device recovery does not qualify a real
  hardware reset. Never deliberately reset the host GPU without separate approval.
- Keep private paths, hostnames, tokens, and personal environment details out of
  publishable docs. Cite techniques and upstream sources when adding them.

## Documentation and checks

- Runnable SC help examples belong in parenthesized `code::` blocks and must
  work from an unsaved IDE document using installed shader assets, not placeholders.
- For audio input examples, default to looping `PlayBuf` / `ExampleFiles.child`,
  with a commented `SoundIn` alternative. Synchronize buffers/SynthDefs in a
  Routine before starting consumers; use short audible settings and a fade-in.
- Keep one project guide in `HelpSource/Guides/` and class pages in
  `HelpSource/Classes/`, with exact class names, valid lower-case SCDoc tags,
  constructor arguments documented once, and simple `See also` links.
- After help changes, install cleanly, run `SCDoc.indexAllDocuments(true)`, render
  changed pages with zero warnings/errors, and execute the affected examples.
  Existing `verify_scdoc` and help-example harnesses cover these requirements.
- Use short, explicit-duration checks first. Do not start one-hour tests without
  scheduling them with the user. The user performs the eight-hour test as the
  last final-release step. Save logs and avoid frequent polling to conserve usage.
- Update the milestone and verification record with passed, failed, blocked, and
  not-run results. Keep negative evidence. Do not claim the project is final.
