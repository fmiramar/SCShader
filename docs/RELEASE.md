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

SCShader binary archives contain the extension and renderer, but rely on the
user-installed prerequisites below. They do not require end users to install
Rust and never download an executable at first boot.

## Current development policy — 2026-10-01

At the user's request, automatic runtime installation/bundling and clean-machine
qualification (item 2), and signing/notarization/download-trust qualification
(item 3), are deferred to a later stable release. Users must install the documented
requirements themselves. This is not a claim of universal OS/driver compatibility
or of a signed/trusted download. Do not disable OS security protections.

- Use native, architecture-matched **SuperCollider 3.14.1** and the platform/API
  described in [PLATFORMS.md](PLATFORMS.md). The current Windows baseline is
  Windows 11 x64 build 26200 with D3D12; the Apple Silicon baseline is macOS
  26.6.2 with Metal. Linux scope remains CachyOS x86-64, Hyprland and Vulkan,
  using the exact environment in the linked native result records.
- Windows needs the **Microsoft Visual C++ v14 x64 runtime**, at least as new as
  the MSVC 14.44 builder. This desktop's `VCRUNTIME140.dll` is **14.51.36231.0**;
  that is the observed test baseline, not a newly qualified minimum. Install the
  supported [Microsoft x64 redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)
  yourself; do not copy individual DLLs from an unrelated application. Windows
  supplies the Universal CRT. Runtime installers are not included in SCShader.
- Install the appropriate GPU driver yourself. The current desktop used RTX 3060
  driver **32.0.15.9636** and RX 580 driver **30.0.13023.4001**; these are exact
  tested versions, not minimum versions or recommendations to downgrade. Other
  hardware/driver versions need their own tests. D3D12 uses FXC; a separate DXC
  installation is not required for this subset.
- Source builders additionally need **Rust/Cargo 1.97.1**, the unchanged lockfile,
  Python **3.11+** (Windows test interpreter **3.14.0**), and native C/C++ tooling.
  Windows was built with VS 2022 **MSVC 14.44** and SDK **10.0.26100.0**. Python
  and Rust are build/test tools, not binary-package runtime requirements.

Notice assembly and matching-source packaging (item 1) are implemented as
described in [NOTICE_REVIEW.md](NOTICE_REVIEW.md), including the explicit upstream
provenance caveats. This does not approve a final release or replace the user's
last eight-hour sign-off.

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
    corresponding-source.json
    SCShader.quark
    LICENSE
    COPYING
