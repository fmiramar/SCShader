# Notice assembly and matching source — 2026-10-01

Scope: SCShader 0.0.18, Rust 1.97.1 and Cargo.lock SHA-256
`99477baa12a417c9c2aee715b1bc27cb59bb40035f7b4855480ef07e5fe762f7`.
This is a technical notice-assembly review, not legal advice or a guarantee
that every upstream licensing question has been resolved.

## Distribution contents

SCShader remains **GPL-3.0-or-later**. `LICENSE` is the project declaration;
`COPYING` contains the complete GPLv3 text, retrieved verbatim from the
[pinned SPDX license list](https://github.com/spdx/license-list-data/blob/31ba1a50e5397e00a304dbadc76531740e89ee48/text/GPL-3.0-or-later.txt).
Each binary package now carries:

- `dependency-audit/THIRD_PARTY_NOTICES.txt`: full dependency notices, attribution,
  selected license alternatives, embedded registry headers and recovery provenance.
- `dependency-audit/inventory.json`: exact target graph and notice hashes. Build
  and procedural-macro dependencies are conservatively retained, not represented
  as necessarily linked into the renderer.
- `dependency-audit/rust/`: the pinned toolchain's standard-library copyright
  inventory and its linked license texts. Cargo metadata alone misses this code.
- `corresponding-source.json`: matching source ZIP name/hash, binary hash,
  lockfile hash and Rust version.

Publish the adjacent `*-corresponding-source.zip` and both ZIP checksums **with
every binary archive**, at the same download location with equivalent access.
The source ZIP includes all SCShader sources/build scripts and the complete
locked Cargo vendor tree, including dependency files outside the active target.
Original headers/notices are preserved in that tree. It includes offline build
instructions and a per-file hash manifest; it does not require the external
agentic implementation plan. A moving `main` URL or GitHub's automatic source
ZIP is not a replacement for this matching source archive. See GPLv3 section 6(d)
in `COPYING` and the [GNU source-distribution FAQ](https://www.gnu.org/licenses/gpl-faq.html#DistributeWithSourceOnInternet).

Rust/compiler/SDK, operating-system libraries and graphics drivers are external
prerequisites, not bundled proprietary SDKs. Rebuilding from source does not
promise a byte-identical executable; the manifest identifies the packaged binary
and records the actual source snapshot. Packaging must run against an unchanged
working tree, with ordinary release features (never `gpu-test-hooks`).

## License choices

`licenses/review.json` explicitly records the reviewed declaration-to-choice
mapping. MIT is selected for MIT/Apache alternatives; Zlib is selected for
MIT/Apache/Zlib alternatives, including the newer objc2 framework/dispatch2
and zune declarations. Single-license Apache-2.0,
BSD-2-Clause, BSD-3-Clause, ISC and Zlib dependencies retain their terms. Legacy
`MIT/Apache-2.0` declarations mean alternatives. `Apache-2.0 AND MIT` (dpi) and
`MIT AND Unicode-3.0` (unicode-ident) retain **both** obligations. Extra alternative
license texts remain in the notice bundle; this does not make every alternative
an additional condition. Apache-2.0 code can be combined in a GPLv3 work; see
the [Apache compatibility explanation](https://www.apache.org/licenses/GPL-compatibility.html).
Dependency ownership and original licenses are not replaced by SCShader's GPL.

The review is pinned to the lockfile and the exact per-target inventory, including
all collected notice text/context, hashes and provenance. `--distribution`
fails on a changed graph, changed notice, unknown target or missing text. Updating
the digest is a review action, not an automatic way to silence a packaging error.
The ordinary audit remains explicitly unreviewed until this check is requested.

## Omitted upstream texts and embedded material

- Older objc2 packages use the license from their exact packaged VCS revision.
  `objc2-foundation` 0.2.2 was previously a false pass: `copying.rs` is source code,
  not a license. The scanner now rejects source-code extensions while recognizing
  actual files such as `LICENSE.MIT` and `LICENSE.APACHE`.
- For newer objc2 packages, the pinned root declared MIT or the three alternatives
  but linked externally. Upstream subsequently supplied full texts in
  [commit ee9a7ada](https://github.com/madsmtm/objc2/commit/ee9a7ada2131f5944b8750428e265c15632f2a19),
  following [issue 826](https://github.com/madsmtm/objc2/issues/826).
  Those exact texts are preserved, including the upstream 2026 attribution.
  Their later revision is explicitly disclosed; no claim is made that these files
  existed in the older crate archive. The pinned licensing discussion about
  Apple SDK-derived bindings remains present as context, not silently resolved.
- `dispatch` 0.2.0 has no upstream license file. Its
  [exact manifest](https://github.com/SSheldon/rust-dispatch/blob/82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748/Cargo.toml)
  declares MIT and credits **Steven Sheldon**. The package includes that declaration
  plus the [canonical MIT terms](https://github.com/spdx/license-list-data/blob/31ba1a50e5397e00a304dbadc76531740e89ee48/text/MIT.txt).
  This is explicitly declaration-backed recovery, **not author confirmation**.
  The canonical copyright placeholders remain a template; no copyright year or
  owner notice was invented. The [upstream omission](https://github.com/SSheldon/rust-dispatch/issues/18)
  itself is not fixed by SCShader. Seek upstream clarification or legal review
  if author-confirmed notice provenance is required for a distribution decision.
- `khronos_api` 3.1.0 also contains OpenGL/EGL/WebGL registry material. Thirteen
  exact source-header excerpts are validated against file and excerpt hashes.
  The ANGLE BSD notice comes from its exact submodule revision
  [7403dd2c](https://github.com/google/angle/blob/7403dd2cd3764fe96660fe09892e764e9ae1dbca/LICENSE),
  not merely from gl-rs's top-level Apache declaration.
- Existing notices such as dpi's libm attribution, libm's source copyright list,
  and unicode-ident's Unicode terms are retained along with every original file
  in the matching source archive.

These recovery decisions are visible in the shipped notices. A zero missing-text
count is not a declaration that upstream authors have confirmed all copyright
provenance, that Apple SDK terms are resolved, or that a final release is approved.
No upstream messages, license changes or dependency upgrades were made.
