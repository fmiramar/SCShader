# Nested composition — Step 10

Implemented: `add(other, amount=1)`, `sub(other, amount=1)`,
`blend(other, amount=0.5)`, `mult(other, amount=1)`, `layer(other)`,
`diff(other)` and `mask(other)`. All seven pinned inventory entries have SC
methods, typed registry metadata, HYIR expression references and native WGSL.
Discovery advertises the 32 procedural plus seven combine names.

Pure expressions execute in one fragment pass. Both branches receive the current
sampling coordinate; coordinate operations after a combine transform both branches,
while operations inside one branch affect that branch only. No intermediate
textures are allocated for nesting. Reused nodes remain function calls rather
than separate passes. Both decoder and compiler enforce topological dependencies,
64 levels and 4096 expanded operations; DAG sharing does not bypass these bounds.
SC mutable child references and deep `.copy` semantics remain unchanged.

Add/sub interpolate between the original color and the full sum/difference;
blend interpolates both RGBA vectors. Mult interpolates between the original and
component product. Diff takes absolute RGB difference and the larger alpha.
Layer mixes RGB using the second alpha, then clamps the sum of alphas. Mask uses
the second RGB luminance (weights 0.2125, 0.7154, 0.0721), ignoring second alpha,
to scale first RGB and alpha. Only layer's alpha is explicitly clamped inside
these formulas; the RGBA8 render target clamps final output. Finite amounts are
not restricted to 0..1. No CSS compositing semantics are implied.

The formulas follow the pinned inventory source. Production WGSL is independently
written; external reference GLSL is read only at test runtime using the existing
[provenance procedure](FAMILIES.md#reference-and-provenance). Reference checks
cover all seven operations with a nested blend in the second branch, fractional
alpha, branch rotation and a rotation after composition. Add/sub/mult use amount
0.35; blend uses its default 0.5. Native Apple M5/Metal 3x3 comparisons at time
0.37 pass with zero channel difference. Rejected nested shape edits preserve the
candidate and its pixels. Existing 32 procedural reference cases still pass.
These are formula/transform checks through Naga, not browser runtime parity.

SC checks exercise defaults, nested ownership/types, deep copy isolation, all
seven live commits, the required oscillator/rotated-polygon blend and a failing
nested scale edit followed by a successful revision 8. A rejected edit leaves
the accepted revision unchanged. Rust tests also reject bad dependencies/types
and over-expanded shared branches. See [verification](../VERIFICATION.md).

`examples/hydra/03_combine.scd` uses its own port and names. Recompile the SC class
library after installation. Evaluate startup, then the replacement separately.
Manual visual observation of this new example and other backends remain open.
Spatial modulation is Step 11. Output sampling/history and media remain deferred.