```

Local assembly omits `<owner>-` unless `SCSHADER_ARTIFACT_PREFIX` is supplied; GitHub Actions sets it from `github.repository_owner`, never from a workstation name or local path. Every archive has a neighboring SHA-256 checksum file using a portable basename rather than an assembler's absolute path.

Each binary ZIP also has an adjacent `*-corresponding-source.zip` and checksum.
Publish all four files together. `corresponding-source.json` inside the binary
package identifies the exact source archive and renderer hashes. The source ZIP
contains the full locked Cargo vendor tree, portable `.cargo/config.toml`, build
instructions and `SOURCE_MANIFEST.json`. Verify it with
`python tools/package_corresponding_source.py --verify <source.zip>`.
For an offline rebuild, extract it and run the command in `SOURCE_README.txt`
from the extracted root, after installing the pinned compiler and native SDK.

Install by extracting the archive and moving its `SCShader/` directory into the normal SuperCollider user Extensions directory, then recompile the class library. `ShaderServer` discovers the packaged executable in `SCShader/renderer/`. The Quark metadata describes the source package only; it does not run a network installer. Users install an architecture-matched official archive explicitly.

## Build targets

`tools/package_macos.sh x64|arm64`, `tools/package_linux.sh x64|arm64`, and `tools/package_windows.ps1 -Architecture x64` build a release executable, stage the extension directory, create a ZIP archive, and write its SHA-256 file. Each script checks the native OS/CPU/Rust host, builds an explicit target, and verifies the binary header before labeling it. See [PLATFORMS.md](PLATFORMS.md) for native build and test commands.

The checked-in GitHub Actions matrix uses explicit macOS Intel and Apple Silicon labels, Linux x64, and Windows x64. The workflow creates owner-prefixed assets and publishes them only for an explicit `v*` tag; ordinary pushes and pull requests only retain artifacts for inspection.

Packaging requires Python 3.11+ and generates an offline locked-target notice
inventory. Every package now requires the checked-in notice-assembly review;
changed graphs/notices fail, including ordinary branch CI. The old
`SCSHADER_REQUIRE_NOTICE_TEXTS` switch is retained but cannot bypass this review.
The standalone audit still labels unchecked output **NOT REVIEWED**. Cargo fetches
all locked dependencies before the offline vendoring step; building a package can
therefore use the network. See [NOTICE_AUDIT.md](NOTICE_AUDIT.md).
All packagers refuse existing stage/archive/checksum paths instead of replacing
them. Use a new safe `SCSHADER_ARTIFACT_PREFIX` for a separate inspection build.

## Maintainer checklist

1. Run Rust format, Clippy, and unit tests from a clean checkout.
2. Install the extension source locally, render all SCDoc pages, execute every help/README setup and cleanup with `tools/check_help_examples.py`, and run the relevant SuperCollider smoke tests. Rendering alone is not example-execution coverage.
3. Build an archive with the platform script and verify its checksum and contents.
4. Smoke-test the executable from the staged archive on its target architecture.
5. Record platform results in `docs/VERIFICATION.md`; do not describe a CI cross-build as a GPU/runtime verification.
6. For a later stable release, complete runtime provisioning/clean-machine and signing/notarization/download-trust qualification. These are explicitly deferred for current development packages, not marked passed.
7. After implementation, packaging, licenses, and other checks are complete, hand the final candidate to the user for the eight-hour sign-off as the last validation step. Do not launch it automatically during development.
8. Tag only after the user's final sign-off, the intended architecture artifacts/checksums, and explicit publication authorization.

Versions 0.0.13 and 0.0.14 passed isolated one-hour traffic gates with bounded RSS.
Version 0.0.14 also passed the short trivial/reload/feedback checks. Development
validation now targets sessions of at most one hour, with short auxiliary stress
checks; the eight-hour test is deferred to the user's final-release sign-off.
Consult [VERIFICATION.md](VERIFICATION.md) for completed checks. No development
result should be presented as a completed final-release sign-off.

Development source checkpoints are synced to the public `main` branch so they can
be tested across computers. No versioned release or release assets have been
published. Native short results exist for Windows, the selected Linux desktop,
and macOS arm64; their exact scope is in the
[platform milestone](PLATFORM_MILESTONE.md). Intel regression and the gates below
remain open before a general release claim.

<a id="remaining-distribution-gates--2026-09-30"></a>

## Remaining distribution work

- **Notice provenance limitations:** assembly/source delivery are implemented;
  the declaration-backed dispatch recovery and upstream Apple SDK discussion are
  disclosed in [NOTICE_REVIEW.md](NOTICE_REVIEW.md). This is not legal certification.
- **Signing deferred to stable:** the installed arm64 renderer has a linker-made
  ad-hoc signature, with no Developer ID authority or TeamIdentifier. Its
  `codesign --verify --strict` check passes; `spctl --assess --type execute`
  rejects it (exit 3). Evidence is in
  `build/platform-tests/macos-arm64-distribution-01/audit.json`.
- **Deferred, transferred-package behavior:** the local installed executable has
  no `com.apple.quarantine` attribute. Local execution therefore does not qualify
  launch after a browser/download transfer. Assess that exact distribution
  package through the intended transfer and SuperCollider launch path; do not
  strip quarantine or disable Gatekeeper to obtain a pass. No notarization
  submission, signing-identity change or account authentication was performed.
- **Unavailable hardware:** native Intel Mac regression, external/mixed-DPI
  displays and other Apple GPU generations require separate hardware. Windows
  and Linux shared-class/harness changes also need their native regression.
- **Not run, final durations:** schedule the exact candidate's one-hour
  validation with the user after the remaining fixes stabilize. The user owns
  the final eight-hour sign-off. Routine authorized pushes run branch CI;
  manual workflow dispatch and release publication still need explicit approval.

These are handoff items for the final-release milestone, not cleared release
requirements. Development packages remain labeled accordingly.
