# Hydra implementation progress

Updated: 2026-11-13

| Step | Status | Evidence |
| --- | --- | --- |
| 01 — Repository and test-baseline audit | PASS (source map and compile check) | [BASELINE.md](BASELINE.md), source `48c31c42a4eff1f31f47632f29c21973c970ca07`; original PATH-based check passed; Step 03 corrected the toolchain record and explicitly passed `rustup run 1.97.1 cargo check --manifest-path renderer/Cargo.toml --locked`. Broader Rust/SC/GPU suites were not run. |
| 02 — Hydra canonical behavior/function inventory | PASS (source inventory) | [HYDRA_API_MATRIX.md](HYDRA_API_MATRIX.md): pinned upstream `9d29a9f`, 52 shader operations with declared defaults, runtime/media/array APIs, tiers, source evidence and SC selector collisions. Browser/native parity tests NOT RUN. |
| 03 — Architecture and ABI boundary | PASS (design/source inspection) | [ARCHITECTURE.md](ARCHITECTURE.md): bounded binary IR/OSC transactions, ownership/rollback, fixed-order output history, native `prev` difference and recovery. Pinned regl 1.7.0 package integrity checked; GPU/browser parity NOT RUN. |
| 04 — Minimal vertical slice | PASS (specification) | [VERTICAL_SLICE.md](VERTICAL_SLICE.md): startup/commit contract, pinned math and three focused checks; all implementation and GPU checks NOT RUN. |
| 05 — sclang AST front end | PASS (offline scope) | `Classes/SCHydra.sc`, `Classes/SCHydraExpr.sc`, `tests/sc/hydra_ast_unit.scd`: 4 registry operations, nested snapshots, bounded network-order IR, copy isolation and validation. Both offline AST and existing compile smoke pass with isolated class config. `.out` reports transport unavailable until Step 06; no renderer/GPU run. |
| 06 — Safe transport / atomic replacement | PASS (one output) | [Implemented staging protocol](../../protocol/hydra.md), bounded IR decoder and SC acknowledged upload. Pinned-toolchain Rust tests/Clippy and native Metal transport/legacy graph smokes pass. Step 08 now verifies committed revisions, rejected edits and native output/Legacy coexistence. |
| 07 — Typed WGSL compiler | PASS (compiler/offscreen; live path verified in Step 08) | [COMPILER.md](COMPILER.md): six operation variants, Naga validation, one GPU candidate cache and three 3x3 native Metal pixel cases. SC compiler/rollback path and legacy graph smoke pass. |
| 08 — First output / presentation | PASS (automated native + installed example); user confirms 01_osc visual PASS | [OUTPUT.md](OUTPUT.md): o0 commits/replacements, presentation, resize, mode selection, cleanup and installed automatic discovery. Presenter pixels pass for linear/sRGB targets on Metal. |
| 09 — Procedural source/coordinate/color families | PASS for 32 procedural entries; full 34-entry acceptance DEFERRED | [FAMILIES.md](FAMILIES.md): all defaults/types, WGSL, SC methods; 32 native numerical reference cases and live chained commits pass. `src`/`prev` were deferred here; Steps 12–13 add output/history support, while media sampling remains deferred. |
| 10 — Nested combine family | PASS (native automated/installed scope) | [COMBINE.md](COMBINE.md): seven operations, nested branch transforms, alpha, rollback pixels and live revision checks. User confirms Step 10 works perfectly. |
| 11 — Spatial modulation | PASS (native automated/installed scope) | [MODULATION.md](MODULATION.md): all 11 operations, nested/current-UV evaluation, negative wrapping and rollback pixels; user confirms Step 11 works perfectly. |
| 12 — Four outputs and static sampling | PASS (native automated/installed scope) | [OUTPUTS.md](OUTPUTS.md): independent revisions, quadrants, static src reads, cycle rejection, resize and hush. Nine ordered visual checks installed and command-tested; user confirms black hush and gray bottom-right invert(0.5). Step 13 supersedes static scheduling. |
| 13 — Temporal buffering and feedback | PASS (native pixels, installed SC/example and logical recovery) | [TEMPORAL_SEMANTICS.md](TEMPORAL_SEMANTICS.md): fixed numeric order, paired histories, legal cycles, native prev, resets and bounded accounting. Upstream callback trace agrees; browser GPU parity unverified. |

| 14 — Live replacement and reclamation | PASS (native fixed-count GPU check; installed SC checks) | [LIVE_CODING.md](LIVE_CODING.md): active pipeline reuse, explicit clear, existing hush/disposal/mode contracts and 32 edits with resource ownership returning to baseline. |

Checkpoint A is available for one-output procedural visuals and live replacement.
The user confirms `01_osc.scd` runs perfectly; new Step 09 visuals have automated
native checks but await user observation. This is a development checkpoint.

Checkpoint B reached in the documented native scope. Next: Step 15, `gpt-6.1-sol` / `high`. Stop at the plan's model/effort
boundary: `/model gpt-6.1-sol --reasoning=high`, then `/status`.
The user authorized Step 13 continuation after the prior switch request; no
in-session tool independently reports the running model/effort pair. No automatic
model change is claimed. Media sources and complete browser/runtime parity remain
outside this checkpoint. The user reports the Step 13 feedback example works properly in the desktop
window; browser pixel parity remains unverified.
