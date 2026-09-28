# Typed shader controls

SCShader 0.0.12 adds reflected controls and per-frame renderer-side interpolation.
The existing `amount: float` control and `/uniform/f` messages remain compatible.
No DSP or GPU work is added to the audio-server thread.

## Shader ABI

Group 0 bindings 0–3 retain the existing built-ins, source texture, sampler, and
Float32 spectrum texture. Add an optional **flat uniform struct at binding 4**:

```wgsl
// @scshader bool enabled
struct Controls {
    gain: f32,
    count: i32,
    enabled: u32,
    color: vec4<f32>,
    transform: mat3x3<f32>,
};
@group(0) @binding(4) var<uniform> controls: Controls;
```

Read controls directly in either shader stage. Keep `amount` at binding 0; its
name is reserved. Custom fields start at zero, including matrices; set identity
matrices explicitly when needed. `amount` starts at 0.5 for each new instance.

The renderer parses WGSL or GLSL with the already bundled Naga frontend and uses
its member offsets and types. It does not infer layout from SuperCollider Event
ordering. Float matrices are column-major; `mat3` columns have 16-byte GPU strides
even though the OSC payload has only nine tightly packed Float32 components.
The supported GLSL equivalent is a `layout(set=0, binding=4, std140) uniform`
block. Naga's GLSL restrictions still apply; this does not broaden the language
compatibility promise. See the [Naga type representation](https://docs.rs/naga/30.0.0/naga/enum.TypeInner.html)
and [member offsets](https://docs.rs/naga/30.0.0/naga/struct.StructMember.html).

| Shader type | Reflected SC type | Value |
| --- | --- | --- |
| `f32` / `float` | `\float` | Finite Float32-range number |
| `i32` / `int` | `\int` | Signed 32-bit Integer |
| `u32` / `uint` | `\uint` | Integral number, 0–4294967295 |
| Annotated `u32` / `uint` | `\bool` | `true` or `false` |
| `vec2`, `vec3`, `vec4` of Float32 | `\vec2`, `\vec3`, `\vec4` | Exactly 2, 3, or 4 finite components |
| Square Float32 matrices | `\mat2`, `\mat3`, `\mat4` | Exactly 4, 9, or 16 flat column-major components |

WGSL uniform storage cannot contain a native `bool`. The checked source comment
`// @scshader bool enabled` assigns logical boolean semantics to a `u32` member;
test it as `controls.enabled != 0u`. Missing, duplicated, or wrongly typed
annotations fail compilation. Other types, arrays, nested structs, non-square
matrices, and integer vectors are not supported controls. The limit is 127 custom
members, 64 UTF-8 bytes per name, and 16 KiB of uniform storage per Shader.

SuperCollider Integers are signed 32-bit. Write large uint values as **Float
literals**, for example `4294967295.0`; writing `4294967295` would overflow in the
language before validation. Typed OSC blobs preserve every integer bit without
depending on `NetAddr.useDoubles` or global serialization settings.

## Language API

Wait for `shader.isRunning` before using automatically reflected custom names.
To supply initial custom values, declare their types in `ShaderDef`'s optional
Event, for example `(gain: \float, color: \vec4)`. The renderer's full reflection
is authoritative; declarations are not a way to override GPU layout or types.

```supercollider
(
// x is a running Shader with these reflected controls.
x.set(\gain, 0.8, \color, [0.2, 0.5, 1.0, 1.0]);
x.setn(\transform, [1, 0, 0, 0, 1, 0, 0, 0, 1]);
x.setAt(0.1, \count, 12); // ordinary relative OSC-bundle timing
x.glide(\color, [1.0, 0.3, 0.1, 1.0], 0.5, \smooth);
x.get(\color, { |value| value.postln });
)
```

`set`/`setAt` validate the entire supplied pair list before sending anything.
`setn` sets one vector or matrix; it does not flatten arbitrary nested arrays.
`value(name)` returns a copy of the last language-side target, not the renderer's
current interpolated value. `get`/`getn` query that current CPU-side renderer
value asynchronously, with request IDs protecting callbacks from reply reordering.
A missing reply calls the callback with `nil` after two seconds and records
`E_TIMEOUT`. These are not GPU texture readbacks and must not be polled at audio
rate. Freeing a Shader cancels outstanding callbacks.

`glide(name, value, duration, mode)` supports `\step`, `\linear`, and `\smooth`.
Step applies immediately; linear and cubic smoothstep run over `duration` seconds
from renderer receipt/application time. Zero duration applies immediately.
Retargeting starts at the current interpolated value, and a normal `set` cancels
the old glide. Floats, vectors, and matrices interpolate component-wise; matrix
interpolation is not a rotation-aware decomposition. Integer and boolean controls
support only step. `ShaderBus` remains a scalar language-side smoother; its
updates can target any reflected float and replace an active glide.

## Reload and errors

All live file-backed shader resources are watched four times per second,
including non-selected graph passes. A candidate's reflection and GPU pipeline
must both validate before replacement. Matching names and types preserve current
values and in-progress glides even if offsets move. New or type-changed controls
start at zero. Removed/type-changed language-side pending values are discarded
with `E_REFLECTION_CHANGED`; no old type is silently sent under the new schema.
An invalid edit preserves the last valid pipeline, controls, and reflection.

The complete original example is `renderer/shaders/typed_controls.wgsl` with
`examples/04_typed_controls.scd`. Tests cover serialization, layout padding,
malformed messages, extreme numeric values, interpolation, and live Metal reloads.
