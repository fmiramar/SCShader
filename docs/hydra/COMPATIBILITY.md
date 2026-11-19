# Hydra compatibility — Step 24

Audited 2026-11-19 against hydra-synth commit
[`9d29a9f4fd8f9081b9759943f38db36f05b9a88f`](https://github.com/hydra-synth/hydra-synth/tree/9d29a9f4fd8f9081b9759943f38db36f05b9a88f)
and SCShader implementation `532af16`. This is a pinned-source compatibility
statement, not a claim about every Hydra version or full browser equivalence.

The [API matrix](HYDRA_API_MATRIX.md) classifies all 52 shader definitions,
the original 32 service/runtime rows, 20 array/dynamic rows and nine additional
inspected source surfaces: **113 rows**, some grouping related names. This audit
updates stale Step 13 statuses using the current code and recorded native proofs.
No renderer, dependency, encoder or network endpoint is added by Step 24.

## How to read the statuses

| Status | Meaning |
| --- | --- |
| **SUPPORTED** | Implemented in the stated native argument/operation scope, with linked evidence. This does not imply JavaScript syntax or browser GPU equality. |
| **NATIVE ALTERNATIVE** | A working native workflow covers the stated purpose, with explicit semantic/API differences. The original browser call or object is not implemented unchanged. |
| **ADAPTER REQUIRED** | The requested live/browser transport needs a separately designed adapter. No such adapter is installed merely because this label appears. |
| **UNSUPPORTED** | No matching core feature/API is provided. A related native mechanism is not proof that the source operation works. |

Implementation and hardware evidence are separate. Current Hydra GPU/media/export
qualification is the recorded macOS arm64 / Apple M5 / Metal scope. Existing
legacy SCShader Windows/Linux results do not qualify new Hydra adapters on those
platforms. Browser/WebGL GPU differential parity is NOT RUN. The user has now
confirmed the corrected Step 23 PNG/MP4 example works in the IDE.

## Implemented native scope and material differences

| Area | Native scope | Limits relevant to porting |
| --- | --- | --- |
| Shader chains | All 52 pinned names; coordinate/color/combine/modulation, src/prev; typed bounded IR and WGSL. | Use SCHydra/SCHydraExpr and explicit slot references. No JS code evaluation, implicit texture-object wrapping or automatic color-to-scalar coercion. [Families](FAMILIES.md), [combines](COMBINE.md), [modulation](MODULATION.md). |
| Dynamic arguments | Native time/wave/sequence/mutable handles; SC Patterns, buses and scheduled controls. | Dynamic arguments only in osc/solid/color/rotate/scale plus custom float inputs. Other operations require literals; arrays are at most 64 finite scalars. Only linear/easeInOutQuad curves. [Numeric API](PARAMETERS.md). |
| Output/history | Four outputs, paired histories, src/prev, independent validated replacement and quadrants. | Fixed numeric render order; n−1/n−2 history rules and RGBA8 storage are explicit. Browser GPU equality, all precision/undefined-domain behavior and arbitrary slot counts are not claimed. [Temporal contract](TEMPORAL_SEMANTICS.md). |
| Clock/callbacks | Session seconds, speed/BPM/reset, scheduled parameters; native event loop owns rendering. | No tick(dt)/autoLoop=false/offline renderer or synchronous per-frame update/afterUpdate callbacks. SC producer rates differ from GPU frame callbacks. [Clock](CLOCK.md), [Patterns](PATTERNS.md). |
| Hush/cleanup | h.hush clears four output histories and selects o0. | It does not stop media sources, SC Patterns, mapped buses or reset nonexistent JS callbacks. Stop owned producers and clearSource/free explicitly. Browser hush clears sources and callbacks too. [Lifecycle](LIVE_CODING.md), [source clear](IMAGES.md). |
| Audio | Existing scsynth analysis, FFT-bin/waveform-point scalar buses, mapBus smoothing/scale/bias/cutoff. | Values differ from Meyda/WebAudio loudness-derived bands; no matching a object/settings/global aN callbacks or DOM graph. [Audio](AUDIO.md). |
| Input | ShaderWindow inputEnabled, mouse/key/resize/focus actions and metrics. | Mouse events use normalized window x/y in 0…1, top-left origin; button events are individual platform codes. No identical JS button bitmask/modifier/enabled object and no direct Hydra mouse numeric handle. Map events into parameters explicitly. [Window protocol](../../protocol/protocol.md). |
| Local images/video | Managed s0…s3, bounded local images, explicit external FFmpeg file decoding. | Not URL fetching, HTML media objects, arbitrary regl parameters or a shared live texture bridge. Video adapter is 640×360 RGBA at 30 FPS with documented timing/drop behavior. [Images](IMAGES.md), [video](VIDEO.md). |
| Camera/screen | Explicit macOS device enumeration, native camera helper and external FFmpeg display capture. | Permission/lifecycle belong to native adapters. Whole displays rather than browser/window-picker objects; other native platforms remain unqualified. [Capture](CAPTURE.md). |
| Custom operations | Typed, session-local WGSL functions in all five categories. | Pasted GLSL/JS setFunction objects, arbitrary function dependencies and source-level compatibility are unsupported. [WGSL API](CUSTOM_FUNCTIONS.md). |
| Resolution/export | Physical render resolution independent of Retina window size; aspect-preserving presentation, opt-in PNG/MP4. | Sources retain media dimensions. No Blob callback, MediaStream/captureStream object or WebM recorder. External H.264/MP4 can drop frames; writtenFrames/FPS determines duration. [Export](EXPORT.md). |

### Constructor options are not interchangeable

The [pinned constructor](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js#L22)
exposes these options. Combined matrix rows do not hide their individual status.

| Source option | Status | Native behavior |
| --- | --- | --- |
| width, height | NATIVE ALTERNATIVE | ShaderWindow chooses logical window size; h.setResolution chooses physical render pixels. |
| numSources, numOutputs | SUPPORTED | The default four/four contract is supported; other counts and createSource growth are unsupported. |
| pb | ADAPTER REQUIRED | No injected JS patch-bay provider or network-video adapter. |
| canvas | ADAPTER REQUIRED | No HTMLCanvasElement import, DOM attachment or CanvasRenderingContext2D return object. |
| makeGlobal | UNSUPPORTED | Session-scoped SC objects; no window/global/prototype mutation. |
| autoLoop | UNSUPPORTED | No manual browser tick ownership; native event loop is automatic. |
| precision | NATIVE ALTERNATIVE | WGSL f32 math and explicit target formats; GLSL lowp/mediump/highp selection is unsupported. |
| extendTransforms | NATIVE ALTERNATIVE | Explicit acknowledged WGSL registration. Browser constructor object/array forms and their revision-specific behavior are unsupported. |
| debug | NATIVE ALTERNATIVE | ShaderServer diagnostics/status and the existing renderer overlay; no global JS console/debug flag equivalence. |
| detectAudio | NATIVE ALTERNATIVE | Audio analysis is explicitly created in SC; startup never automatically opens the microphone. |
| enableStreamCapture | NATIVE ALTERNATIVE | Export is explicitly requested; no automatic canvas.captureStream or live MediaStream is created. |

## Tier C audit: DOM, JavaScript and browser audio UI

The [pinned source adapter](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-source.js)
uses real video/image/canvas elements and regl texture objects. initCanvas returns
a 2D drawing context; init({src,dynamic}) holds an external object and optionally
uploads its changing pixels. Native source images/decoders do not supply those
objects. The [HTML canvas specification](https://html.spec.whatwg.org/multipage/canvas.html#the-canvas-element)
defines a drawing context and associated bitmap/serialization behavior; importing
pixels alone does not reproduce that browser API.

The [pinned runtime](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js#L156)
implements loadScript by creating a script element in document.head. Its
[sandbox](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/eval-sandbox.js)
evaluates JavaScript and can bind synth properties to window. sclang evaluates a
different language; the renderer accepts bounded typed data/WGSL rather than JS.
Matching these calls would require a JavaScript host plus the browser APIs they
use: DOM, canvas/media objects, event scheduling and their lifecycle. An embedded
JS evaluator alone would not provide them. No JS interpreter or browser host is
added, and no loadScript/eval network endpoint is exposed.

`a.show()/hide()` and the audio draw method manipulate a browser canvas. There is
no matching native band-graph overlay. The existing diagnostic overlay reports
renderer health rather than reproducing the WebAudio analyzer/UI.

## Tier C audit: initStream, patch-bay providers and WebRTC

The pinned source does **not** define a complete network-video protocol.
[initStream](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-source.js#L66)
requires a supplied pb, calls pb.initSource(streamName), and listens for its
`got video` event containing a matching name and video object. pb defaults to null;
the pinned package lists regl, raf-loop and Meyda dependencies, not a bundled
patch-bay implementation. A native adapter must first select and pin an actual
provider/peer protocol. A stream name alone is insufficient to implement it.

A WebRTC adapter would need a WebRTC stack or browser helper, agreed codecs and
media conversion, peer/session lifecycle, ICE configuration and a separate
signaling service. WebRTC deliberately leaves application signaling outside the
peer-connection API; ICE may use STUN and TURN for connectivity. These are explicit
integration/deployment choices, not additional OSC message names.
[WebRTC peer connections](https://webrtc.org/getting-started/peer-connections),
[W3C WebRTC Recommendation](https://www.w3.org/TR/2025/REC-webrtc-20250313/).

A real adapter also needs application-level peer authorization, capture consent,
credential handling, encrypted transport and disconnect/reconnect behavior.
The [WebRTC security architecture](https://www.rfc-editor.org/rfc/rfc8827.html)
describes the media/security trust model; transport encryption does not choose
which peer/application should be trusted. Its dependencies, licensing and runtime
provisioning require their own exact-build assessment. No security or connectivity
qualification is claimed for a nonexistent adapter.

Decision: **ADAPTER REQUIRED, not implemented**. The demonstrated session needs
local camera/display/media input and file export, now implemented. No peer,
signaling deployment, remote stream provider, shared GPU API or external producer
has been requested or supplied for validation. Keep network video optional and
separate; do not add a server, new listening address, WebRTC stack or credentials
to all installations. The renderer's current OSC endpoint remains IPv4 loopback.
This is a scope decision from the observed use case, not a claim that remote
streams are undesirable or impossible.

## External texture interoperability decision

TouchDesigner, Processing, openFrameworks and a browser Hydra host have no shipped
SCShader live texture adapter. In particular, the existing one-row float-buffer
protocol and ShaderTexture's static image path are not an arbitrary RGBA streaming
or zero-copy GPU-sharing API, and do not bind themselves to Hydra source slots.

The working interoperability paths are explicit local image/video files or the
qualified whole-display capture adapter. They incur their documented latency,
conversion and permission behavior. Naming a source application does not qualify
live texture compatibility. A future named adapter could import pixels into the
existing managed source ownership model without exposing DOM or JS objects.

No wire protocol is defined in Step 24: there is no concrete producer/consumer
pair or required transport to justify freezing one. Before such a proposal,
identify both endpoints and target platforms, pixel/channel/color/alpha layout,
size/rate limits, frame timestamp/drop semantics, resource/queue ownership and
cleanup. Compare an out-of-process pixel transport with any platform-specific GPU
sharing dependency, then validate actual frames and disposal on both endpoints.
These are criteria for a later adapter proposal, not a protocol implemented here.

## Evidence and remaining audit

The matrix links native code contracts and the existing bounded evidence in
VERIFICATION.md. `tools/check_hydra_compatibility.py` checks inventory completeness,
status vocabulary, evidence links and native/pinned operator-name coverage. With
`--reference build/hydra-reference` it also verifies the pinned commit, declared
shader categories, array curves and source/constructor surfaces. It reads source
metadata and never executes browser code, starts media, or opens network ports.

Step 24 changes documentation and its audit tooling only. No new browser GPU,
WebRTC, external texture, Windows/Linux/Intel or long-run qualification is added.
Step 25 still needs the representative compatibility/regression corpus, precise
argument/default/domain review and updated help/legacy checks. Explicit status
coverage closes this audit step; it does not close those behavioral release gates.
