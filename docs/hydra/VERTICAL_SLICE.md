# Hydra first visible slice

Status: **specification only**. No class, protocol command, shader, or example below
exists yet. Source baseline: SCShader `d9eb75b`; Hydra behavior reference and
ownership/transport decisions are pinned in [ADR 001](ARCHITECTURE.md).

## User result

Boot the existing `ShaderServer`, open one `SCHydra` session, submit
`osc(10).rotate(0.2)` to `o0`, wait for the commit acknowledgement, and select
`o0` in the renderer's existing window. At default offset zero, all RGB channels
are equal: expect moving **grayscale** diagonal bands with opaque alpha. Rotation
changes the sampling coordinates; no additional window or audio server is needed.

Target interaction, illustrative until the classes are implemented:

```supercollider
(
v = ShaderServer.new(\hydraSlice);
v.waitForBoot { |server|
    h = SCHydra.new(server);
    h.waitForReady {
        h.osc(10).rotate(0.2).out(\o0, { |result|
            if(result[\ok]) { h.render(\o0) } { result.postln };
        });
    };
};
)

// Cleanup in a later evaluation:
// h.free; v.quit;
```

The proposed `out(output, action)` callback receives a bounded result with `ok`,
transaction ID, output, committed revision, and error code/message on failure.
It fires once after a committed reply or an explicitly reported timeout/error.
The actual implementation must reconcile a lost commit reply with `status` before
declaring failure; an unresolved timeout reports **unknown outcome**, not a false
rollback claim. `waitForReady` must expose a failure/timeout callback and must not
silently queue unbounded work. Use the project's existing class/help conventions
when finalizing method arguments. The example in Step 08 must include explicit
startup error handling and cleanup and run from an unsaved SC IDE document.

ADR 001 chooses mutable expression builders with immutable `.out` snapshots to
match the pinned Hydra chain behavior. Step 05 calls the expression class immutable;
implement the ADR's snapshot invariant and document this deliberate resolution.
No mutation after `.out` may change a submitted or active snapshot.

## Exact slice and boundaries

- Initial end-to-end demonstration: `osc`, `rotate`, one `o0`, and presentation.
  Step 07 additionally specifies `solid`, `gradient`, `scale`, and `color` for its
  first code-generation family; advertise each only when actually implemented.
- The SC front end in Step 05 must also construct and serialize a nested
  `osc(...).blend(noise(...), ...)` graph for offline inspection, as that step
  requires. This does **not** imply renderer support for `blend` or `noise` before
  later compiler steps. Unsupported renderer operations return a capability error.
- `osc(10)` expands to frequency `10`, sync `0.1`, offset `0`; `rotate(0.2)`
  expands to angle `0.2`, speed `0`. Angles are radians; time is seconds. The IR
  example and bounds are in [ADR 001](ARCHITECTURE.md#ir-and-shader-abi).
- Handshake follows ordinary server readiness, followed by `capabilities`, `open`,
  upload `begin`/`chunk`/`commit`, a committed revision, then `present`. Only
  `o0` is advertised in the initial renderer slice; other outputs, feedback,
  dynamic parameters, media and custom functions remain unadvertised until their
  respective implementations and checks exist. The older renderer's error or
  timeout reports Hydra unavailable without changing Legacy presentation.
- Commit validates a complete bounded IR and the generated WGSL/pipeline before
  installing it at a frame boundary. A malformed input, unsupported operation,
  stale expected revision, or failed compile reports a structured error and leaves
  the accepted `o0` program, revision and picture intact. With no previous Hydra
  program, failure retains the current Legacy presentation. A pending or failed
  candidate must not automatically select Hydra presentation.
- `render(\legacy)` returns to the existing Shader/ShaderGraph presentation.
  `h.free` releases the Hydra session and selects Legacy; `v.quit` stops the
  renderer. Resize follows the bounded history policy in the ADR.

## Reference math for the checks

At fixed `time = 0`, the pinned `osc(10, 0.1, 0)` produces each RGB channel as
`0.5 + 0.5 * sin(10 * u)` with alpha 1. For normalized bottom-left UV `(x,y)`,
rotation at `0.2` radians gives `u = 0.5 + cos(0.2)*(x-0.5) + sin(0.2)*(y-0.5)`.
This follows the pinned GLSL matrix's column-major multiplication; coordinate
orientation must be explicit at the WGSL boundary. Before output quantization:

| UV | Expected R = G = B | Alpha |
| --- | --- | --- |
| (0.5, 0.5) | 0.020538 | 1 |
| (0.75, 0.5) | 0.959784 | 1 |
| (0.5, 0.75) | 0.146053 | 1 |

The two off-center samples distinguish transform direction and help catch a
flipped Y axis. GPU readback must select actual pixel centers and compute expected
values for those exact UVs; allow at most two 8-bit channel values of quantization
and floating-point tolerance. These equations are reference targets, not a claim
that the browser or native renderer has already passed them.

## Three focused checks

1. **SC / IR, no renderer.** Compile the new classes with the existing SC harness;
   construct and inspect the `osc.rotate` snapshot and a nested
   `osc.blend(noise)` snapshot. Assert ordered nodes/defaults and network-order
   bytes, deep snapshot isolation, cross-session rejection, invalid output/type
   rejection, and preservation of the existing `\shader` Event. No OSC boot needed.
2. **Rust parser and WGSL, no GPU.** Decode the same fixture, reject a truncated
   chunk/IR, wrong arity/type, nonfinite value and stale revision; generate WGSL
   and validate it with Naga. Check the fixed-time math at the UVs above using a
   small independent reference calculation and verify `rotate` evaluates before
   the oscillator. Report this as IR/compiler verification, not rendered pixels.
3. **Short native integration, GPU available.** Boot and negotiate, submit the
   expression, wait for its committed revision and select `o0`; read three pixels
   at a controlled time using the existing test-only GPU readback path adapted to
   the Hydra output. Compare against the pixel-center calculation, then submit a
   malformed candidate and a stale revision, confirm structured errors and that
   the revision and last valid output remain active. Select Legacy and run one
   existing Shader example; resize once, then free/quit normally. Also observe
   the moving bands in the window and record backend, device and binary/source
   hashes. Pixel proof, user-visible observation and legacy regression are
   separate results.

If no GPU/window is available, checks 1–2 are the fallback and check 3 is
**NOT RUN**. A generated source string, Naga validation, or software adapter is
not proof of native rendering or visual correctness. Keep all runs short and use
the existing harness; no soak or benchmark is part of this slice.

## Completion gate

Steps 05–08 complete the slice only when the supported classes compile, the
bounded capability handshake and accepted commit are observed, a native GPU pass
and visible band observation are recorded, invalid/stale candidates preserve the
last valid picture, and the existing Shader example still runs. Record explicit
PASS/FAIL/BLOCKED/NOT RUN outcomes and the exact binary/source for each check.
