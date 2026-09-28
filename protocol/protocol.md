# SCShader OSC protocol v1

Transport is UDP bound to IPv4 loopback only. The default renderer port is `57140`. Strings are OSC strings; resource IDs, sequence numbers, protocol versions, and ports are OSC `int32`; numeric time/value inputs accept OSC float, double, or integer values when representable.

## Versioned messages

### Client to renderer

```text
/scshader/v1/hello
    protocolMajor:int (= 1)
    replyPort:int
    clientName:string

/scshader/v1/ping
    sequence:int
    clientTime:double

/scshader/v1/status

/scshader/v1/schedule/clear

/scshader/v1/window/resize
    logicalWidth:int (> 0)
    logicalHeight:int (> 0)

/scshader/v1/window/title
    title:string (non-empty)

/scshader/v1/window/position
    logicalX:int
    logicalY:int

/scshader/v1/window/fullscreen
    enabled:int (0 or 1)

/scshader/v1/window/borderless
    enabled:int (0 or 1)

/scshader/v1/window/vsync
    enabled:int (0 or 1)

/scshader/v1/window/cursor-visible
    visible:int (0 or 1)

/scshader/v1/window/input-enabled
    enabled:int (0 or 1)

/scshader/v1/window/front

/scshader/v1/window/metrics

/scshader/v1/window/close

/scshader/v1/shader/create
    resourceID:int (> 0)
    sourcePath:string
    sourceType:string ("wgsl", "glsl", or "shadertoy"; default "wgsl")

/scshader/v1/shader/reload
    resourceID:int

/scshader/v1/shader/free
    resourceID:int

/scshader/v1/texture/create
    textureID:int (> 0)
    imagePath:string

/scshader/v1/texture/free
    textureID:int (> 0)

/scshader/v1/shader/texture
    resourceID:int (> 0)
    name:string (= "source")
    textureID:int (> 0)

/scshader/v1/shader/feedback
    resourceID:int (> 0)
    source:string ("previous" or "framebuffer")
    amount:float (0 through 1)

/scshader/v1/buffer/create
    bufferID:int (> 0)
    length:int (1 through 16384)

/scshader/v1/buffer/write
    bufferID:int (> 0)
    start:int (>= 0)
    values:blob (one or more big-endian IEEE-754 float32 values)

/scshader/v1/buffer/free
    bufferID:int (> 0)

/scshader/v1/shader/buffer
    resourceID:int (> 0)
    name:string (= "spectrum")
    bufferID:int (> 0)

/scshader/v1/graph/set
    resourceID:int (> 0), ... (unique pass IDs in render order; no arguments restores normal selected-Shader rendering)

/scshader/v1/uniform/f
    resourceID:int
    name:string (any reflected float, including "amount")
    value:float

/scshader/v1/uniform/set
    resourceID:int
    name:string
    type:string (float, int, uint, bool, vec2, vec3, vec4, mat2, mat3, mat4)
    values:blob (tightly packed big-endian 32-bit components)

/scshader/v1/uniform/glide
    resourceID:int
    name:string
    type:string
    values:blob
    duration:float (finite, non-negative seconds)
    mode:string (step, linear, smooth)

/scshader/v1/uniform/get
    resourceID:int
    name:string
    requestID:int

/scshader/v1/timing/marker
    sequence:int

/scshader/v1/quit
```

### Renderer to client

