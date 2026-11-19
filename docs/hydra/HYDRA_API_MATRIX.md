# Pinned Hydra API inventory

Reference: `hydra-synth` commit **9d29a9f4fd8f9081b9759943f38db36f05b9a88f**, retrieved 2026-11-12. The reference checkout is ignored under `build/hydra-reference`. Only API metadata and independently written behavior notes are included here; no upstream shader implementation is incorporated.

Updated through Step 24 (2026-11-19), against SCShader source `532af16`.
This inventory covers 52 shader definitions, the original 32 runtime/service rows,
20 array/dynamic rows and nine additional inspected source surfaces. Every row has
an explicit compatibility status; definitions and browser/adapter decisions are in
[COMPATIBILITY.md](COMPATIBILITY.md). Tier A is synthesis, B native media/input
services, C browser/network integration. Tier identifies responsibility, not parity.

`SUPPORTED` means the documented bounded native contract is implemented, not that
JavaScript syntax, arbitrary arguments or browser GPU pixels match. All 52 shader
names are recognized within their declared types/domains: finite scalar literals,
literal vec4 for sum, and explicit expression/slot references where required.
Native numeric arguments are supported only in osc/solid/color/rotate/scale and
custom float inputs. JS function callbacks, implicit color-expression-to-scalar
coercion and automatic texture-object-to-expression conversion are unsupported.
Use explicit src/sum/native numeric handles. Browser GPU equality remains NOT RUN.
Existing native evidence is linked per family and service; Step 24 is a source
and documentation audit, not a new hardware qualification.

## Shader operations

52 exported definitions discovered in `src/glsl/glsl-functions.js`: src: 8, coord: 10, color: 16, combineCoord: 11, combine: 7.

Declared arguments below exclude the implicit receiving chain. The factory prepends a required second expression for `combine` and `combineCoord`. Defaults are source metadata, not inferred documentation defaults. Each source link identifies the authoritative math and channel expression for later parity verification.

