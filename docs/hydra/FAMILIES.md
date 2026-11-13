# Procedural family checkpoint — 2026-11-13

Step 09 implements 32 procedural operations: six generators, all ten coordinate
transforms and all sixteen color operations. The registry records pinned argument
names, types and defaults; explicit SC methods build bounded IR and the typed
compiler emits WGSL. Capability discovery advertises exactly these 32 names.
`sum` accepts a literal four-component vector, copied from caller storage and
encoded with HYIR tag 5. Other arguments remain finite scalar literals.

The eight-entry upstream src category also contains `src` and `prev`. These need
output bindings/history (Steps 12–13) or media sources (Step 19); they are deferred,
not placeholders. Thus the procedural Checkpoint A works, while the original
Step 09 acceptance requiring no missing inventory entry remains incomplete.
Nested combines are Step 10; dynamic parameters and arrays are later work.

`examples/hydra/02_procedural.scd` offers noise/kaleid/posterize, polygon/repeat/
rotate and voronoi/scroll/colorama chains. Evaluate the startup block, then one
replacement block at a time after its commit completes. It uses a separate port
from `01_osc.scd`. Recompile the SC class library to load newly installed methods
before starting it. The user has confirmed the earlier `01_osc.scd` visually;
manual observation of the new example remains open.

## Semantics and bounds

Coordinate transforms evaluate the receiver at transformed UV, without allocating
extra passes. Negative wrapping uses floor-based modulo. Repeat applies its X
offset before deriving the Y offset. Shape preserves the reference angle ordering
and constants; kaleid intentionally does not add a recentering offset. `shift`
adds the fractional offsets to RGBA and does not wrap the final color. `sum`
replicates the weighted RGBA sum into RGB and preserves the original alpha;
`r/g/b/a` replicate the selected scaled/offset channel into all four channels.
Luma changes alpha, while thresh preserves it. RGBA8 output clamps at the render
target; intermediate expressions are not globally clamped.

Zero divisors for shape, kaleid, pixelate and posterize are rejected; negative
threshold tolerances are rejected. Existing osc/scale restrictions still apply.
Upstream undefined domains, extreme finite values, transcendental precision,
pow on negative inputs and smoothing edge cases are not qualified by this check.
No full browser, frame timing, arbitrary-input or cross-backend parity is claimed.

## Reference and provenance

Behavior and metadata are pinned to hydra-synth
`9d29a9f4fd8f9081b9759943f38db36f05b9a88f`; see the linked inventory for sources.
Production WGSL was written independently from mathematical behavior, without
incorporating upstream shader function bodies. Noise uses a scalar/corner
implementation of simplex mathematics; background is Gustavson's
[webgl-noise](https://stegu.github.io/webgl-noise/) and
[Ashima/Gustavson webgl-noise](https://github.com/ashima/webgl-noise) (MIT).
This records implementation provenance, not a general licensing clearance.

The development-only `hydra_family_check` reads original GLSL at runtime from
pinned Git objects in a separately supplied external reference checkout. No
reference GLSL bodies are embedded or shipped by this repository or renderer.
It compiles those reference fragments through Naga and compares them to the
production WGSL on the same native adapter. This tests shader formula behavior,
not the Hydra browser runtime. Run with the pinned Rust toolchain:

```sh
cargo run --manifest-path renderer/Cargo.toml --locked --example hydra_family_check -- build/hydra-reference metal
```

All 32 cases pass on Apple M5/Metal: nine RGBA8 pixels each at time 0.37, maximum
channel difference zero (allowed tolerance two). Inputs include nontrivial
noise/voronoi/shape parameters, negative repeat coordinates and offsets, negative
scroll wrapping, negative fractional shift and weighted vec4 sum. Source SC
checks also verify all defaults, vector copy isolation/validation and four serial
chained commit/present cases. Exact build/install evidence is in
[VERIFICATION.md](../VERIFICATION.md).
