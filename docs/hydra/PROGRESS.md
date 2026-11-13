# Hydra implementation progress

Updated: 2026-11-12

| Step | Status | Evidence |
| --- | --- | --- |
| 01 — Repository and test-baseline audit | PASS (source map and compile check) | [BASELINE.md](BASELINE.md), source `48c31c42a4eff1f31f47632f29c21973c970ca07`; original PATH-based check passed; Step 03 corrected the toolchain record and explicitly passed `rustup run 1.97.1 cargo check --manifest-path renderer/Cargo.toml --locked`. Broader Rust/SC/GPU suites were not run. |
| 02 — Hydra canonical behavior/function inventory | PASS (source inventory) | [HYDRA_API_MATRIX.md](HYDRA_API_MATRIX.md): pinned upstream `9d29a9f`, 52 shader operations with declared defaults, runtime/media/array APIs, tiers, source evidence and SC selector collisions. Browser/native parity tests NOT RUN. |
| 03 — Architecture and ABI boundary | PASS (design/source inspection) | [ARCHITECTURE.md](ARCHITECTURE.md): bounded binary IR/OSC transactions, ownership/rollback, fixed-order output history, native `prev` difference and recovery. Pinned regl 1.7.0 package integrity checked; GPU/browser parity NOT RUN. |
| 04 — Minimal vertical slice | PASS (specification) | [VERTICAL_SLICE.md](VERTICAL_SLICE.md): startup/commit contract, pinned math and three focused checks; all implementation and GPU checks NOT RUN. |
| 05 — sclang AST front end | PASS (offline scope) | `Classes/SCHydra.sc`, `Classes/SCHydraExpr.sc`, `tests/sc/hydra_ast_unit.scd`: 4 registry operations, nested snapshots, bounded network-order IR, copy isolation and validation. Both offline AST and existing compile smoke pass with isolated class config. `.out` reports transport unavailable until Step 06; no renderer/GPU run. |

Current next step: Step 06 (safe OSC transport and atomic replacement). The plan assigns `gpt-6.1-sol` / `high`; request the model/effort switch before executing Step 06. Step 05 ran on `gpt-6-sol` as reported by the session, a deviation from its `gpt-6.1-sol` recommendation. Keep implementation opt-in and additive to the existing `/scshader/v1/` protocol and linear `ShaderGraph` path. Steps 02–04 define the pinned compatibility target, architecture and vertical slice; feature implementation starts in Step 05.