| Operation | Category | Arguments/defaults | Behavior and reference | Tier / status |
| --- | --- | --- | --- | --- |
| `noise` | src | `scale:float=10; offset:float=0.1` | Simplex noise; opaque grayscale. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L78) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `voronoi` | src | `scale:float=5; speed:float=0.3; blending:float=0.3` | Animated nearest-cell pattern; opaque RGB. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L96) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `osc` | src | `frequency:float=60; sync:float=0.1; offset:float=0` | Sine RGB channels with phase offsets; opaque. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L144) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `shape` | src | `sides:float=3; radius:float=0.3; smoothing:float=0.01` | Regular polygon distance and soft edge; opaque grayscale. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L171) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `gradient` | src | `speed:float=0` | UV-derived color; opaque. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L199) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `src` | src | `tex:sampler2D=NaN` | Samples external/output texture at wrapped UV; RGBA preserved. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L212) | A / SUPPORTED: output/image/local video/macOS capture slots; browser URL/DOM sources excluded; [evidence](IMAGES.md) |
| `solid` | src | `r:float=0; g:float=0; b:float=0; a:float=1` | Constant RGBA; alpha supplied. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L226) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `rotate` | coord | `angle:float=10; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L254) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `scale` | coord | `amount:float=1.5; xMult:float=1; yMult:float=1; offsetX:float=0.5; offsetY:float=0.5` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L276) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `pixelate` | coord | `pixelX:float=20; pixelY:float=20` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L313) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `posterize` | color | `bins:float=3; gamma:float=0.6` | Quantizes RGB with gamma; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L332) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `shift` | color | `r:float=0.5; g:float=0; b:float=0; a:float=0` | Adds fractional RGBA offsets; resulting channels are not wrapped. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L355) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `repeat` | coord | `repeatX:float=3; repeatY:float=3; offsetX:float=0; offsetY:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L388) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateRepeat` | combineCoord | `texture:vec4 (required; implicit first argument); repeatX:float=3; repeatY:float=3; offsetX:float=0.5; offsetY:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L419) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `repeatX` | coord | `reps:float=3; offset:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L450) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateRepeatX` | combineCoord | `texture:vec4 (required; implicit first argument); reps:float=3; offset:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L471) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `repeatY` | coord | `reps:float=3; offset:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L492) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateRepeatY` | combineCoord | `texture:vec4 (required; implicit first argument); reps:float=3; offset:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L513) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `kaleid` | coord | `nSides:float=4` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L534) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateKaleid` | combineCoord | `texture:vec4 (required; implicit first argument); nSides:float=4` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L554) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `scroll` | coord | `scrollX:float=0.5; scrollY:float=0.5; speedX:float=0; speedY:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L573) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `scrollX` | coord | `scrollX:float=0.5; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L604) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateScrollX` | combineCoord | `texture:vec4 (required; implicit first argument); scrollX:float=0.5; speed:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L623) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `scrollY` | coord | `scrollY:float=0.5; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L642) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `modulateScrollY` | combineCoord | `texture:vec4 (required; implicit first argument); scrollY:float=0.5; speed:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L661) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `add` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Weighted addition of second RGBA. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L680) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `sub` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Weighted subtraction of second RGBA. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L693) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `layer` | combine | `texture:vec4 (required; implicit first argument)` | Second alpha mixes RGB; alpha sum clamped to [0,1]. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L706) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `blend` | combine | `texture:vec4 (required; implicit first argument); amount:float=0.5` | Linear interpolation of both RGBA values. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L715) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `mult` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Interpolates original and channel-wise RGBA product. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L728) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `diff` | combine | `texture:vec4 (required; implicit first argument)` | Absolute RGB difference; maximum alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L741) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `modulate` | combineCoord | `texture:vec4 (required; implicit first argument); amount:float=0.1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L750) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `modulateScale` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=1; offset:float=1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L764) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `modulatePixelate` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=10; offset:float=3` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L785) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `modulateRotate` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=1; offset:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L804) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `modulateHue` | combineCoord | `texture:vec4 (required; implicit first argument); amount:float=1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L826) | A / SUPPORTED: native declared literal arguments; [evidence](MODULATION.md) |
| `invert` | color | `amount:float=1` | Interpolated RGB inversion; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L839) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `contrast` | color | `amount:float=1.6` | RGB centered contrast; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L852) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `brightness` | color | `amount:float=0.4` | RGB offset; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L866) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `mask` | combine | `texture:vec4 (required; implicit first argument)` | Second luminance multiplies first RGB and alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L879) | A / SUPPORTED: native declared literal arguments; [evidence](COMBINE.md) |
| `luma` | color | `threshold:float=0.5; tolerance:float=0.1` | Luminance key scales RGB and replaces alpha with key. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L890) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `thresh` | color | `threshold:float=0.5; tolerance:float=0.04` | Luminance threshold produces grayscale; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L909) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `color` | color | `r:float=1; g:float=1; b:float=1; a:float=1` | Signed per-channel multiplier, negative values invert then scale; affects alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L927) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `saturate` | color | `amount:float=2` | RGB luminance interpolation; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L959) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `hue` | color | `hue:float=0.4` | HSV hue rotation; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L974) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `colorama` | color | `amount:float=0.005` | HSV channel offset, converts and wraps RGB; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L990) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `prev` | src | `none` | Samples `prevBuffer` at wrapped UV; the pinned upstream binding can alias the draw attachment. Native safe previous-output semantics are defined in ARCHITECTURE.md. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1007) | A / SUPPORTED: native paired histories; temporal scope documented; [evidence](TEMPORAL_SEMANTICS.md) |
| `sum` | color | `scale:vec4=[1, 1, 1, 1]` | Weighted channel sum replicated into RGB; preserves alpha. Also used by scalar input conversion. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1016) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `r` | color | `scale:float=1; offset:float=0` | Red channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1034) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `g` | color | `scale:float=1; offset:float=0` | Green channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1052) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `b` | color | `scale:float=1; offset:float=0` | Blue channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1070) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |
| `a` | color | `scale:float=1; offset:float=0` | Alpha channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1088) | A / SUPPORTED: native declared literal arguments; [evidence](FAMILIES.md) |

## Expression evaluation and runtime contracts

