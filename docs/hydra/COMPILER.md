# Initial native Hydra compiler

2026-11-12. Step 07 implements a typed lowering and WGSL emitter for `solid`,
`osc`, `gradient`, `rotate`, `scale`, and `color`. The IR decoder and SC front end
accept these operations. `noise` and `blend` remain available to the offline AST,
but their shader lowering is deferred and returns a compile error explicitly.

Each lowered operation is a Rust enum variant with fixed scalar arguments. It
emits a scoped pure function `n<ID>(uv) -> vec4<f32>`. Coordinate nodes call their
receiver with transformed UV; color nodes call it first and modify the returned
RGBA. Helpers precede their callers in dependency order. Generated IDs and exact
f32 bit literals prevent identifier injection and decimal formatting overflow.
The compiler bounds node count, depth and generated source (256 KiB), then parses
and validates WGSL with the existing Naga dependency before GPU creation.

UV is normalized with the bottom-left visual convention. There is no implicit
aspect stretch correction or coordinate wrap in this procedural family, matching
the pinned operations; non-square outputs retain their normalized-coordinate
behavior. Sampling/wrap operators remain deferred. Oscillator phases and rotation
use the common `hydra.clock.x` time in seconds; coordinate transforms evaluate
before their receiving source. `color` preserves signed per-channel behavior,
including alpha; only the final Rgba8Unorm attachment clamps/quantizes results.

The fixed group-0/binding-0 frame ABI is 64 bytes, aligned to 16 bytes:

| Offset | Type | Components |
| --- | --- | --- |
| 0 | vec4<f32> clock | time seconds, speed, BPM, delta seconds |
| 16 | vec4<f32> resolution | width, height, inverse width, inverse height |
| 32 | vec4<f32> pointer | x, y, button state, reserved |
| 48 | vec4<u32> counters | frame counter, three reserved zeros |

Only time is consumed by this initial family. Literal inputs do not need the
ADR's reserved parameter block or source/output texture groups yet; those bindings
are not advertised as available. Step 08 supplies per-frame values from the
session clock. This checkpoint uses explicit fixed-time uniforms in the offscreen
proof and does not claim a running session clock or window presentation.

`Renderer` owns one compiled candidate (source, pipeline and bind-group layout).
Identical source reuses that candidate. Preparing a replacement validates CPU
lowering/Naga and the GPU pipeline under an error scope before replacing the
cache; failure keeps the old candidate. Close releases it; device recreation drops
it because no accepted Hydra output exists yet. This is not an inactive global
shader cache and does not allocate output textures in the running renderer.

A successful OSC compilation still returns `E_HYDRA_CAPABILITY` with the explicit
message that the pipeline compiled/cached but presentation activation is absent.
Failure returns `E_HYDRA_COMPILE`; no `committed` reply or revision increment is
fabricated. Step 08 must wire activation, presentation and last-picture rollback
before closing Step 06's full replacement gate.

Domain decisions: `osc` frequency zero is rejected because the pinned expression
has undefined frequency division; `scale` rejects zero/overflowed axis extents.
No arbitrary epsilon is substituted. Finite transport values do not guarantee
well-defined shader arithmetic for every extreme input; later parity audits must
retain explicit numerical limitations. General expression-to-scalar conversions,
dynamic parameters and custom functions are not implemented here.

## Focused proof

`cargo run --manifest-path renderer/Cargo.toml --locked --example hydra_compiler_check -- metal`
selects Metal explicitly, rejects a CPU adapter, and reads back three 3x3 Rgba8Unorm
images at time zero: signed solid/color, oscillator/rotation, and gradient with
asymmetric scale/off-center origin followed by rotation. The independent f64
reference uses the exact pixel-center UVs and allows at most two 8-bit values per
channel. Nonuniform scale and off-center origin make transform order observable.
An invalid candidate is rejected before each readback; the retained candidate
still renders the expected pixels. Reads use a five-second bounded GPU wait.

This passes on Apple M5/Metal. Unit validation, strict Clippy and the SC transport,
AST, class compile and legacy graph smokes also pass. See [verification](../VERIFICATION.md).
Window display/manual visual observation, four outputs, feedback, other backends,
and full Hydra parity remain NOT RUN or unimplemented as appropriate.
