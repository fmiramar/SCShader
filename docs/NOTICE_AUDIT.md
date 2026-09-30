# Dependency-notice audit

`tools/license_inventory.py` produces a deterministic, target-specific inventory
from `cargo metadata --locked --offline`. It preserves notice text and its hash,
records declared licenses/repositories and packaged VCS revisions, and identifies
missing text without substituting a generic SPDX template. Python 3.11 or newer
is required. Build or fetch dependencies for the target before auditing; the tool
itself does not fetch from the network or modify the lockfile.

```sh
python3 tools/license_inventory.py --target x86_64-apple-darwin \
  --output-dir build/license-audit/macos-x64-review --require-texts
```

Use a fresh output directory for each audit. Outputs are `inventory.json` and
`notices-audit.txt`; neither contains local manifest/cache paths. The graph
conservatively includes build-time and procedural-macro dependencies as well as
runtime dependencies. Optional notice-file paths cannot escape their crate root.

`licenses/upstream-notices.json` supplements omissions in crate archives with
14 exact upstream files for 23 pinned package versions. Text is stored as JSON
strings to preserve the original bytes, including absent final newlines. Each
source carries an immutable commit URL and SHA-256; package version, packaged VCS
commit, and SPDX declaration must match. Hash changes, mutable URLs, duplicate
entries, and revision/declaration mismatches fail the audit. No network request
is made when generating or packaging the inventory.

Upstream files that only discuss licensing or link to external license terms
are retained as **context only**; they do not fill a missing-license-text entry.

`--require-texts` exits nonzero when declarations/text are missing **after writing
the audit**. Even a zero exit is only a file-presence check, not an SPDX-expression
evaluation or proof of complete obligations. Both outputs remain marked
**unreviewed**. Do not install them as final third-party notices without reviewing
license alternatives/conjunctions, embedded third-party material, copyright
statements, and the distribution's actual target/features.

## Current result — 0.0.14

The locked target graphs were audited offline on 2026-09-24; these are metadata
checks, not native builds or runtime verification of the other platforms.

| Target | Packages | Missing before supplements | Still missing full text |
| --- | ---: | ---: | ---: |
| macOS x64 | 132 | 22 | 10 |
| macOS arm64 | 132 | 22 | 10 |
| Linux x64 | 152 | 4 | 0 |
| Windows x64 | 123 | 6 | 0 |

The ten macOS gaps are `dispatch 0.2.0` and nine newer objc2-family packages.
The [dispatch issue](https://github.com/SSheldon/rust-dispatch/issues/18) reports
the missing MIT text; neither the pinned revision nor the upstream root provided
a license file when inspected. The [pinned objc2 licensing discussion](https://github.com/madsmtm/objc2/blob/8852b424193ca41602281b3d7540d7c8ed51e49a/LICENSE.md)
links to license terms and describes Apple SDK considerations, but does not
contain the complete terms. Do not borrow older attribution notices for newer
versions without establishing that they apply. Missing text is not itself a
finding that a project is unlicensed.

All package scripts now include `dependency-audit/inventory.json` and
`dependency-audit/notices-audit.txt`, explicitly marked **NOT REVIEWED**. These
are development evidence, not final third-party notices or distribution
clearance. Set `SCSHADER_REQUIRE_NOTICE_TEXTS=1` to fail packaging on unresolved
text gaps; tagged CI builds set it automatically. The macOS gate currently
fails as intended. Even a Linux/Windows text-presence pass still needs review of
license choices, source headers, embedded material (including registry data),
attribution, and any notice obligations before final release.

Ten unit tests cover text preservation, missing/context-only reporting, package
filtering, out-of-root rejection, pin/hash/declaration validation, and the checked-in
collection. Those audit tests need no GPU or credentials.

## Apple Silicon recheck — 2026-09-30, development 0.0.18

The pinned-toolchain audit of the current locked arm64 graph still reports 132
packages and the same ten missing texts. `--require-texts` exits 1 as intended.
Evidence is under `build/license-audit/macos-arm64-review-20260930/`, including
`inventory.json`, `notices-audit.txt` and an anonymous upstream-tree review.

The gaps are `block2 0.6.2`, `dispatch 0.2.0`, `dispatch2 0.3.1`, `objc2 0.6.4`,
`objc2-encode 4.1.0`, and `objc2-core-foundation`, `objc2-core-graphics`,
`objc2-io-surface`, `objc2-metal`, `objc2-quartz-core` at `0.3.2`.
The four pinned objc2 repository trees contain the existing licensing discussion
but no additional full license files. The
[pinned dispatch tree](https://github.com/SSheldon/rust-dispatch/tree/82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748)
contains no notice file; the upstream issue has no comment supplying one.
The [objc2 licensing discussion](https://github.com/madsmtm/objc2/blob/8852b424193ca41602281b3d7540d7c8ed51e49a/LICENSE.md)
still links to terms rather than providing the complete texts and attributions.

No substitute notices or dependency upgrades were introduced. Closing this gate
requires establishing the applicable complete texts and attribution for these
exact versions, then reviewing the license choices and Apple SDK-derived material.
Contacting maintainers would need separate authorization to send messages.
This review remains blocked for distribution clearance, not for local development.