- [`generate-glsl.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/generate-glsl.js): coordinate operators evaluate before their receiving chain; color operators evaluate after it. Later chained coordinate operators therefore wrap earlier coordinates. Nested inputs receive their own copy of the current UV. `combine` samples colors; `combineCoord` changes sampling coordinates.
- [`format-arguments.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/format-arguments.js): functions and arrays become per-frame uniforms; scalar constants are inlined. Texture arguments require `getTexture`; a texture used as a vec4 expression is wrapped with `src`. Scalar conversion from a color expression uses `sum`. Non-number callback results and callback exceptions fall back to declared defaults.
- [`generator-factory.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/generator-factory.js): source methods create expressions; chained methods mutate the expression transform list. Custom functions use the five supported categories. The array form of constructor `extendTransforms` calls `concat` without assignment in this revision; do not assume it successfully registers entries.
- [`output.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/output.js) and [`hydra-synth.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js): output-local framebuffer index toggles during drawing. `getTexture` exposes the framebuffer opposite the current index, while presentation uses `getCurrent`. Outputs tick sequentially in index order. Step 03 traced the locked regl callback order: self/earlier-output reads select n-1, later-output reads n-2 for continuously drawn outputs. See [the architecture](ARCHITECTURE.md#temporal-contract-match-observable-safe-output-reads); browser/GPU verification is still required.
- These operations do not automatically clamp intermediate RGBA values. Texture format, sampler filtering/wrapping, float math and final framebuffer conversion are part of the compatibility target.

## Runtime and service APIs

| API | Tier | Native equivalence / reference test sketch | Status |
| --- | --- | --- | --- |
| `s0, s1, s2, s3` | A | Fixed four typed native slots, image/video/capture replacement and explicit clearing. | SUPPORTED: [evidence](IMAGES.md) |
| `o0, o1, o2, o3` | A | Four independent output programs, revisions and paired histories. | SUPPORTED: [evidence](TEMPORAL_SEMANTICS.md) |
| `out(output=o0)` | A | expr.out(\oN,action), default o0; acknowledged atomic validated replacement. | SUPPORTED: [evidence](LIVE_CODING.md) |
| `render(output), render()` | A | h.render(\oN) selects one; h.render/nil selects column-ordered quadrants. | SUPPORTED: [evidence](OUTPUTS.md) |
| `hush()` | A | h.hush clears four output histories/selects o0 asynchronously. It does not clear media sources, stop SC producers or reset JS callbacks. Use clearSource/free and stop owned producers explicitly. | NATIVE ALTERNATIVE: [evidence](LIVE_CODING.md) |
| `setFunction(object)` | A | Typed h.setFunction/name/category/defaults/WGSL and expr.call. Arbitrary GLSL/JS objects are unsupported. | NATIVE ALTERNATIVE: [evidence](CUSTOM_FUNCTIONS.md) |
| `time=0, speed=1, bpm=30` | A | h.reset/h.clock; monotonic session seconds and bounded speed/BPM controls, not writable JS globals. | NATIVE ALTERNATIVE: [evidence](CLOCK.md) |
| `fps, stats.fps` | A | ShaderServer.status reports measured window presentation FPS; ShaderWindow.vsync controls presentation. Assigning a browser-style render throttle is unsupported. Export FPS is an independent sampling/encoding setting. | NATIVE ALTERNATIVE: [evidence](EXPORT.md) |
| `tick(dt), autoLoop` | A | Native event loop owns frames. No manual browser tick(dt), autoLoop=false stepping or offline deterministic-render API. | UNSUPPORTED: [evidence](CLOCK.md) |
| `update(dt), afterUpdate(dt)` | A | SC Routines/Patterns/bus mappings send scheduled controls; no synchronous per-GPU-frame JS or SC callback contract. | NATIVE ALTERNATIVE: [evidence](PATTERNS.md) |
| `setResolution(width,height), width, height` | B | h.setResolution sets physical render pixels; fixed resolution preserves window aspect. Sources retain their media dimensions; no canvas globals. | NATIVE ALTERNATIVE: [evidence](EXPORT.md) |
| `init({src,dynamic}), dynamic` | C | Arbitrary DOM element, ImageBitmap/canvas or external texture handles have no native import API. Existing managed source adapters are explicit alternatives. | ADAPTER REQUIRED: [evidence](COMPATIBILITY.md) |
| `initImage(url="", params)` | B | h.image(slot,localPath,action), bounded PNG/JPEG/PNM. URL fetching, browser image elements and regl params unsupported. | NATIVE ALTERNATIVE: [evidence](IMAGES.md) |
| `initVideo(url="", params)` | B | h.video(slot,localPath,decoder,loop,rate,action), explicit external FFmpeg. Browser URLs/elements/params unsupported. | NATIVE ALTERNATIVE: [evidence](VIDEO.md) |
| `initCam(index,params)` | B | h.initCam(slot,index,action), explicit macOS enumeration and native camera helper. Browser constraints/permission objects unsupported; other platforms unqualified. | NATIVE ALTERNATIVE: [evidence](CAPTURE.md) |
| `initScreen(index=0,params)` | B | h.initScreen uses an enumerated whole display on macOS. No browser/window-picker or constraints-object parity. | NATIVE ALTERNATIVE: [evidence](CAPTURE.md) |
| `initStream(streamName,params), pb` | C | Pinned source delegates to a caller-supplied pb event provider. No native patch-bay/WebRTC transport, discovery or signaling implementation is installed. | ADAPTER REQUIRED: [evidence](COMPATIBILITY.md) |
| `initCanvas(width=1000,height=1000)` | C | No browser CanvasRenderingContext2D/DOM object. A future texture bridge could import pixels, not recreate the returned Canvas API. | ADAPTER REQUIRED: [evidence](COMPATIBILITY.md) |
| `source.clear()` | B | h.clearSource(slot,action) releases the managed image/decoder; samples become transparent black. Browser MediaStream object/track lifetime is not exposed. | NATIVE ALTERNATIVE: [evidence](IMAGES.md) |
| `screencap(), getScreenImage(callback), canvasToImage(callback)` | B | h.screenshot writes a new PNG and exportStatus reports completion. No browser Blob, download element or image callback value. | NATIVE ALTERNATIVE: [evidence](EXPORT.md) |
| `vidRecorder.start(), vidRecorder.stop()` | B | Explicit external-FFmpeg H.264/MP4 recording, bounded live samples and drops. No browser MediaRecorder/WebM/25-FPS captureStream object. | NATIVE ALTERNATIVE: [evidence](EXPORT.md) |
| `a.fft, a.vol, a.bins, a.settings` | B | Existing ShaderAnalysis/ShaderFFTTexture scalar buses map native amplitude/onset/bin values. Not Meyda loudness bands or matching numerical normalization. | NATIVE ALTERNATIVE: [evidence](AUDIO.md) |
| `a.setBins(numBins=4)` | B | Select existing FFT binBus accessors and analysis configuration; no browser a.bins/settings recreation or equivalent band aggregation. | NATIVE ALTERNATIVE: [evidence](AUDIO.md) |
| `a.setCutoff, a.setSmooth, a.setScale` | B | ShaderBus smoothing and mapBus scale/bias/cutoff are explicit mappings with different units/response; not setters on a WebAudio analyzer. | NATIVE ALTERNATIVE: [evidence](AUDIO.md) |
| `a.setMax(max), a.max=15` | B | Deprecated browser loudness setting has no matching native a object or setter. | UNSUPPORTED: [evidence](AUDIO.md) |
| `a.beat, a.onBeat, a.detectBeat` | B | Native onset analysis and bus mapping/SC scheduling; no identical browser adaptive beat detector or callback. | NATIVE ALTERNATIVE: [evidence](AUDIO.md) |
| `a.show(), a.hide()` | C | No browser audio graph DOM overlay. The renderer diagnostic overlay is not a band-analysis monitor. | UNSUPPORTED: [evidence](COMPATIBILITY.md) |
| `a0…aN(scale=1,offset=0)` | B | Explicit FFT-bin bus plus mapBus scale/bias; generated global JS helpers are not installed. | NATIVE ALTERNATIVE: [evidence](AUDIO.md) |
| `mouse.x/y/buttons/mods/enabled` | B | ShaderWindow.inputEnabled and mouse/key actions deliver normalized window events. Map numeric parameters explicitly; no browser pointer/modifier object or direct Hydra mouse numeric source. | NATIVE ALTERNATIVE: [evidence](COMPATIBILITY.md) |
| `loadScript(url=""), eval(code), makeGlobal, canvas` | C | No JavaScript evaluator, DOM/canvas host, remote script loader or browser-global namespace. SC language evaluation is a different language/API. | UNSUPPORTED: [evidence](COMPATIBILITY.md) |
| `numSources=4, numOutputs=4` | A | Fixed four native inputs/outputs are supported. Arbitrary constructor counts and dynamic extra slots are unsupported. | SUPPORTED: [evidence](OUTPUTS.md) |
| `precision, extendTransforms, debug, detectAudio, enableStreamCapture` | A/B/C | See the individual constructor-option decisions in COMPATIBILITY.md: native f32, explicit WGSL registration, diagnostics and opt-in analysis/export replace only the stated functions. | NATIVE ALTERNATIVE: [evidence](COMPATIBILITY.md) |

Service evidence: [runtime](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js), [source handling](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-source.js), [audio](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/audio.js), [recording](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/video-recorder.js), [mouse](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/mouse.js). Runtime-internal `resize`, `tick`, `getTexture` and `getCurrent` implement the lifecycle/sampling contracts above; they are not additional user-facing shader operators.

## Arrays and dynamic parameters

| API | Tier | Semantics / sketch | Status |
| --- | --- | --- | --- |
| `array.fast(speed=1)` | A | h.seq(...).fast; finite bounded values, up to 64 entries and zero-speed fallback to one. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `array.smooth(smooth=1)` | A | h.seq(...).smooth in 0..1; modifiers are captured on commit, not global Array prototype methods. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `array.ease(ease="linear")` | A | h.seq(...).ease supports only linear/easeInOutQuad; custom functions and other named curves are rejected. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `array.offset(offset=0.5)` | A | h.seq(...).offset; native discrete negative indices wrap safely, unlike upstream undefined negative array items. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `array.fit(low=0,high=1)` | A | h.seq(...).fit mutates its handle and drops offset; native constant sequences are rejected. Upstream returns a new array. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `function(props)` | A | Native time/wave/sequence/parameter handles and SC producer updates replace selected numeric uses. Arbitrary JS/SC per-frame callbacks and their fallback semantics are unsupported. | NATIVE ALTERNATIVE: [numeric scope](PARAMETERS.md) |
| `ease(linear)` | A | Native sequence evaluator supports linear within its bounded numeric scope. | SUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInQuad)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeOutQuad)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInOutQuad)` | A | Native sequence evaluator supports easeInOutQuad within its bounded numeric scope. | SUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInCubic)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeOutCubic)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInOutCubic)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInQuart)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeOutQuart)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInOutQuart)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInQuint)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeOutQuint)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(easeInOutQuint)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |
| `ease(sin)` | A | No native evaluator for this named curve; an SC producer may calculate values but is not curve API parity. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |

