# Numeric sources — Step 15

The minimal numeric API supports dynamic scalar arguments in `osc`, `solid`,
`color`, `rotate` and `scale`. Other operations retain constant arguments and
explicitly reject dynamic arguments. This meets the frequency/color/transform
checkpoint without claiming complete Hydra argument compatibility.

```supercollider
h.osc(h.param(\frequency, 8))
    .color(h.wave(0.7, 0, 0.2, 1), 0.7, 1)
    .rotate(h.time(0.1))
    .scale(h.seq([1, 1.3, 1.05]).fast(0.5).smooth)
    .out(\o0);
// After the commit acknowledgement:
h.set(\frequency, 16, \o0, { |result| result.postln });
```

`param(name, initial)` returns the same session-local mutable handle on repeated
use of the name; its initial value applies on first creation. `set` updates one
already committed output containing that handle. It retains history, revision
and pipeline. The client sends one acknowledged update when requested, without
uploading a graph. Update each output explicitly if a handle is used in several
outputs. A successful set also changes the handle's initial value for future
snapshots. Freeing the controller cancels its tracked requests.

`time(rate=1, bias=0)` computes seconds × rate + bias. `wave(rate=1, phase=0,
low=0, high=1)` maps sin(seconds × rate + phase) to the supplied interval; rate
is radians per second. These small native expressions and sequences evaluate
at the common renderer frame time and write numeric uniforms. They require no
language callbacks, arbitrary JavaScript or serialized SC Function. More general
expression operators and automatic Function producers remain unsupported; SC
Routines/Patterns may call set at their chosen control rate.

Sequences contain 1..64 finite scalars. `fast` changes rate; zero follows the
pinned upstream fallback to one. `offset` uses remainder modulo one. `smooth`
accepts 0..1, with zero selecting discrete steps. Positive smoothing centers the
interpolation interval about each step. `ease(\linear)` enables full smoothing.
`ease(\easeInOutQuad)` matches the pinned piecewise quadratic easing; other easing names report an
error. `fit(low,high)` maps the range and drops offset as the pinned upstream does;
constant sequences are rejected because upstream range division is undefined.
Sequences currently use the initial 30 BPM (time × speed × 0.5). Step 16 adds clock
controls. Native discrete negative phases wrap safely; upstream's JavaScript
remainder may produce an undefined array item. Smoothed phases wrap in both.

Construct numeric sources and sequence modifiers before committing. Subsequent
modifier changes require another snapshot; mutable set is the uniform-only path.
Each session has at most 256 handles; each declaration has at most 68 scalar
payload entries. Values must be finite and bounded in magnitude by 1e6. Dynamic
frequency and scale values must avoid the same zero-divisor domains as the
reference; no epsilon is inserted. A literal invalid frequency remains rejected.

HYIR 1.1 adds numeric declarations before nodes: u32 ID (1..256), u8 kind,
u8 scalar count, then that many big-endian f32s. Kind 0 is mutable `[initial]`,
1 linear time `[rate,bias]`, 2 wave `[rate,phase,low,high]`, 3 sequence
`[speed,offset,smoothing,ease,values...]`; ease is 0 linear or 1 easeInOutQuad.
Scalar argument tag 1 names a declared u32 ID. HYIR 1.0 remains accepted with
zero declarations. IDs are uniform slots; source identity contains their wiring
but excludes current values and evaluator payloads. Matching structural source
can reuse its pipeline across different numeric initial values.

The optional group 0 binding 1 is a 1024-byte uniform block (64 vec4s), separate
for each output image. `parameter(epoch, session, sequence, output, revision,
parameterID, value:f32)` is an immediate-only request returning the ordinary ack.
Duplicate retries are idempotent; conflicting/older sequences and stale revisions
are rejected before mutation. Invalid handles/values preserve accepted state.
This is bounded stop-and-wait control traffic; timed parameter events are Step 16.
Numeric values are retained in accepted CPU programs for recovery; numeric
recovery injection has not been qualified in this step.

Checks cover numeric pixels with unchanged pipeline identity, sequence/reference
traces, malformed numeric sources, revision/sequence/failure guards, source SC
snapshot/commit/set and installed regression checks. Native numerical checks use
Apple M5/Metal; browser GPU and other platforms remain unqualified. See
VERIFICATION.md and the runnable `examples/hydra/06_parameters.scd`.
