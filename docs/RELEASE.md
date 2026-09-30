# Release packaging

## Source development handoff (not an installable release)

`python3 tools/package_source.py --output dist/fmiramar-SCShader-0.0.16-source-handoff.zip`
creates a portable snapshot of the current source, including untracked files in
reviewed source roots. Use `python` on Windows. The adjacent original implementation
plan is required (or supply `--plan`); no Git history or native executable is copied.
The script refuses existing output/checksum names, excludes generated state,
normalizes shell-script permissions, and writes a per-file manifest and SHA-256.
Verify with `python3 tools/package_source.py --verify <source-archive.zip>`.
An optional `--base-commit <full-commit-id>` records the originating baseline,
not a claim that the working tree is committed. See [START_HERE.md](../START_HERE.md)
for platform plans and continuation. Runtime-release instructions follow below.

## Binary distribution

SCShader release archives are self-contained SuperCollider extensions. They intentionally do not require an end user to install Rust and they never download an executable at first boot.

## Archive layout

Each archive contains one outer directory followed by the installable extension:

```text
<owner>-SCShader-<version>-<platform>-<architecture>/
  SCShader/
    Classes/
    HelpSource/
    renderer/scshader-renderer[.exe]
    shaders/
    examples/
    docs/
    protocol/
    dependency-audit/
    build-info.json
    SCShader.quark
    LICENSE
```

Local assembly omits `<owner>-` unless `SCSHADER_ARTIFACT_PREFIX` is supplied; GitHub Actions sets it from `github.repository_owner`, never from a workstation name or local path. Every archive has a neighboring SHA-256 checksum file using a portable basename rather than an assembler's absolute path.

Install by extracting the archive and moving its `SCShader/` directory into the normal SuperCollider user Extensions directory, then recompile the class library. `ShaderServer` discovers the packaged executable in `SCShader/renderer/`. The Quark metadata describes the source package only; it does not run a network installer. Users install an architecture-matched official archive explicitly.

## Build targets

`tools/package_macos.sh x64|arm64`, `tools/package_linux.sh x64|arm64`, and `tools/package_windows.ps1 -Architecture x64` build a release executable, stage the extension directory, create a ZIP archive, and write its SHA-256 file. Each script checks the native OS/CPU/Rust host, builds an explicit target, and verifies the binary header before labeling it. See [PLATFORMS.md](PLATFORMS.md) for native build and test commands.

The checked-in GitHub Actions matrix uses explicit macOS Intel and Apple Silicon labels, Linux x64, and Windows x64. The workflow creates owner-prefixed assets and publishes them only for an explicit `v*` tag; ordinary pushes and pull requests only retain artifacts for inspection.

Packaging also requires Python 3.11+ and generates an offline, locked-target
dependency audit. Development packages label this evidence **NOT REVIEWED**.
Tagged CI builds set `SCSHADER_REQUIRE_NOTICE_TEXTS=1`, which currently rejects
macOS packages because ten full license texts remain unresolved. A text-presence
pass is not a completed distribution review; see [NOTICE_AUDIT.md](NOTICE_AUDIT.md).

## Maintainer checklist

1. Run Rust format, Clippy, and unit tests from a clean checkout.
2. Install the extension source locally, render all SCDoc pages, execute every help/README setup and cleanup with `tools/check_help_examples.py`, and run the relevant SuperCollider smoke tests. Rendering alone is not example-execution coverage.
3. Build an archive with the platform script and verify its checksum and contents.
4. Smoke-test the executable from the staged archive on its target architecture.
5. Record platform results in `docs/VERIFICATION.md`; do not describe a CI cross-build as a GPU/runtime verification.
6. For macOS distribution, assess codesigning and notarization before general distribution, including GitHub release assets.
7. After implementation, packaging, licenses, and other checks are complete, hand the final candidate to the user for the eight-hour sign-off as the last validation step. Do not launch it automatically during development.
8. Tag only after the user's final sign-off, the intended architecture artifacts/checksums, and explicit publication authorization.

Versions 0.0.13 and 0.0.14 passed isolated one-hour traffic gates with bounded RSS.
Version 0.0.14 also passed the short trivial/reload/feedback checks. Development
validation now targets sessions of at most one hour, with short auxiliary stress
checks; the eight-hour test is deferred to the user's final-release sign-off.
Consult [VERIFICATION.md](VERIFICATION.md) for completed checks. No development
result should be presented as a completed final-release sign-off.

Nothing has been published. Native short results now exist for Windows, the
selected Linux desktop, and macOS arm64; their exact scope is in the
[platform milestone](PLATFORM_MILESTONE.md). Intel regression and the gates below
remain open before a general release claim.

## Remaining distribution gates — 2026-09-30

- **Blocked notice clearance:** the current arm64 strict audit still finds ten
  missing full texts. See [NOTICE_AUDIT.md](NOTICE_AUDIT.md) for the exact versions
  and upstream review. Other targets' text-presence passes are not legal clearance.
- **Signing decision required:** the installed arm64 renderer has a linker-made
  ad-hoc signature, with no Developer ID authority or TeamIdentifier. Its
  `codesign --verify --strict` check passes; `spctl --assess --type execute`
  rejects it (exit 3). Evidence is in
  `build/platform-tests/macos-arm64-distribution-01/audit.json`.
- **Not run, transferred-package behavior:** the local installed executable has
  no `com.apple.quarantine` attribute. Local execution therefore does not qualify
  launch after a browser/download transfer. Assess that exact distribution
  package through the intended transfer and SuperCollider launch path; do not
  strip quarantine or disable Gatekeeper to obtain a pass. No notarization
  submission, signing-identity change or account authentication was performed.
- **Unavailable hardware:** native Intel Mac regression, external/mixed-DPI
  displays and other Apple GPU generations require separate hardware. Windows
  and Linux shared-class/harness changes also need their native regression.
- **Not run, final durations and CI:** schedule the exact candidate's one-hour
  validation with the user after the remaining fixes stabilize. The user owns
  the final eight-hour sign-off. Hosted CI and publication need explicit approval.

These are handoff items for the final-release milestone, not cleared release
requirements. Development packages remain labeled accordingly.
