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
| 09 — Procedural source/coordinate/color families | PASS for 32 procedural entries; full 34-entry acceptance DEFERRED | [FAMILIES.md](FAMILIES.md): all defaults/types, WGSL, SC methods; 32 native numerical reference cases and live chained commits pass. `src`/`prev` require later sampling/history work. |
| 10 — Nested combine family | PASS (native automated/installed scope) | [COMBINE.md](COMBINE.md): seven operations, nested branch transforms, alpha, rollback pixels and live revision checks. User confirms Step 10 works perfectly. |
| 11 — Spatial modulation | PASS (native automated/installed scope) | [MODULATION.md](MODULATION.md): all 11 operations, nested/current-UV evaluation, negative wrapping and rollback pixels; new demo awaits user observation. |

Checkpoint A is available for one-output procedural visuals and live replacement.
The user confirms `01_osc.scd` runs perfectly; new Step 09 visuals have automated
native checks but await user observation. This is a development checkpoint.

Next: Step 12, `gpt-6.1-sol` / `high` (same planned pair).
The user's Step 10 visual confirmation is recorded; Step 11 manual observation
remains open. `src`/`prev`, multiple outputs, feedback and other-platform parity
remain later scope. No runtime model switch is inferred from plan text.