```text
/scshader/v1/ready
    protocolMajor:int
    rendererVersion:string
    backendName:string
    deviceName:string

/scshader/v1/pong
    sequence:int
    clientTime:double
    rendererTime:double

/scshader/v1/status.reply
    fps:double
    frameIndex:int
    gpuFrameMs:double
    cpuFrameMs:double
    scheduledEventCount:int
    shaderCount:int
    textureCount:int
    bufferCount:int
    windowCount:int
    scheduledPayloadBytes:int
    rejectedScheduled:double
    droppedContinuous:double
    lateEventCount:double
    textureBytesEstimate:double
    showStats:int
    gpuRecoveryCount:int
    lastCompileOK:int
    suppressedDiagnostics:double

/scshader/v1/gpu/recovered
    recoveryCount:int
    message:string
    backendName:string
    deviceName:string

/scshader/v1/schedule/cleared
    removedCommandCount:int

/scshader/v1/window/metrics.reply
    logicalWidth:int
    logicalHeight:int
    pixelWidth:int
    pixelHeight:int
    pixelRatio:double
    logicalX:int
    logicalY:int
    fullscreen:int (0 or 1)
    borderless:int (0 or 1)
    vsync:int (0 or 1)
    cursorVisible:int (0 or 1)
    title:string

/scshader/v1/input/mouse
    kind:string ("move", "down", or "up")
    normalizedX:double (0 through 1)
    normalizedY:double (0 through 1)
    button:int (-1 for movement; otherwise platform button code)

/scshader/v1/input/key
    state:string ("down" or "up")
    key:string
    repeat:int (0 or 1)

/scshader/v1/input/resize
    logicalWidth:int
    logicalHeight:int
    pixelWidth:int
    pixelHeight:int
    pixelRatio:double

/scshader/v1/input/focus
    focused:int (0 or 1)

/scshader/v1/shader/created
    resourceID:int

/scshader/v1/shader/reloaded
    resourceID:int

/scshader/v1/shader/freed
    resourceID:int

/scshader/v1/shader/reflection
    resourceID:int
    uniformName:string, uniformType:string, ... (one complete snapshot)

/scshader/v1/uniform/value
    resourceID:int
    name:string
    requestID:int
    type:string
    values:blob

/scshader/v1/timing/marker.reply
    sequence:int
    scheduledRendererTime:double
    appliedRendererTime:double

/scshader/v1/error
    severity:string
    subsystem:string
    resourceID:int
    code:string
    message:string
```

`shaderCount` counts client-created shader resources. It excludes the renderer's built-in fallback pipeline, so a newly booted renderer reports zero until the first `/shader/create` succeeds.

Error codes currently emitted are `E_PROTOCOL`, `E_WINDOW`, `E_BAD_ARGUMENT`, `E_QUEUE_FULL`, `E_SCHEDULE_LIMIT`, `E_LATE_EVENT`, `E_RESOURCE_NOT_FOUND`, `E_SHADER_VALIDATE`, `E_SHADER_COMPILE`, `E_TEXTURE_LOAD`, `E_GPU_DEVICE_LOST`, and `E_GPU_FATAL`. Malformed datagrams and unsupported addresses produce protocol diagnostics without terminating the renderer.

Version 0.0.14 adds `/scshader/v1/diagnostics/overlay enabled:int` (0 or 1; defaults off) and the last five status fields above. `lateEventCount` is cumulative applied commands over 50 ms late, not bundle containers. Texture bytes estimate owned GPU image/data/frame-history/overlay allocations; they exclude swapchain/driver overhead and CPU image caches. Last compile OK describes the most recent explicit or watched attempt, not all source files. A failed compile preserves its last-valid pipeline.

On device loss the renderer emits warning `E_GPU_DEVICE_LOST`, attempts one in-process restart, then sends `gpu/recovered` with a feedback-history reset notice. IDs, shader sources/controls/ramps, decoded images/data, graph/bindings, frame counter, renderer clocks, and pending scheduler work survive. Feedback and intermediate history clear to black. Recovery takes time and may make commands late; it does not guarantee uninterrupted output or protection from driver hangs. A second loss, failed recovery, or uncaptured GPU validation/OOM error reports `E_GPU_FATAL` and exits 70. A lost surface is recreated once before a successful present is required. The compile-time `gpu-test-hooks` endpoints are absent from ordinary release builds.

Image decode failure retains the new ID with a 2x2 magenta/black placeholder and reports `E_TEXTURE_LOAD`; duplicate IDs remain unchanged. Dimensions cannot exceed 4096 pixels per axis or the device limit if smaller. Cached RGBA data adds four CPU bytes per pixel for device recovery. Float32 buffers remain capped at 16,384 values and additionally at the adapter's maximum one-row texture width. The SC sender uses 2048-value chunks so each datagram fits the usual macOS UDP ceiling.

