# Hydra implementation progress

Updated: 2026-11-12

| Step | Status | Evidence |
| --- | --- | --- |
| 01 — Repository and test-baseline audit | PASS (source map and compile check) | [BASELINE.md](BASELINE.md), source `48c31c42a4eff1f31f47632f29c21973c970ca07`; original PATH-based check passed; Step 03 corrected the toolchain record and explicitly passed `rustup run 1.97.1 cargo check --manifest-path renderer/Cargo.toml --locked`. Broader Rust/SC/GPU suites were not run. |
| 02 — Hydra canonical behavior/function inventory | PASS (source inventory) | [HYDRA_API_MATRIX.md](HYDRA_API_MATRIX.md): pinned upstream `9d29a9f`, 52 shader operations with declared defaults, runtime/media/array APIs, tiers, source evidence and SC selector collisions. Browser/native parity tests NOT RUN. |
| 03 — Architecture and ABI boundary | PASS (design/source inspection) | [ARCHITECTURE.md](ARCHITECTURE.md): bounded binary IR/OSC transactions, ownership/rollback, fixed-order output history, native `prev` difference and recovery. Pinned regl 1.7.0 package integrity checked; GPU/browser parity NOT RUN. |
| 04 — Minimal vertical slice | PASS (specification) | [VERTICAL_SLICE.md](VERTICAL_SLICE.md): startup/commit contract, pinned math and three focused checks; all implementation and GPU checks NOT RUN. |
| 05 — sclang AST front end | PASS (offline scope) | `Classes/SCHydra.sc`, `Classes/SCHydraExpr.sc`, `tests/sc/hydra_ast_unit.scd`: 4 registry operations, nested snapshots, bounded network-order IR, copy isolation and validation. Both offline AST and existing compile smoke pass with isolated class config. `.out` reports transport unavailable until Step 06; no renderer/GPU run. |
| 06 — Safe transport / atomic replacement | PASS (one output) | [Implemented staging protocol](../../protocol/hydra.md), bounded IR decoder and SC acknowledged upload. Pinned-toolchain Rust tests/Clippy and native Metal transport/legacy graph smokes pass. Step 08 now verifies committed revisions, rejected edits and native output/Legacy coexistence. |
| 07 — Typed WGSL compiler | PASS (compiler/offscreen; live path verified in Step 08) | [COMPILER.md](COMPILER.md): six operation variants, Naga validation, one GPU candidate cache and three 3x3 native Metal pixel cases. SC compiler/rollback path and legacy graph smoke pass. |
| 08 — First output / presentation | PASS (automated native + installed example); manual observation NOT RUN | [OUTPUT.md](OUTPUT.md): o0 commits/replacements, presentation, resize, mode selection, cleanup and installed automatic discovery. Presenter pixels pass for linear/sRGB targets on Metal. |

Current next step: Step 09 (remaining generator/coordinate/color families), the
same recommended `gpt-6.1-sol` / `medium` pair; no model change is needed.
The installed development build now runs the first slice. Keep manual visual,
feedback/recovery, other-platform and full-parity claims separate from this
short native/installed acceptance. Step 13 still owns temporal feedback proof.
