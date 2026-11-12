# Hydra implementation progress

Updated: 2026-11-12

| Step | Status | Evidence |
| --- | --- | --- |
| 01 — Repository and test-baseline audit | PASS (source map and compile check) | [BASELINE.md](BASELINE.md), source `48c31c42a4eff1f31f47632f29c21973c970ca07`; `cargo check --manifest-path renderer/Cargo.toml --locked` passed. Broader Rust/SC/GPU suites were not run. |
| 02 — Hydra canonical behavior/function inventory | PASS (source inventory) | [HYDRA_API_MATRIX.md](HYDRA_API_MATRIX.md): pinned upstream `9d29a9f`, 52 shader operations with declared defaults, runtime/media/array APIs, tiers, source evidence and SC selector collisions. Browser/native parity tests NOT RUN. |

Current next step: Step 03 (architecture). The plan assigns `gpt-6-astra` / `xhigh` to this checkpoint; request that model switch before executing Step 03. Keep implementation opt-in and additive to the existing `/scshader/v1/` protocol and linear `ShaderGraph` path. Do not start feature code before Steps 02–04 define the pinned compatibility target, architecture and vertical slice.