`gpuFrameMs` is `-1.0` when the renderer has no timestamp-query measurement available. `scheduledEventCount` counts pending commands **including bundle containers**, not only leaf messages. The current renderer reports one window and zero client-created shaders after a successful boot; its built-in fallback pipeline is not a client resource. The last three fields were appended in 0.0.13: charged future payload bytes and cumulative rejected-future/dropped-incoming packet counts. The counters do not reset on schedule clearing. Old clients may ignore the tail; new SC classes default missing tail fields to zero for older renderers.

`schedule/clear` discards future work when the command executes and acknowledges the removed command count. It does not flush already-applied or incoming work, cancel active glides, or stop a client's producer. Send it as an immediate message after stopping producers. An immediate `ping` is a command barrier, not a wait for future events. The SC method also cancels its local timing-marker callbacks; cached Shader target values are not rolled back.

## Current shader-resource ABI

Each successful `shader/create` retains an independent fullscreen pipeline and compact uniform state. Without a graph, the most recently created resource is presented; `graph/set` selects a validated linear pass order. A successful `shader/reload` compiles the candidate before replacing that resource's pipeline. If compilation or source-file reading fails, the prior valid pipeline remains active and the renderer sends `E_SHADER_COMPILE`.

Native WGSL retains the built-in `Uniforms` binding and `vs_main`/`fs_main` entry points. An optional flat group-0/binding-4 uniform block is reflected through Naga, supporting the ten types above with a 127-member/16-KiB bound and names of at most 64 UTF-8 bytes. `amount: float` remains a reserved binding-0 control. GLSL uses Naga's fragment frontend with the built-in vertex stage; `sourceType: "shadertoy"` wraps `mainImage(out vec4, in vec2)` in that scaffold. See [the exact ABI](../docs/CONTROLS.md); this is not arbitrary-resource reflection or universal GLSL support.

From 0.0.12, reflection is one complete name/type snapshot sent before each successful create/reload acknowledgement. The language replaces its prior schema instead of merging stale names. A snapshot containing only `amount, float` has the same wire shape as earlier renderers. Clients using custom controls must use matching 0.0.12-or-later classes and renderer.

Typed blobs contain exactly 1, 2, 3, 4, 9, or 16 words, as appropriate. Float types use IEEE Float32; int uses two's-complement i32; uint uses u32; bool uses u32 0 or 1. Vectors and matrices are tightly packed on the wire, matrices in column-major order; the renderer adds GPU padding. `set` and the legacy `uniform/f` cancel a control's current ramp. `glide` starts from the current value at command application time; step/zero duration is immediate, linear uses elapsed/duration, and smooth uses cubic smoothstep. Integer/bool ramps reject non-step modes. A get reply echoes the request ID and samples CPU-side renderer state, not GPU pixels. Missing replies time out at the language layer after two seconds.

Invalid values produce `E_BAD_ARGUMENT` without changing that control. Failed reloads preserve the pipeline, reflection, values, and ramps; successful reloads retain controls whose names and types match. File watching now covers all live shaders, including inactive passes.

For the Shadertoy wrapper, `iResolution`, `iTime`, `iTimeDelta`, `iFrame`, `iMouse`, and UTC `iDate` are populated each frame. `iChannel0` through `iChannel3` are accepted but all refer to the same SCShader `source` texture. `scPhase` is renderer time, `scAmplitude` is the compact `amount` control, and `scBeat`, `scTempo`, `scPitch`, `scPitchConfidence`, and `scOnset` are currently zero. Create/reload diagnostics include the source path and fragment language/stage plus the Naga/backend message when supplied.

## Diagnostic flood limits

Version 0.0.16 appends `suppressedDiagnostics` to status without moving the existing
fields. Structured error/warning replies and their corresponding stderr entries
share one process-wide token bucket: burst 20, refilling at 20 per second. Excess
diagnostics increment this lifetime counter without logging or replying; status,
pong, successful resource replies, and command execution are not throttled by it.
`E_GPU_DEVICE_LOST` and `E_GPU_FATAL` bypass the budget. Message bodies are limited
to 4096 UTF-8 bytes plus a truncation marker; codes and resource IDs stay intact.