Evidence: [array utilities](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/array-utils.js), [named curves](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/easing-functions.js). Array speed 0 falls back to 1 in upstream truthiness checks; non-smoothed negative indices use JS remainder while smoothing wraps positively. These edge cases need explicit compatibility decisions.

## Additional inspected source surfaces

These explicitly classify accessible source objects/helpers that were grouped or
implicit in the Step 02 inventory. Internal JS/reflection access is not a promise
that native GPU handles or a JavaScript host are available.

| API | Tier | Native scope / missing behavior | Status |
| --- | --- | --- | --- |
| `createSource(i)` | C | Native source count is fixed at four; no dynamic creation beyond s0…s3. | UNSUPPORTED: [slot contract](IMAGES.md) |
| `captureStream` | C | Native recording produces files; no MediaStream/track object or outgoing live video bridge. | ADAPTER REQUIRED: [decision](COMPATIBILITY.md) |
| `expression.glsl(), expression.compile(transforms)` | C | No browser GLSL/render-pass-array return API. Native snapshots contain typed IR, not reusable JS/GLSL objects. | UNSUPPORTED: [compiler](COMPILER.md) |
| `expression.addTransform(object)` | A | Explicit typed chains/custom call and bounded snapshots; arbitrary JS transform records are not accepted. | NATIVE ALTERNATIVE: [custom functions](CUSTOM_FUNCTIONS.md) |
| `source.resize(), source.tick(), source.getTexture()` | C | Native source adapters own uploads, dimensions and lifetime; opaque browser/regl texture handles cannot be passed to Hydra. | NATIVE ALTERNATIVE: [source contract](IMAGES.md) |
| `output.render(), output.tick(), output.resize(), output.getTexture(), output.getCurrent()` | A | Managed out/src/setResolution use native histories and selection. Raw regl render passes and JS framebuffer handles are not exposed. | NATIVE ALTERNATIVE: [histories](TEMPORAL_SEMANTICS.md) |
| `audio.tick(), audio.draw()` | B/C | SC analysis owns sampling; no browser canvas drawing or synchronous analyzer tick API. | NATIVE ALTERNATIVE: [audio scope](AUDIO.md) |
| `synth, s, o, generator, regl, sandbox` | C | No matching browser internal/reflection objects; use SCHydra/SCHydraExpr session APIs. | UNSUPPORTED: [host boundary](COMPATIBILITY.md) |
| `function-valued easing` | C | No serialized JS/SC evaluator inside the renderer; sequence easing uses the two named native curves. | UNSUPPORTED: [numeric scope](PARAMETERS.md) |

