# Dependency record

Updated 2026-09-23. Typed reflection reuses the existing Naga WGSL/GLSL frontends; no dependency was added for controls or interpolation. Direct dependency versions are exact in `renderer/Cargo.toml`; `renderer/Cargo.lock` fixes the full transitive graph.

| Component | Version | Purpose | Upstream license |
| --- | --- | --- | --- |
| Rust | 1.97.1 | Stable compiler/toolchain selected in `rust-toolchain.toml` | MIT or Apache-2.0 |
| `wgpu` | 30.0.0 | Cross-platform WebGPU-style rendering; Metal backend on macOS | MIT or Apache-2.0 |
| `image` | 0.25.8 | PNG/JPEG/PNM source texture decoding | MIT or Apache-2.0 |
| Naga GLSL frontend | bundled with `wgpu` 30.0.0 | Parser/validator used for the supported GLSL and Shadertoy fragment paths | MIT or Apache-2.0 |
| `pp-rs` | 0.2.1 | GLSL preprocessor enabled by wgpu's `glsl` feature | MIT or Apache-2.0 |
| `unicode-xid` | 0.2.6 | Unicode identifier support used by the GLSL preprocessor | MIT or Apache-2.0 |
| `winit` | 0.30.13 | Stable desktop window and event loop | Apache-2.0 |
| `rosc` | 0.11.4 | Small pure-Rust OSC 1.0 encoder/decoder | MIT or Apache-2.0 |
| `bytemuck` | 1.25.2 | Checked conversion of the uniform structure to GPU bytes | Zlib, Apache-2.0, or MIT |
| `pollster` | 1.0.1 | Minimal blocking executor for one-time GPU initialization/error-scope futures | Apache-2.0 or MIT |

These versions were the current stable releases inspected at the start of implementation and support the selected stable Rust release. `winit` 0.31 was still beta, so the renderer uses stable 0.30.13. No dependency is sourced from a moving Git branch.

Primary upstream references:

- Rust 1.97.1 release: <https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/>
- `wgpu` documentation: <https://docs.rs/wgpu/30.0.0/wgpu/>
- `winit` 0.30.13: <https://docs.rs/crate/winit/0.30.13>
- `rosc` 0.11.4: <https://docs.rs/rosc/0.11.4/rosc/>
- `bytemuck` 1.25.2: <https://docs.rs/crate/bytemuck/1.25.2>
- `pollster` 1.0.1: <https://docs.rs/crate/pollster/1.0.1>

Before a binary release, generate and review a complete third-party license inventory for every target from the locked graph. This file records direct-dependency compatibility but is not a substitute for the packaged notices.

The offline [notice-audit tool](NOTICE_AUDIT.md) now records target-specific
declarations, bundled texts/hashes, and exact upstream VCS revisions. Its first
macOS x64 audit found 132 dependencies, including 22 without bundled notice text.
Pinned upstream supplements now reduce that to ten unresolved full-text gaps;
Linux x64 and Windows x64 inventories have no remaining text-presence gaps.
Development packages include the unreviewed audit. Full distribution review and
the remaining macOS notices are still required; see the audit record for limits.