The budget is shared across senders and error codes with fixed storage, not an
unbounded map of client addresses. Under an error flood, unrelated ordinary
errors may be suppressed too: do not assume every failed request produces a
reply. Stop the offending traffic, inspect status, and retry after the budget
refills. GPU recovery does not reset the counter/budget; process restart does.
This bounds structured diagnostics, not all transport/startup logging, GPU
compilation cost, total live-resource allocations, or kernel-level UDP loss.

## Textures and feedback

`texture/create` decodes a static PNG, JPEG, or PNM image in the renderer process. `shader/texture` currently binds only `source`. `shader/feedback` samples the last completed offscreen renderer frame, then writes the next frame to the other of two render targets before swapping them. This ping-pong arrangement avoids sampling from the same texture that is being rendered and resets its history on resize. Texture errors return structured diagnostics and leave the renderer process running.

## Visual bus batching and audio analysis

`ShaderBus` is a language-side abstraction, not a new renderer OSC address. At each 60 Hz language-side smoothing step, it groups all mapped float uniforms for one Shader into an ordinary or timestamped bundle of `/scshader/v1/uniform/f` messages. Any reflected float can be mapped; buses do not poll `scsynth`. Each new bus value replaces a renderer-side glide for that control.

`ShaderAnalysis` is likewise an audio-server-to-language bridge. Its generated `scsynth` SynthDef sends controlled-rate `/scshader/analysis` `SendReply` packets directly to the language; those packets never cross the renderer transport. Each reply has the normal `SendReply` node ID and reply ID followed by, in fixed order, amplitude, pitch, pitch confidence, spectral centroid, spectral flatness, onset, and zero-crossing data. The selected values enter their ShaderBuses and then use the existing uniform bundle path.

## Float buffers and stream textures

`buffer/create`, `buffer/write`, and `buffer/free` own a moderate-size renderer resource. `buffer/write` uses a single OSC blob rather than thousands of OSC float arguments; floats are network-order/big-endian IEEE-754 binary32. The renderer stores each resource as a padded one-row `R32Float` texture. `shader/buffer` exposes only the fixed `spectrum` binding at this stage. It is non-filterable, so WGSL uses `textureLoad` rather than a sampler to preserve exact bin indices. `bufferCount` in status excludes the renderer's internal zero fallback texture.

`ShaderFFTTexture` and `ShaderWaveformTexture` use `SendReply` only from `scsynth` to `sclang`, under `/scshader/fft` and `/scshader/waveform` respectively. Those private audio-server replies carry normal OSC float arguments; after validation, the language transfers each complete update to the renderer as the `buffer/write` blob above. They are not renderer protocol addresses.

## Multipass graph sequencing

`graph/set` replaces the renderer's current pass sequence only after all listed Shader resources are present. It accepts a linear render order rather than arbitrary edges; `ShaderGraph` performs the stricter language-side topology validation. Each non-terminal pass renders to one of two renderer-owned graph targets, and its completed target is the following pass's `source`; the terminal pass renders to the window and feedback target. The renderer keeps the prior graph when resource validation fails. Graph targets are recreated and cleared on resize.

## Window and input behavior

The renderer owns one native window. `window/resize` and `window/position` use logical display units; `window/metrics.reply` supplies both logical and framebuffer dimensions plus their pixel ratio. The initial monitor is selected through renderer startup arguments, so the protocol does not expose a runtime monitor-switch command. Fullscreen is borderless fullscreen on the current monitor. `window/close` exits the renderer because no second renderer window exists in this milestone.

Input messages are emitted only after `window/input-enabled 1`. Mouse coordinates are normalized against the current framebuffer and clamped to the inclusive 0 through 1 range. The renderer keeps only the latest mouse movement between rendered frames and emits it immediately before that frame; button, key, resize, and focus notifications are not coalesced. Key strings identify logical window-system keys and are not physical scan codes.

