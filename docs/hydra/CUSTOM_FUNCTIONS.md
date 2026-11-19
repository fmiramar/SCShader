# Custom WGSL functions — Step 22

Typed `setFunction`-style operations are **SUPPORTED** using WGSL function bodies.
Arbitrary GLSL/JavaScript snippets from browser Hydra are **UNSUPPORTED**. This
step adds no GLSL subset parser or transpiler; adapt the body explicitly to WGSL.
The existing legacy shader loader's GLSL support is a separate API.

Registrations belong to one SCHydra/native session. They do not add methods,
classes or selectors to SuperCollider's global namespace. Start a source with
`h.call(name, arguments)`; append another category with `expression.call(name,
arguments)`. Arguments are positional arrays; omitted trailing values use the
registered defaults. Wait for each structural request's callback.

```supercollider
// Inside an already ready h callback:
h.setFunction(\tintPulse, \color,
    [[\amount, \float, 0.65], [\tint, \vec4, [0.2, 1, 0.7, 1]]],
    "let pulse=0.55+0.45*sin(time); return vec4<f32>(mix(color.rgb,color.rgb*tint.rgb,amount*pulse),color.a);",
    { |result|
        if(result[\ok]) {
            h.osc(8, 0.1, 0.4)
                .blend(h.shape(4).call(\tintPulse), 0.5)
                .call(\tintPulse, [h.param(\strength, 0.65)])
                .out(\o0, { |committed| committed.postln });
        } { result.postln };
    });
```

The complete no-audio startup, ordered tests and repeatable cleanup are in
[10_custom_functions.scd](../../examples/hydra/10_custom_functions.scd).
Recompile SC classes after installation. Block 3 deliberately submits invalid
code: expect `E_HYDRA_FUNCTION` and continued animation. Block 4 removes the
registration while retaining the accepted visual. Block 6 registers again and
builds a new nested composition.

## Signatures and evaluation

Every body receives `uv: vec2<f32>` and `time: f32` (Hydra session seconds).
Explicit inputs support `\float` / `f32` and `\vec4` / `vec4<f32>` with finite
defaults. Float arguments also accept the existing native numeric sources and
mutable parameters; vec4 arguments are literal four-element arrays.

| Category | Call form | Additional implicit values | Return and behavior |
| --- | --- | --- | --- |
| `\src` | `h.call(name, args)` | none | `vec4<f32>` color at current UV |
| `\coord` | `expr.call(name, args)` | none | `vec2<f32>`; sample receiver at returned UV |
| `\color` | `expr.call(name, args)` | `color: vec4<f32>` receiver sample | `vec4<f32>` color |
| `\combine` | `expr.call(name, [otherExpr] ++ args)` | `color`, `other: vec4<f32>` | `vec4<f32>` color; both samples use current UV |
| `\combineCoord` | `expr.call(name, [otherExpr] ++ args)` | `other: vec4<f32>` | `vec2<f32>`; sample receiver at returned UV |

Coordinate results use the same nested evaluation and wrapping rules as the
existing compiler. A body does not access textures or other registered functions
directly; compose operations through expression nodes. The first explicit call
argument for combine categories is a required expression, without a default.

## Validation, replacement and ownership

Registration validates the bounded schema, parses and validates WGSL using the
pinned Naga dependency, then validates a temporary GPU pipeline on the active
device before publishing the definition. Return type/signature errors, recursion,
extra functions and module/resource injection fail. Failed registration retains
the prior definition and running output. Ordinary output commits retain their
existing atomic validation/replacement contract. WGSL math can produce nonfinite
results; pure typed code is not a promise of visually meaningful output.

`h.removeFunction(name, action)` removes the callable registration. Accepted
programs contain frozen copies of their definitions: removal or redefinition
does not change their current animation or numeric controls. Resize recompiles
the frozen program. Existing recovery uses the same retained program, with the
normal history reset; custom-function recovery was not separately exercised.
Removal of an absent name acknowledges without changing anything.

Each successful registration has a generation-bound wire name. Expression handles
using an earlier generation cannot create new snapshots after removal/redefinition;
they report a stale/unknown operation instead of silently adopting new code.
Rebuild the composition to use the new registration. Releasing/closing the session
releases its registry, programs and resources. No function persists across boot.

Names start with an ASCII letter and contain only letters, digits or underscore.
Function names are at most 24 bytes; input names at most 16. Builtin operation
names, API-reserved names and `u_`/`h_` prefixes cannot be used for functions;
inputs must be unique and cannot use `uv`, `time`, `color` or `other`. WGSL keywords
in a signature fail compiler validation. Generated names remain strings in SC.

Limits: 16 active registrations per session; eight explicit inputs per function;
1–2048 ASCII body bytes; 3072-byte definition blob. Bodies have at most 512 Naga
expressions, 32 local variables, 128 statements and 4096 bytes of local storage.
Local arrays contain at most 64 scalar/vector elements, without nested arrays.
Loops, arbitrary function calls/dependencies, discard and synchronization are
unsupported. Scalar/vector constructors, pure builtin math, local assignment,
conditionals and returning branches are supported within these bounds.
The existing IR node/depth/expansion limits and 256 KiB generated-WGSL cap still
apply. These constraints bound this extension; they are not an isolation guarantee
for arbitrary untrusted shaders or a GPU execution-time budget.

## Evidence and remaining scope

Native Apple M5/Metal checks cover all five categories, typed/default/numeric
inputs, nested SC composition, failed registration and output replacement,
removal/stale generations/session isolation, frozen resize and resource disposal.
The pixel checker verifies that a rejected edit retains the prior image. The
installed example is exercised through its actual startup block.
See [VERIFICATION.md](../VERIFICATION.md) for exact binaries/results and preserved
failures. Physical IDE appearance awaits the user's observation. Windows, Linux,
Intel Mac and browser/GLSL source compatibility are not qualified by this result.
