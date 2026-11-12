# Pinned Hydra API inventory

Reference: `hydra-synth` commit **9d29a9f4fd8f9081b9759943f38db36f05b9a88f**, retrieved 2026-11-12. The reference checkout is ignored under `build/hydra-reference`. Only API metadata and independently written behavior notes are included here; no upstream shader implementation is incorporated.

This inventory describes the pinned source, including lesser-used exported operations. All Hydra entries are initially **NOT IMPLEMENTED**. Existing SCShader resources are reuse candidates, not evidence of Hydra parity. Tier A is shader/runtime synthesis, B is native media/input services, C is browser/network integration.

## Shader operations

52 exported definitions discovered in `src/glsl/glsl-functions.js`: src: 8, coord: 10, color: 16, combineCoord: 11, combine: 7.

Declared arguments below exclude the implicit receiving chain. The factory prepends a required second expression for `combine` and `combineCoord`. Defaults are source metadata, not inferred documentation defaults. Each source link identifies the authoritative math and channel expression for later parity verification.

| Operation | Category | Arguments/defaults | Behavior and reference | Tier / status |
| --- | --- | --- | --- | --- |
| `noise` | src | `scale:float=10; offset:float=0.1` | Simplex noise; opaque grayscale. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L78) | A / NOT IMPLEMENTED |
| `voronoi` | src | `scale:float=5; speed:float=0.3; blending:float=0.3` | Animated nearest-cell pattern; opaque RGB. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L96) | A / NOT IMPLEMENTED |
| `osc` | src | `frequency:float=60; sync:float=0.1; offset:float=0` | Sine RGB channels with phase offsets; opaque. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L144) | A / NOT IMPLEMENTED |
| `shape` | src | `sides:float=3; radius:float=0.3; smoothing:float=0.01` | Regular polygon distance and soft edge; opaque grayscale. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L171) | A / NOT IMPLEMENTED |
| `gradient` | src | `speed:float=0` | UV-derived color; opaque. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L199) | A / NOT IMPLEMENTED |
| `src` | src | `tex:sampler2D=NaN` | Samples external/output texture at wrapped UV; RGBA preserved. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L212) | A / NOT IMPLEMENTED |
| `solid` | src | `r:float=0; g:float=0; b:float=0; a:float=1` | Constant RGBA; alpha supplied. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L226) | A / NOT IMPLEMENTED |
| `rotate` | coord | `angle:float=10; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L254) | A / NOT IMPLEMENTED |
| `scale` | coord | `amount:float=1.5; xMult:float=1; yMult:float=1; offsetX:float=0.5; offsetY:float=0.5` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L276) | A / NOT IMPLEMENTED |
| `pixelate` | coord | `pixelX:float=20; pixelY:float=20` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L313) | A / NOT IMPLEMENTED |
| `posterize` | color | `bins:float=3; gamma:float=0.6` | Quantizes RGB with gamma; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L332) | A / NOT IMPLEMENTED |
| `shift` | color | `r:float=0.5; g:float=0; b:float=0; a:float=0` | Offsets and wraps channels, including alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L355) | A / NOT IMPLEMENTED |
| `repeat` | coord | `repeatX:float=3; repeatY:float=3; offsetX:float=0; offsetY:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L388) | A / NOT IMPLEMENTED |
| `modulateRepeat` | combineCoord | `texture:vec4 (required; implicit first argument); repeatX:float=3; repeatY:float=3; offsetX:float=0.5; offsetY:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L419) | A / NOT IMPLEMENTED |
| `repeatX` | coord | `reps:float=3; offset:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L450) | A / NOT IMPLEMENTED |
| `modulateRepeatX` | combineCoord | `texture:vec4 (required; implicit first argument); reps:float=3; offset:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L471) | A / NOT IMPLEMENTED |
| `repeatY` | coord | `reps:float=3; offset:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L492) | A / NOT IMPLEMENTED |
| `modulateRepeatY` | combineCoord | `texture:vec4 (required; implicit first argument); reps:float=3; offset:float=0.5` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L513) | A / NOT IMPLEMENTED |
| `kaleid` | coord | `nSides:float=4` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L534) | A / NOT IMPLEMENTED |
| `modulateKaleid` | combineCoord | `texture:vec4 (required; implicit first argument); nSides:float=4` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L554) | A / NOT IMPLEMENTED |
| `scroll` | coord | `scrollX:float=0.5; scrollY:float=0.5; speedX:float=0; speedY:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L573) | A / NOT IMPLEMENTED |
| `scrollX` | coord | `scrollX:float=0.5; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L604) | A / NOT IMPLEMENTED |
| `modulateScrollX` | combineCoord | `texture:vec4 (required; implicit first argument); scrollX:float=0.5; speed:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L623) | A / NOT IMPLEMENTED |
| `scrollY` | coord | `scrollY:float=0.5; speed:float=0` | Transforms sampling UV before evaluating the receiving expression; sampled alpha follows that expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L642) | A / NOT IMPLEMENTED |
| `modulateScrollY` | combineCoord | `texture:vec4 (required; implicit first argument); scrollY:float=0.5; speed:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L661) | A / NOT IMPLEMENTED |
| `add` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Weighted addition of second RGBA. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L680) | A / NOT IMPLEMENTED |
| `sub` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Weighted subtraction of second RGBA. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L693) | A / NOT IMPLEMENTED |
| `layer` | combine | `texture:vec4 (required; implicit first argument)` | Second alpha mixes RGB; alpha sum clamped to [0,1]. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L706) | A / NOT IMPLEMENTED |
| `blend` | combine | `texture:vec4 (required; implicit first argument); amount:float=0.5` | Linear interpolation of both RGBA values. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L715) | A / NOT IMPLEMENTED |
| `mult` | combine | `texture:vec4 (required; implicit first argument); amount:float=1` | Interpolates original and channel-wise RGBA product. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L728) | A / NOT IMPLEMENTED |
| `diff` | combine | `texture:vec4 (required; implicit first argument)` | Absolute RGB difference; maximum alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L741) | A / NOT IMPLEMENTED |
| `modulate` | combineCoord | `texture:vec4 (required; implicit first argument); amount:float=0.1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L750) | A / NOT IMPLEMENTED |
| `modulateScale` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=1; offset:float=1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L764) | A / NOT IMPLEMENTED |
| `modulatePixelate` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=10; offset:float=3` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L785) | A / NOT IMPLEMENTED |
| `modulateRotate` | combineCoord | `texture:vec4 (required; implicit first argument); multiple:float=1; offset:float=0` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L804) | A / NOT IMPLEMENTED |
| `modulateHue` | combineCoord | `texture:vec4 (required; implicit first argument); amount:float=1` | Samples nested expression at current UV, then uses its channels to transform the receiving expression UV; sampled alpha follows receiving expression. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L826) | A / NOT IMPLEMENTED |
| `invert` | color | `amount:float=1` | Interpolated RGB inversion; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L839) | A / NOT IMPLEMENTED |
| `contrast` | color | `amount:float=1.6` | RGB centered contrast; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L852) | A / NOT IMPLEMENTED |
| `brightness` | color | `amount:float=0.4` | RGB offset; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L866) | A / NOT IMPLEMENTED |
| `mask` | combine | `texture:vec4 (required; implicit first argument)` | Second luminance multiplies first RGB and alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L879) | A / NOT IMPLEMENTED |
| `luma` | color | `threshold:float=0.5; tolerance:float=0.1` | Luminance key scales RGB and replaces alpha with key. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L890) | A / NOT IMPLEMENTED |
| `thresh` | color | `threshold:float=0.5; tolerance:float=0.04` | Luminance threshold produces grayscale; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L909) | A / NOT IMPLEMENTED |
| `color` | color | `r:float=1; g:float=1; b:float=1; a:float=1` | Signed per-channel multiplier, negative values invert then scale; affects alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L927) | A / NOT IMPLEMENTED |
| `saturate` | color | `amount:float=2` | RGB luminance interpolation; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L959) | A / NOT IMPLEMENTED |
| `hue` | color | `hue:float=0.4` | HSV hue rotation; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L974) | A / NOT IMPLEMENTED |
| `colorama` | color | `amount:float=0.005` | HSV channel offset, converts and wraps RGB; preserves alpha. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L990) | A / NOT IMPLEMENTED |
| `prev` | src | `none` | Samples output-local prior framebuffer at wrapped UV; RGBA preserved. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1007) | A / NOT IMPLEMENTED |
| `sum` | color | `scale:vec4=[1, 1, 1, 1]` | Weighted channel sum replicated into RGB; preserves alpha. Also used by scalar input conversion. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1016) | A / NOT IMPLEMENTED |
| `r` | color | `scale:float=1; offset:float=0` | Red channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1034) | A / NOT IMPLEMENTED |
| `g` | color | `scale:float=1; offset:float=0` | Green channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1052) | A / NOT IMPLEMENTED |
| `b` | color | `scale:float=1; offset:float=0` | Blue channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1070) | A / NOT IMPLEMENTED |
| `a` | color | `scale:float=1; offset:float=0` | Alpha channel scale/offset replicated into all four RGBA channels. [Source](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/glsl/glsl-functions.js#L1088) | A / NOT IMPLEMENTED |

## Expression evaluation and runtime contracts

- [`generate-glsl.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/generate-glsl.js): coordinate operators evaluate before their receiving chain; color operators evaluate after it. Later chained coordinate operators therefore wrap earlier coordinates. Nested inputs receive their own copy of the current UV. `combine` samples colors; `combineCoord` changes sampling coordinates.
- [`format-arguments.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/format-arguments.js): functions and arrays become per-frame uniforms; scalar constants are inlined. Texture arguments require `getTexture`; a texture used as a vec4 expression is wrapped with `src`. Scalar conversion from a color expression uses `sum`. Non-number callback results and callback exceptions fall back to declared defaults.
- [`generator-factory.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/generator-factory.js): source methods create expressions; chained methods mutate the expression transform list. Custom functions use the five supported categories. The array form of constructor `extendTransforms` calls `concat` without assignment in this revision; do not assume it successfully registers entries.
- [`output.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/output.js) and [`hydra-synth.js`](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js): output-local framebuffer index toggles during drawing. `getTexture` exposes the framebuffer opposite the current index, while presentation uses `getCurrent`. Outputs tick sequentially in index order. Resolve precise cross-output visibility against this implementation before choosing native frame semantics; do not replace it with an assumed simultaneous snapshot.
- These operations do not automatically clamp intermediate RGBA values. Texture format, sampler filtering/wrapping, float math and final framebuffer conversion are part of the compatibility target.

## Runtime and service APIs

| API | Tier | Native equivalence / reference test sketch | Status |
| --- | --- | --- | --- |
| `s0, s1, s2, s3` | A | Four default source slots; replace and clear one without affecting others. | NOT IMPLEMENTED |
| `o0, o1, o2, o3` | A | Four default independent outputs; overwrite one and inspect others. | NOT IMPLEMENTED |
| `out(output=o0)` | A | Assign compiled expression to output; invalid candidate must preserve last valid display. | NOT IMPLEMENTED |
| `render(output), render()` | A | Select output, or show all four quadrants; assert layout and aspect. | NOT IMPLEMENTED |
| `hush()` | A | Clear sources, assign transparent black to all outputs, select o0 and reset update callbacks. | NOT IMPLEMENTED |
| `setFunction(object)` | A | Register a category/name/input/GLSL definition; native custom WGSL policy must be explicit. | NOT IMPLEMENTED |
| `time=0, speed=1, bpm=30` | A | Time increments in seconds from tick dt in milliseconds times speed; array phase uses BPM. | NOT IMPLEMENTED |
| `fps, stats.fps` | A | Optional frame throttle and measured frame frequency; do not confuse with audio clock. | NOT IMPLEMENTED |
| `tick(dt), autoLoop` | A | Explicit tick or automatic browser loop; native renderer must own frame advancement. | NOT IMPLEMENTED |
| `update(dt), afterUpdate(dt)` | A | Callbacks before sources/outputs and after presentation; accumulated dt in milliseconds. | NOT IMPLEMENTED |
| `setResolution(width,height), width, height` | B | Resize canvas, output histories and sources; verify resized sampling resolution. | NOT IMPLEMENTED |
| `init({src,dynamic}), dynamic` | C | DOM-backed source handles require an adapter; native source API may provide alternatives. | NOT IMPLEMENTED |
| `initImage(url="", params)` | B | Asynchronous still image slot; native file/URI policy and errors must be defined. | NOT IMPLEMENTED |
| `initVideo(url="", params)` | B | Autoplay, loop, muted dynamic video; native decoder must have bounded buffering. | NOT IMPLEMENTED |
| `initCam(index,params)` | B | Permission-controlled camera source; handle absent device. | NOT IMPLEMENTED |
| `initScreen(index=0,params)` | B | Permission-controlled desktop capture; upstream browser helper ignores index. | NOT IMPLEMENTED |
| `initStream(streamName,params), pb` | C | Patch-bay video event transport; adapter required, outside core. | NOT IMPLEMENTED |
| `initCanvas(width=1000,height=1000)` | C | Returns browser 2D context; native texture bridge does not reproduce Canvas API. | NOT IMPLEMENTED |
| `source.clear()` | B | Stops source tracks and resets to a 1x1 texture; native resources must be released. | NOT IMPLEMENTED |
| `screencap(), getScreenImage(callback), canvasToImage(callback)` | B | One requested frame image; native file/result API must be explicit. | NOT IMPLEMENTED |
| `vidRecorder.start(), vidRecorder.stop()` | B | Optional 25 fps canvas stream and WebM browser recording; native format/encoder declared. | NOT IMPLEMENTED |
| `a.fft, a.vol, a.bins, a.settings` | B | Browser Meyda loudness-derived bands, not a conventional raw FFT; reuse SC analysis with documented numerical differences. | NOT IMPLEMENTED |
| `a.setBins(numBins=4)` | B | Recreate bands/settings; test changed band count and bounds. | NOT IMPLEMENTED |
| `a.setCutoff, a.setSmooth, a.setScale` | B | Defaults cutoff=2, smooth=0.4, scale=10; update per-band settings. | NOT IMPLEMENTED |
| `a.setMax(max), a.max=15` | B | Deprecated setter; do not add a loudness subsystem to mimic this. | NOT IMPLEMENTED |
| `a.beat, a.onBeat, a.detectBeat` | B | Adaptive threshold callback; SC native analysis alternative must disclose differences. | NOT IMPLEMENTED |
| `a.show(), a.hide()` | C | Browser audio graph overlay; optional native monitoring, no DOM parity. | NOT IMPLEMENTED |
| `a0…aN(scale=1,offset=0)` | B | Generated browser audio-band callback helpers; native wrappers can use existing buses. | NOT IMPLEMENTED |
| `mouse.x/y/buttons/mods/enabled` | B | Pointer/button/modifier state; map renderer coordinates explicitly. | NOT IMPLEMENTED |
| `loadScript(url=""), eval(code), makeGlobal, canvas` | C | JavaScript/DOM execution and global binding have no direct native equivalence. | NOT IMPLEMENTED |
| `numSources=4, numOutputs=4` | A | Constructor permits counts; core native contract is four unless explicitly extended. | NOT IMPLEMENTED |
| `precision, extendTransforms, debug, detectAudio, enableStreamCapture` | A/B/C | Constructor options: native precision/capability defaults must be documented; browser options need explicit alternatives. | NOT IMPLEMENTED |

Service evidence: [runtime](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js), [source handling](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-source.js), [audio](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/audio.js), [recording](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/video-recorder.js), [mouse](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/mouse.js). Runtime-internal `resize`, `tick`, `getTexture` and `getCurrent` implement the lifecycle/sampling contracts above; they are not additional user-facing shader operators.

## Arrays and dynamic parameters

| API | Tier | Semantics / sketch | Status |
| --- | --- | --- | --- |
| `array.fast(speed=1)` | A | Set sequence speed; compare phases at equal time/BPM. | NOT IMPLEMENTED |
| `array.smooth(smooth=1)` | A | Interpolate around step boundaries; test wrap and negative time. | NOT IMPLEMENTED |
| `array.ease(ease="linear")` | A | Named or function easing enables smoothing; unknown name leaves settings unchanged. | NOT IMPLEMENTED |
| `array.offset(offset=0.5)` | A | Set phase offset using JS remainder; test negative offset. | NOT IMPLEMENTED |
| `array.fit(low=0,high=1)` | A | Remap min/max into range; returns new array carrying speed/smooth/ease, but not offset. Constant array behavior needs explicit policy. | NOT IMPLEMENTED |
| `function(props)` | A | Per-frame numeric parameter from time, mouse, bpm, resolution; invalid callbacks fall back to defaults. | NOT IMPLEMENTED |
| `ease(linear)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInQuad)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeOutQuad)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInOutQuad)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInCubic)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeOutCubic)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInOutCubic)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInQuart)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeOutQuart)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInOutQuart)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInQuint)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeOutQuint)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(easeInOutQuint)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |
| `ease(sin)` | A | Compare named curve at 0, 0.25, 0.5, 0.75, 1 against pinned source. | NOT IMPLEMENTED |

Evidence: [array utilities](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/array-utils.js), [named curves](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/easing-functions.js). Array speed 0 falls back to 1 in upstream truthiness checks; non-smoothed negative indices use JS remainder while smoothing wraps positively. These edge cases need explicit compatibility decisions.

## SuperCollider collisions and reuse

Inspected installed SC 3.14.1 class-library methods: `Object` already defines `blend`, `repeat`, and `shape`; `AbstractFunction` defines `rotate` and `blend`; `Function` defines `sum` and `r`. Hydra expression objects should derive directly from `Object` and deliberately override conflicting selectors with documented semantics. Do not globally extend `Object`, `Array`, or `Function` for Hydra. Existing Shader classes must remain unchanged. No `SCHydra` class exists in the current project classes; a full installed-extension namespace check is still required before class installation.

Existing `ShaderBus`, `ShaderAnalysis`, texture classes, input events and timestamped OSC are reuse candidates with source contracts in [BASELINE.md](BASELINE.md). They are not marked REUSED until a Hydra adapter and verification exist.

## Verification boundary

This inventory was extracted from pinned source declarations and inspected runtime files; upstream/browser visual sketches and native parity tests were NOT RUN. It is an implementation checklist, not a compatibility claim. Representative sketches should cover solid RGBA, chained coordinate order, nested blend/modulation, four outputs, prev/self/cross-feedback, array timing, invalid updates, media replacement and resize/recovery.
