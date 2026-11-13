# Spatial modulation — Step 11

All eleven pinned `combineCoord` operations are implemented: `modulate`,
`modulateRepeat`, `modulateRepeatX`, `modulateRepeatY`, `modulateKaleid`,
`modulateScrollX`, `modulateScrollY`, `modulateScale`, `modulatePixelate`,
`modulateRotate` and `modulateHue`. Registry names/defaults/types and explicit SC
methods match the pinned inventory. Discovery now advertises 50 shader operations;
`src`/`prev` still require later output/history/media work.

Each operation evaluates its modulator at the current UV, derives a new UV and
calls its receiver there. Ordinary coordinate operations after modulation wrap
both branches; operations inside the modulator affect only its sampling. Nested
modulation and shared expressions remain function calls within one fragment pass.
No intermediate texture or extra pass is allocated. Existing topological, node,
depth, expansion and generated-source limits remain enforced.

Displacement uses modulator RG; scale and pixelate derive their two axis extents
from RG plus a scalar offset. Rotate derives its angle from red, rotates about
0.5 and recenters. Kaleid adds red to the radius of its folded polar coordinate
without recentering the final coordinate. Scroll uses red plus time/speed and
wraps with floor-based fract. Repeat includes a parity step plus the modulator
channel offset; its second parity test observes the first coordinate change.
The pinned `modulateRepeatY` scales X, and uses red: this surprising behavior is
preserved. `modulateHue` offsets coordinates by G−R and B−G in pixel units using
the current output resolution, rather than changing hue in color space. Receiver
alpha follows the warped sample; modulator alpha does not blend into it.

Scalar inputs must be finite. Zero kaleid sides and identically zero scale/
pixelate extent formulas (multiple and offset both zero) fail compilation.
Dynamic extents may cross zero depending on modulator pixels; those upstream
undefined domains and extreme finite values are not qualified. No epsilon clamp
or alternate formula is silently substituted. The native numerical evidence
covers defined domains, not every argument or browser-runtime behavior.

The independently written WGSL follows the pinned mathematical behavior using
the [existing provenance procedure](FAMILIES.md#reference-and-provenance).
The external GLSL reference is read at test runtime; no upstream shader bodies
are distributed. The existing reference helper now emits coordinate/color
signatures correctly for combineCoord and exposes resolution from the same frame
uniform. Fifty 3x3 RGBA8 cases at time 0.37 pass on Apple M5/Metal with zero
maximum channel difference. The eleven new cases nest displacement inside the
receiving chain, use a transformed/composited modulator shared at different UVs,
and apply another rotation after modulation. Negative repeat/scroll parameters
exercise wrapping. Invalid edits retain the previous candidate and its pixels.
Rust checks cover all defaults, types, references and constant-divisor rejection;
SC checks cover all defaults, copy isolation, ownership and eleven live chains,
followed by rejection and successful revision 12. See [verification](../VERIFICATION.md).

`examples/hydra/04_modulation.scd` supplies two chained demos on a separate port.
Recompile the SC class library after installation, then evaluate startup and the
replacement separately. Step 10's example is visually confirmed by the user;
the user also confirms the modulation example works perfectly. Step 12 handles
multiple outputs and sampling. Other platforms and full browser parity remain
unqualified.