## SuperCollider collisions and reuse

Step 02 inspected SC 3.14.1 selectors: Object already defines blend/repeat/shape;
AbstractFunction defines rotate/blend; Function defines sum/r. Current SCHydraExpr
inherits Object and deliberately provides its own scoped methods. SCHydra and
SCHydraNumber own their respective session/numeric APIs; no global Object/Array/
Function extension installs Hydra operators. The earlier statement that SCHydra
did not yet exist applied only to the initial audit baseline.

ShaderBus/ShaderAnalysis/FFT/waveform mappings are now reused through the explicit
Step 17 bridge. Existing window events and scheduled OSC provide native controls;
none creates browser JS globals, DOM handles or transparent network access.

## Verification boundary

The source inventory is pinned; status is audited through Steps 23–24. Native
proofs below are historical results with their own binary/hardware scope. No new
runtime, browser GPU or network adapter test is claimed by Step 24. Compatibility
labels must be read with the argument/service limits above and COMPATIBILITY.md. Representative sketches should cover solid RGBA, chained coordinate order, nested blend/modulation, four outputs, prev/self/cross-feedback, array timing, invalid updates, media replacement and resize/recovery.

Step 07 evidence for the six partial entries: [native compiler](COMPILER.md), including domain restrictions and exact offscreen pixel scope. Step 08 adds native o0 presentation; no entry claims full synthesis-language parity or manual visual verification.

Step 09: [procedural family evidence](FAMILIES.md) covers all 32 procedural entries (6 src, 10 coord, 16 color). `src` and `prev` were deferred from the broader 34-entry inventory scope at Step 09. Steps 12–13 add output sampling and native prev; full runtime/browser parity remains outside that acceptance claim.

Step 10 implements all seven combine operations; see [COMBINE.md](COMBINE.md).

Step 11 implements all eleven combineCoord operations; see [MODULATION.md](MODULATION.md).

Step 12 adds output-only src sampling and four targets; see [OUTPUTS.md](OUTPUTS.md).

Step 13 implements temporal output `src` and native `prev`; see [TEMPORAL_SEMANTICS.md](TEMPORAL_SEMANTICS.md). All 52 names are recognized within the documented native argument scope. Upstream aliases/coercions and full runtime parity remain unqualified; native media sampling was added in Steps 19–21.

Steps 19–21 add typed native source slots: [images](IMAGES.md), [local video](VIDEO.md) and [macOS camera/display capture](CAPTURE.md). Browser URL and cross-platform capture parity remain outside the implemented scope.
