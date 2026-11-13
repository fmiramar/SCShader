# Hydra temporal semantics — Step 13

This replaces Step 12's provisional same-frame dependency sorting with fixed-order
history sampling. Capability `outputs-temporal-v1` identifies this behavior;
`outputs-static-v1` is no longer advertised. ABI/IR remains 1.0.

## Reference and decision

The pinned [Hydra output implementation](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/output.js)
changes the output's framebuffer index in the framebuffer callback. Its
`getTexture()` returns the opposite image; `getCurrent()` returns the current
image. [Argument conversion](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/format-arguments.js)
uses `getTexture()` for output sampling. The locked **regl 1.7.0**
[command compiler](https://github.com/regl-project/regl/blob/v1.7.0/lib/core.js)
evaluates the framebuffer before texture uniforms for individual draws. Hydra's
[frame loop](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js)
draws outputs in numeric order and presents `getCurrent()` afterward.

`tools/check_hydra_temporal_reference.py` reads those exact Git objects and
integrity-checks the regl package against Hydra's lockfile. It executes upstream
Output callbacks with a small recorder in macOS JavaScriptCore, using the
source-verified callback order. Changing frame tags and a cycle confirm the
Step 03 derivation. This is **not execution of regl/WebGL** and does not establish
browser GPU parity. No upstream shader bodies are included in production code.

For continuously assigned outputs, when computing Hydra frame n:

| Read by output i | Image read |
| --- | --- |
| `src(oi)` or native `prev()` | i at n−1 |
| `src(oj)`, j < i | j at n−1 |
| `src(oj)`, j > i | j at n−2 |
| Unassigned output | transparent black |
| Presentation | newest image, n |

Startup, replacement and reset history is transparent black, including both
initial images. Cycles are legal: they refer to retained images and require no
topological sorting. There are no same-frame output edges. Nested expressions
still evaluate within the current pass at its current UV. Frame numbers in the
table describe uninterrupted history; resets start a new history interval.

## GPU ownership and ordering

Each assigned output owns two RGBA8Unorm images. All assigned outputs advance
once per acquired, selected Hydra frame in order o0, o1, o2, o3. Unassigned outputs
share a zero-initialized 1×1 image. A common bank index is sufficient because all
assigned outputs advance together; new pairs are aligned to that index with both
images empty. No individual output is paused independently.

For write bank d of output i, input j uses bank 1−d when j ≤ i, otherwise bank d.
Native prev uses i's bank 1−d. Thus no input binding, including unused bindings,
contains the destination attachment. Two input bind groups per assigned output
are prepared on edits/resize, with no texture or bind-group allocation per frame.
The existing renderer owns timing and submission; no new worker, readback loop or
unbounded queue is added. Separate submissions between outputs preserve Metal's
existing pass-order boundary. The common bank changes after all output submissions;
presentation reads that bank. Surface acquisition precedes this work, so a missed
surface frame does not advance history. A partial frame lost with the device is
never reconstructed.

Images are limited to 4096 per axis and the device's limit. Eight full-size RGBA8
images consume at most **512 MiB plus the 4-byte blank**. Preparing a replacement
pair can temporarily retain ten images (640 MiB plus blank); rebuilding the whole
set on resize can retain both generations (1 GiB plus two blanks). These are
owned color-image bounds, excluding swapchain, legacy resources, driver overhead
and in-flight GPU retirement. `texture_bytes` counts both images in each live
pair. This step verifies accounting at small sizes; it does not allocate or
qualify maximum-size histories on every supported GPU.

## Reset and lifecycle contract

| Event | Result |
| --- | --- |
| New session / first assignment | Both history images zero-initialized before first read |
| Successful `.out` replacement | Destination pair reset; unrelated histories unchanged |
| Rejected program or oversized resize | Accepted programs and histories retained |
| Successful resize | All pairs rebuilt at new dimensions, programs retained, history zero |
| `hush` | Four acknowledged transparent-solid replacements reset pairs sequentially, then select o0 |
| Select another Hydra view | Histories continue; only presentation changes |
| Select Legacy | Hydra histories freeze; session clock continues |
| Surface reacquisition only | Histories retained |
| Logical-device recovery | Accepted programs/dimensions/mode/clock retained; all history cleared |
| Close/free | All histories released; return to Legacy |

New WebGPU textures are guaranteed initialized before sampling; first-read GPU
checks verify transparent black without a background clear/readback task. Hush
remains sequential, not an atomic four-output operation or GPU fence. A failure
reports partial progress. Commit acknowledgement means installation, while the
next rendered frame presents the new result. Recovery retains the existing single
restart policy; a second device loss exits with E_GPU_FATAL/code 70. Test-only
readback/injection is absent from installed builds.

## Explicit native differences

- Native `h.prev` means the destination's previous completed image, identical to
  self `src`. The pinned upstream prevBuffer callback selects the active attachment
  under the traced callback order; that read/write alias is deliberately unsupported.
- Successful output replacement resets history, retaining Step 12's lifecycle.
  Upstream program replacement retains its framebuffer pair. Step 14 can refine
  live-edit behavior; this step does not promise seamless history across edits.
- Native sampling retains Step 12's linear filtering and wrapped coordinates;
  pinned upstream output textures request nearest magnification. Timing checks use
  spatially constant tags, so filtering cannot obscure their result. Transformed
  feedback is not claimed pixel-identical to a browser.
- `src` accepts o0…o3 only. Media sources, dynamic parameters and browser callbacks
  remain outside this checkpoint. All 52 operation names are recognized, which is
  not complete argument/runtime parity.

## Verification and use

The production `hydra_outputs_check` executable runs three short temporal cases:
self/prev accumulation and the requested scaled oscillator blend; changing
lower/higher-index frame tags and a cycle; and rollback/reset/hush with bounded
allocation accounting. Existing quadrant, gradient orientation, linear filtering,
linear/sRGB presentation and static settling cases remain enabled.

`tools/check_hydra_recovery.py` uses an isolated gpu-test-hooks executable, freezes
Hydra while Legacy is selected, and reads all four images before/after reset.
This avoids a race with feedback rebuilding on the next frame. It checks surface
reacquisition, one logical-device restart, retained revisions/programs/mode, resize,
hush and second-loss exit. This is not a physical GPU reset. The private
`/scshader/v1/test/hydra/pixel` command accepts one output index and shares the
existing `test/pixel.reply`; it is unavailable without gpu-test-hooks.

Exact results and executable hashes belong in [VERIFICATION.md](../VERIFICATION.md).
The ordered [05_feedback.scd](../../examples/hydra/05_feedback.scd) demonstrates
self-feedback, a two-output cycle, native prev, resize and hush. Recompile the SC
class library, run Step 1, then Steps 2 and 3 after each acknowledgement; Step 0
cleans up. Manual observation of this new example remains separate from automated
command and pixel checks. The user's Step 12 black hush and gray bottom-right
invert(0.5) observations are confirmed expected behavior.