## Phase-0 aliases

For the original feasibility example, the same renderer also accepts:

```text
/scshader/hello replyPort:int
/scshader/ping sequence:int clientTime:double
/scshader/uniform/f resourceID:int name:string value:float
/scshader/quit
```

Replies use `/scshader/ready`, `/scshader/pong`, and `/scshader/error` when the initiating request used the unversioned form. New code should use v1.

## Queue, clock, and bundle behavior

Decoded packets cross from the OSC thread to the window/render thread through a bounded 256-packet queue. A whole bundle is one incoming entry. Packets containing only continuous uniform/buffer updates use non-blocking enqueue; overload drops that whole packet, increments `droppedContinuous`, and emits `E_QUEUE_FULL` at power-of-two counts. Other packets wait for queue space (or shutdown); the UDP socket itself cannot guarantee lossless delivery. There is no per-key coalescing at this layer.

Before recursive OSC decoding, an iterative preflight enforces at most 256 packet elements including containers, depth at most 16, valid bundle lengths, and no OSC array type tags. Typed vectors/matrices use blobs, not OSC arrays. Parsing/validating any member fails the entire packet with `E_PROTOCOL` before enqueue; accepted direct bundle members execute consecutively. This follows the ordering/atomic-dispatch model in the [OSC 1.0 specification](https://opensoundcontrol.stanford.edu/spec-1_0.html), not database-style rollback: runtime failures such as missing resources or shader compilation do not undo earlier valid commands.

File-backed live shaders are polled at most four times per second, including inactive passes. A changed source file is compiled before replacing the current pipeline; success sends `shader/reloaded`, while a failure sends `E_SHADER_COMPILE` and preserves the last valid pipeline. Compilation runs on the renderer thread and can temporarily delay frames/commands.

An OSC bundle timetag is converted at receipt from OSC's UTC/NTP calendar time to a deadline in the renderer process's monotonic clock. The renderer then keeps future events in timestamp order, preserving packet order for equal timetags, and applies due events before the next eligible render frame. The conversion makes one wall-clock delta at receipt; once queued, wall-clock adjustments cannot move an event. The special OSC immediate timetag and ordinary messages remain immediate.

A nested immediate bundle inherits its enclosing deadline; a dated nested bundle cannot precede its parent. Later nested bundles are separately scheduled when their parent executes, as permitted by OSC's nested-bundle ordering exception. More than one hour of lookahead is rejected with `E_PROTOCOL` at packet validation. The future scheduler admits at most 8192 commands including containers and 16 MiB of charged live payload (owned string/vector capacities and active scheduled-record sizes). This is not total RSS or retained heap capacity. Admission is whole-packet and rejection increments `rejectedScheduled` with `E_SCHEDULE_LIMIT`; continuous rejection warnings are throttled to power-of-two totals.

Each dispatcher pass interleaves up to 128 due/incoming pairs, yielding between atomic packets after approximately 2 ms. This is a cooperative service budget, not a frame-time guarantee: one bundle or compilation can take longer. Due work cannot monopolize an unbounded drain before incoming status/clear commands. Rendering and lateness remain frame-oriented under load.

`/scshader/v1/ping` and `/scshader/v1/pong` form a lightweight NTP-like estimate. `ShaderServer` records the SuperCollider send time, renderer reply time, and SuperCollider receive time, retaining the lowest-round-trip estimate of `rendererTime ≈ Main.elapsedTime + rendererClockOffset`. The two process clocks are never assumed to have a shared epoch. The estimate is useful for diagnostics and coordination; OSC bundle scheduling itself uses the bundle's standard timetag.

If a bundled event is already due, it is applied immediately. If it is more than 50 ms overdue at application, the renderer additionally reports `E_LATE_EVENT`; it never moves an overdue event into the future. `/scshader/v1/timing/marker` is a test/measurement command. Its reply records the renderer-clock deadline and the time at which the renderer applied the marker, after preceding same-bundle commands. It does not measure GPU presentation or display scan-out.
