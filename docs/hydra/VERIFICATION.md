# Hydra compatibility and regression audit — Step 25

Date: 2026-11-19. Development version **0.0.18**, not final-release qualification.

## Source and execution scope

Baseline source: `42b0c94ad3acbefbd58c0e08a07fba9821ab7fa7` (Step 24).
The accompanying Step 25 implementation corrects signed sequence offsets and
adds bounded audit fixtures. Implementation commit: **c3a0c7912caa0619a6f7675680568fb1dc250a9e**.
Pinned hydra-synth reference:
[`9d29a9f4fd8f9081b9759943f38db36f05b9a88f`](https://github.com/hydra-synth/hydra-synth/tree/9d29a9f4fd8f9081b9759943f38db36f05b9a88f).
Reference code is read from committed Git objects in the external ignored
checkout; upstream GLSL bodies are not copied into this project.

Native environment: macOS 26.6.2 arm64, Apple M5 / **Metal**, SuperCollider
3.14.1, pinned Rust 1.97.1; target `aarch64-apple-darwin`. Dependencies,
`Cargo.lock`, version and OSC schemas are unchanged. No GPU test hooks are shipped.
The installed ordinary renderer SHA-256, before and after this audit, is:

```text
60e3d99e52b932a9b39811481099772f54a78567b3788c83fb59648519ae55b1
```

The functional correction is in `SCHydraNumber.sc`; Rust changes affect test
fixtures only. Identical renderer bytes are expected. Local native binary and
matching vendored-source archives use prefix `fmiramar-hydra25-reviewed`.
The installed extension is backed up outside Extensions before replacement;
all 156 final files are verified against that archive. The tested classes,
examples, shaders and renderer executable match the candidate; documentation
is updated with these results. No release/tag/upload occurs.

## Matrix coverage and proof index

The [API matrix](HYDRA_API_MATRIX.md) retains **113 classified rows**: 52 shader,
32 runtime/service, 20 array/dynamic and nine additional inspected surfaces.
Counts remain 59 SUPPORTED, 30 NATIVE ALTERNATIVE, four ADAPTER REQUIRED and
20 UNSUPPORTED. Grouped rows and argument limits preclude treating these counts
as a compatibility percentage. The source completeness check passes against the
pinned shader categories, 14 easing names, 13 constructor options and inspected
public method sets. No skipped function is removed or silently marked complete.

Every Tier A shader row maps to the paths and proofs below. Tier A service and
numeric rows retain their documented supported/alternative/unsupported boundary;
unsupported JS callbacks, stepping and argument coercions are not accepted by
these tests. Tier B behavior is defined in the linked native contracts. Tier C
browser/network adapters remain explicit unimplemented work.

| Scope | Implementation | Proof rerun in this audit | Contract |
| --- | --- | --- | --- |
| 32 procedural, coordinate and color operations | `registry.rs`, `compiler.rs`, `families.rs`, `procedural.wgsl`; `SCHydraExpr.sc` | Registry/default audit; Rust validation; 32 cases in the 50-operation native comparator; SC family unit/commit/example checks | [Families](FAMILIES.md), [compiler](COMPILER.md) |
| Seven combines | `compiler.rs`, `gpu.rs` | Seven reference comparisons, nested alpha corpus, SC combine unit/commit/example checks | [Combine](COMBINE.md) |
| 11 spatial modulations | `modulation.rs`, `compiler.rs`, `gpu.rs` | Eleven reference comparisons, composed current-UV corpus, SC modulation unit/commit/example checks | [Modulation](MODULATION.md) |
| src/prev, four outputs, feedback and clear/hush | `outputs.rs`, `output.rs`, `mod.rs`; `SCHydra.sc` | Production output GPU fixture: exact temporal tags, cycles, self/prev, source orientation/pixels, failed-edit rollback, resize and disposal; SC output/feedback checks | [Temporal semantics](TEMPORAL_SEMANTICS.md), [outputs](OUTPUTS.md), [live coding](LIVE_CODING.md) |
| Numeric values, arrays and clock | `parameters.rs`, `clock.rs`, `compiler.rs`; `SCHydraNumber.sc`, `SCHydra.sc` | Pinned JSC array/temporal traces; signed-offset regression; Rust unit tests; uniform pixels; SC parameters/clock/two-run example | [Parameters](PARAMETERS.md), [clock](CLOCK.md) |
| Patterns, audio and input | `SCHydra.sc`, existing `ShaderBus`/`ShaderAnalysis`/`ShaderWindow`, `Shader.sc` | Installed Hydra audio/Pattern checks and legacy bus/analysis/window/Pattern tests; no new listening qualification | [Audio](AUDIO.md), [Patterns](PATTERNS.md), [compatibility](COMPATIBILITY.md) |
| Images/video/capture | `outputs.rs`, `video.rs`, `capture.rs`, `camera.rs`; `SCHydra.sc` | Static source and 32 in-place RGBA upload GPU checks; installed image check. Prior opt-in video/capture hardware evidence remains separate; no new device activation | [Images](IMAGES.md), [video](VIDEO.md), [capture](CAPTURE.md) |
| Custom WGSL | `custom.rs`, `compiler.rs`; `SCHydra.sc` | All five categories, defaults/vec4/number inputs, invalid edit retention and frozen definitions in production GPU fixture; installed custom check | [Custom functions](CUSTOM_FUNCTIONS.md) |
| Export/resolution | `export.rs`, `outputs.rs`; `SCHydra.sc` | Native export pixel/disposal fixture; explicit PNG/MP4 decode and queued installed example checks | [Export](EXPORT.md) |

Rust paths above are relative to `renderer/src/hydra/`; SC classes are in
`Classes/`. The differential fixture is
[`hydra_family_check.rs`](../../renderer/examples/hydra_family_check.rs); the
production temporal/resource fixture is
[`hydra_outputs_check.rs`](../../renderer/examples/hydra_outputs_check.rs).
Unit tests remain in the implementation modules and `tests/sc/`.

## Defaults, precision and composed corpus

`check_hydra_defaults.py` evaluates only the pinned definition metadata in macOS
JavaScriptCore and compares all **52** input names/types/defaults to the native
registry at f32 precision. All match, including implicit combine texture inputs
and literal vec4 defaults. Upstream `src` uses NaN to mark a required texture;
the native explicit slot API defaults to o0. Arbitrary JS texture objects remain
unsupported. This difference is reported, not normalized into a false match.

The existing 50 non-texture operation checks pass at 3×3 pixels. Ten additional
font-free compositions pass at 64×36, fixed time 0.37 seconds, RGBA8 output.
Each saves native/reference PNGs under ignored local evidence. The maximum
allowed byte-channel difference stays **2**; no tolerance or assertion is relaxed.

| Corpus case | Behavior sampled | Maximum channel difference |
| --- | --- | --- |
| rgba-weights | Signed color weights, alpha and vec4 sum | 0 |
| coordinate-order | Rotate/scale/scroll nesting and time-dependent UV | 0 |
| nested-alpha | Nested blend and layer with fractional alpha | 0 |
| nested-modulation | Current-UV nested modulation, kaleid and transforms | 0 |
| negative-repeat | Negative repeat/scroll wrapping | 0 |
| luma-mask | Smooth luma, transformed mask and alpha | 0 |
| posterize-hue | Saturation, hue, gamma quantization and shift | 0 |
| default-shape | Default polygon/repeat arguments and rotation | 0 |
| gradient-channels | Channel extraction, invert and nonuniform scale | 0 |
| noise-warp | Simplex noise through transforms and color | 0 |

The reference GLSL and independent native WGSL are compiled through Naga and
executed on the same physical Metal GPU. This isolates translation/composition
behavior in these samples. It is **not a browser/WebGL differential run**;
shared compiler behavior, browser mediump/highp, all floating-point domains and
all resolutions/times remain outside its proof. Native history is RGBA8, with
explicit quantization/clamping. Temporal feedback has separate multi-frame
production checks rather than a single screenshot. Native `prev` deliberately
avoids upstream attachment aliasing; see the temporal contract.

## Failure found and corrected

Before the fix, `h.seq([0,1]).offset(-0.25)` serialized phase **+0.75** because
SC modulo wraps positive. Pinned JavaScript remainder preserves **−0.25**. For a
smoothed sequence, that whole-step shift changes the visual value. The failing
SC payload/log is retained under `numeric-before/`, not replaced by passing data.

The serializer now computes signed remainder and validates the supplied phase
before mutation. Tests cover ±fractional phases, whole negative cycles and
nonfinite rejection preserving the previous valid offset. Pinned JSC and Rust
checks both verify the smoothed value **0.75 at time zero** for phase −0.25.
Source and installed SC regressions pass after the fix. Only linear and
`easeInOutQuad` remain implemented; no additional easing/dynamic coverage is
claimed. Recompile the SC class library to load the installed class correction.

An initial Python discovery command used `tests` and found zero tests (nonzero
exit); the corrected `tests/python` command supplies the result below. A local
install script also rejected ZIP container paths. The backup was restored, all
files were validated before swapping the candidate into place, and the partial
SC run was interrupted and repeated from a fresh evidence directory. These
harness failures do not count as regression passes.

## Commands and results

Run from the repository root. Use a native interactive macOS desktop and fresh
evidence directories. The external reference checkout must contain the pin.
Define `renderer` as the absolute installed executable and `sc_config` as an
isolated class-library YAML including the installed Classes and SC standard
library, excluding duplicate development classes. Put the pinned Rust binaries
first on PATH for Cargo subprocesses:

```sh
task_rust_bin="$(dirname "$(rustup which --toolchain 1.97.1 cargo)")"
export PATH="$task_rust_bin:$PATH"
cargo fmt --manifest-path renderer/Cargo.toml --check
cargo clippy --manifest-path renderer/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path renderer/Cargo.toml --locked --all-targets --all-features
python3 -m unittest discover -s tests/python -v
python3 tools/check_hydra_compatibility.py --reference build/hydra-reference
python3 tools/check_hydra_defaults.py --reference build/hydra-reference
python3 tools/check_hydra_parameters_reference.py
python3 tools/check_hydra_temporal_reference.py --reference build/hydra-reference --output-dir build/hydra-tests/verification-25/temporal-reference
cargo run --manifest-path renderer/Cargo.toml --locked --example hydra_family_check -- build/hydra-reference metal
cargo run --manifest-path renderer/Cargo.toml --locked --example hydra_family_check -- build/hydra-reference metal --corpus "$PWD/build/hydra-tests/verification-25/corpus-01"
cargo run --manifest-path renderer/Cargo.toml --locked --example hydra_outputs_check -- metal
python3 tools/run_sc_checks.py --renderer "$renderer" --sclang-config "$sc_config" --output-dir build/hydra-tests/verification-25/installed-sc-02 --audio-input-channels 0
python3 tools/check_help_examples.py --extension "$extension" --sclang-config "$sc_config" --output-dir build/hydra-tests/verification-25/help --audio-input-channels 0
python3 tools/check_hydra_export.py --renderer "$renderer" --sclang-config "$sc_config" --ffmpeg "$ffmpeg" --ffprobe "$ffprobe" --output-dir build/hydra-tests/verification-25/export
python3 tools/check_hydra_scene_profile.py --renderer "$renderer" --output-dir build/hydra-tests/verification-25/profile-after
SCSHADER_ARTIFACT_PREFIX=fmiramar-hydra25-reviewed bash tools/package_macos.sh arm64
python3 tools/package_corresponding_source.py --verify dist/fmiramar-hydra25-reviewed-SCShader-0.0.18-macos-arm64-corresponding-source.zip
```

`extension` is the absolute installed SCShader directory; `ffmpeg` and `ffprobe`
are the explicitly selected external tools (9.0.1 here). The temporal recorder
verifies the pinned regl 1.7.0 archive integrity, then executes source callbacks
in a recorder; it does not execute regl/WebGL. GPU/live suites run serially.

| Check | Result |
| --- | --- |
| Rust formatting and strict all-target/all-feature Clippy | PASS |
| Rust tests | PASS: 97 main tests; example-module tests also pass. Two existing opt-in native media tests ignored, with reasons retained |
| Python tests | PASS: 138 discovered, 133 passed, five native Windows API/job/power tests skipped |
| API/default/array/temporal reference checks | PASS within the scopes above |
| Metal operation comparisons and composed corpus | PASS: 50 operations plus ten compositions |
| Production Metal output/feedback/source/custom/export/resource fixture | PASS: 32 edits; all eight tracked resource classes return to baseline |
| Installed SuperCollider suite | PASS: all 53 default checks, including the new numeric edge regression |
| SCDoc/help/legacy demos | PASS: 13 SCDoc pages rendered without warnings/errors; 18 installed demos / 36 setup-cleanup blocks, including legacy graph/Pattern demos |
| Explicit export and installed queued example | PASS: both opt-in SC checks; channel/orientation decode, two queued PNGs and H.264 MP4; failure recovery |
| Final binary/source package and installed file hashes | PASS: ordinary arm64 package, vendored-source inventory and all 156 installed files verified against the final ZIP |

Raw logs, pixel artifacts, manifests and profile samples are local ignored
`build/hydra-tests/verification-25/`; they are not source or release assets.

## Brief 720p profile

One nine-node osc/noise/shape scene with kaleid, transforms, blend and modulation
uses fixed 1280×720 render pixels and a 1280×720 physical window. The harness
warms up for one second and samples for three seconds, once before and once
after. There is no broad benchmark or performance optimization.

| Metric | Before | After |
| --- | --- | --- |
| Ordinary renderer hash | Same SHA-256 above | Same SHA-256 above |
| Median reported FPS | 60.334 | 60.175 |
| Median CPU frame interval, including pacing | 16.400 ms | 16.384 ms |
| Maximum sampled CPU frame interval | 17.035 ms | 16.960 ms |
| Owned texture byte estimate | 22,118,664 | 22,118,664 |
| Queue/recovery/rejected-packet counters | Zero | Zero |

GPU timestamp measurement is unavailable (`gpuFrameMs=-1` in the raw samples).
The baseline harness initially mislabeled this sentinel in its descriptive text;
the raw values are retained and the final harness description is corrected.
Texture bytes are the renderer's owned allocation estimate, not whole-device
memory, allocation churn or process RSS. CPU frame interval includes vsync and
is not GPU execution time. Identical binaries and these brief samples cannot
establish a speedup, regression threshold or long-run memory qualification.

## Remaining limits and next work

- Native Hydra acceptance here covers Apple M5/Metal only. New Hydra behavior
  on Windows/D3D12, Linux/Vulkan and Intel macOS remains **NOT RUN**; historical
  legacy platform results do not substitute for it.
- Browser/WebGL pixels, browser texture-object coercion, arbitrary JS callbacks,
  DOM/network/WebRTC/live external-texture adapters remain unqualified or
  unimplemented as classified in the matrix.
- Dynamic arguments still cover five builtin operations/custom float inputs;
  twelve reference easing curves are unsupported. Those are explicit feature
  gaps for a subsequent implementation, not hidden audit failures.
- Camera/display permissions, denial/unplug and live device startup are not
  retested here; retain the qualified Step 21 scope and user observations.
- Physical audiovisual alignment/listening, other devices/displays, real GPU
  resets, long recordings and final-release duration gates remain separate.
  No one-hour/eight-hour soak was started.

Step 25 acceptance is bounded native regression and complete disclosure, not
full Hydra equivalence. Step 26 documentation/examples/handoff and Step 27
independent review remain in the original plan. Any broader numeric/easing
implementation requires its own scoped follow-up and renewed compatibility proof.
