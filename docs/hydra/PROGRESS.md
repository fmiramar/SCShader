# Hydra implementation progress

Updated: 2026-11-12

| Step | Status | Evidence |
| --- | --- | --- |
| 01 — Repository and test-baseline audit | PASS (source map and compile check) | [BASELINE.md](BASELINE.md), source `48c31c42a4eff1f31f47632f29c21973c970ca07`; original PATH-based check passed; Step 03 corrected the toolchain record and explicitly passed `rustup run 1.97.1 cargo check --manifest-path renderer/Cargo.toml --locked`. Broader Rust/SC/GPU suites were not run. |
| 02 — Hydra canonical behavior/function inventory | PASS (source inventory) | [HYDRA_API_MATRIX.md](HYDRA_API_MATRIX.md): pinned upstream `9d29a9f`, 52 shader operations with declared defaults, runtime/media/array APIs, tiers, source evidence and SC selector collisions. Browser/native parity tests NOT RUN. |
| 03 — Architecture and ABI boundary | PASS (design/source inspection) | [ARCHITECTURE.md](ARCHITECTURE.md): bounded binary IR/OSC transactions, ownership/rollback, fixed-order output history, native `prev` difference and recovery. Pinned regl 1.7.0 package integrity checked; GPU/browser parity NOT RUN. |
| 04 — Minimal vertical slice | PASS (specification) | [VERTICAL_SLICE.md](VERTICAL_SLICE.md): startup/commit contract, pinned math and three focused checks; all implementation and GPU checks NOT RUN. |
| 05 — sclang AST front end | PASS (offline scope) | `Classes/SCHydra.sc`, `Classes/SCHydraExpr.sc`, `tests/sc/hydra_ast_unit.scd`: 4 registry operations, nested snapshots, bounded network-order IR, copy isolation and validation. Both offline AST and existing compile smoke pass with isolated class config. `.out` reports transport unavailable until Step 06; no renderer/GPU run. |
| 06 — Safe transport / atomic replacement | Transport PASS; activation pending | [Implemented staging protocol](../../protocol/hydra.md), bounded IR decoder and SC acknowledged upload. Pinned-toolchain Rust tests/Clippy and native Metal transport/legacy graph smokes pass. No committed output yet; Step 07 adds a candidate pipeline cache and Step 08 closes the activation gate. |
| 07 — Typed WGSL compiler | Compiler + native offscreen PASS; window display pending | [COMPILER.md](COMPILER.md): six operation variants, Naga validation, one GPU candidate cache and three 3x3 native Metal pixel cases. SC compiler/rollback path and legacy graph smoke pass. |

Current next step: Step 08 (output activation and window presentation), recommended
`gpt-6.1-sol` / `medium`. Request that effort switch before Step 08. Step 07 ran
on the reported `gpt-6.1-sol`; pipeline compilation and offscreen GPU pixels pass,
but the window still shows Legacy. Step 06's complete commit/revision rollback
gate remains pending Step 08. Do not label transport readiness or cached pipelines
as an accepted/running Hydra output.
