# Hydra implementation baseline

Date: 2026-11-12
SCShader source: `48c31c42a4eff1f31f47632f29c21973c970ca07` (`main`, clean after fast-forward)

This is the Step 01 source and build baseline for the proposed opt-in Hydra subsystem. It records what was inspected and does not claim Hydra functionality exists.

## Relevant source map

| Concern | Current owner | Contract / evidence |
| --- | --- | --- |
| Renderer process, CLI and window startup | `renderer/src/main.rs`, `renderer/src/app.rs` | CLI binds the renderer to its configured OSC port; OSC receiver binds IPv4 loopback (`renderer/src/osc.rs`). |
| OSC decoding, validation and dispatch model | `renderer/src/osc.rs`, `renderer/src/protocol.rs` | `/scshader/v1/...`; bounded packet decoding and command parsing; new Hydra messages need distinct addresses and limits. |
| Protocol documentation | `protocol/protocol.md` | Existing schemas, replies, compatibility behavior, graph limitations and resource bounds. |
| Shader compilation and GPU resources | `renderer/src/renderer.rs` | Owns pipelines, GPU resources, graph render targets and shader replacement. |
| SuperCollider process/API lifecycle | `Classes/ShaderServer.sc` | Server process, boot/quit, OSC endpoint and replies. |
| Shader resources and graph API | `Classes/Shader.sc`, `Classes/ShaderGraph.sc` | `ShaderGraph` validates one linear pass chain and sends `/scshader/v1/graph/set`; it is not a Hydra DAG. |
| Texture, feedback, analysis/audio inputs | `Classes/ShaderTexture.sc`, `ShaderBuffer.sc`, `ShaderFFTTexture.sc`, `ShaderWaveformTexture.sc`, `ShaderAnalysis.sc`, corresponding Rust protocol/renderer handlers | Existing facilities to inspect/reuse; no assumption that they provide Hydra source-slot semantics. |
| Existing tests and docs | `tests/sc/`, `renderer` unit tests, `tests/python/`, `HelpSource/` | Existing focused smoke suites, protocol tests and SCDoc harness; Hydra coverage is not present in this baseline. |

## Existing behavior constraints

- The OSC protocol is versioned at `/scshader/v1/` and loopback-only. `graph/set` accepts a validated **linear render order**, not an arbitrary graph; the SC `ShaderGraph` performs topology checks.
- A shader reload compiles a candidate before replacing the active pipeline; failure preserves the prior valid pipeline.
- Existing graph targets are renderer-owned and cleared/recreated on resize. Renderer/OSC queues and packet bounds are documented in the protocol and source.
- Existing public classes include `ShaderGraph`; choose a distinct Hydra class/API name and preserve `\shader` Event behavior.
- Project instructions are in `AGENTS.md`, `START_HERE.md`, `docs/STATUS.md`, `docs/PLATFORM_MILESTONE.md`, `docs/plans/COMMON.md`, and `docs/plans/MACOS_ARM64.md`. Current development version is 0.0.18. Hydra is separate proposed work; platform qualification and existing release gates remain distinct.

## Host and baseline check

- OS: macOS 26.6.2; process/host architecture: arm64 / `aarch64-apple-darwin`.
- GPU reported by `system_profiler SPDisplaysDataType`: Apple M5, Metal 4. An external HP display and built-in Retina display were connected. This is inventory, not a Hydra or renderer visual qualification.
- SuperCollider: `sclang 3.14.1`; executable entry on PATH is a shell launcher. Its final process architecture was not independently checked.
- Pinned Rust toolchain: 1.97.1 (`rust-toolchain.toml`). Correction from Step 03: rustup reports that configured toolchain, but bare `cargo`/`rustc` on this host resolve to Homebrew 1.98.1. The original command below was not a pinned-toolchain check. Use `rustup run 1.97.1 cargo ...` here.
- Git state after update: clean at the source commit above.
- Command: `cargo check --manifest-path renderer/Cargo.toml --locked` — **PASS**, finished dev profile in 7.67 seconds. This is a compile check only; no Rust unit tests, SC integration tests, packaging, or GPU visual tests were run for this step.
- Corrected check at source `94b9f1a03395d48b7b84a9386ad7c2951b44a137` (only documentation changes since the original source): `rustup run 1.97.1 cargo check --manifest-path renderer/Cargo.toml --locked` — **PASS**, dev profile finished in 0.17 seconds. `rustup run 1.97.1 rustc -V` confirms rustc 1.97.1. No linked build, unit suite or GPU run is implied.
- `git pull --ff-only origin main` fast-forwarded `main` from `c90c9e9` to `48c31c4` before the check.

## Not established

No Hydra upstream SHA or exhaustive API inventory is pinned yet. No SC architecture/API collision audit beyond the inspected class names is complete. No renderer executable was built or run, and no output pixels, media input, or GPU recovery behavior was tested as part of this baseline.

## Toolchain follow-up (Step 06)

Selecting Cargo via `rustup run` alone did not isolate all subprocesses from this
host's Homebrew PATH. The Step 03 explicit Cargo check is not independent proof
that every compiler/subtool was pinned. Step 06 prepends the directory returned
by `dirname "$(rustup which --toolchain 1.97.1 cargo)"` to PATH for the command
process; `rustc -V` and `cargo clippy -V` then report 1.97.1 / 0.1.97. The native
build, strict lint and unit tests pass under that environment; see
[verification](../VERIFICATION.md#hydra-transport-staging--2026-11-12).
