# Hydra output protocol — implementation checkpoint

2026-11-13. Additive to [SCShader v1](protocol.md) and implementing the first
output/temporal portion of [ADR 001](../docs/hydra/ARCHITECTURE.md). 50 procedural/composition/modulation operators plus output-only `src` and native `prev`,
four paired output histories and presentation are available. Steps 19–20 add image
and opt-in local video sources; Step 21 adds macOS camera/display capture.
Step 15 adds the bounded numeric API below.

All suffixes below use `/scshader/v1/hydra/`. Fields use ordinary OSC types:
`i` = int32, `s` = string, `b` = blob. Clock and single numeric parameter controls may use inherited future timetags.
Structural requests must be immediate, including when nested in bundles. Reject timed structural packets before scheduling; a
malformed member rejects the whole packet. The receiver remains loopback-only.

| Request | Arguments | Reply (after address) |
| --- | --- | --- |
| capabilities | request:i, abiMajor:i, replyPort:i | capabilities.reply(request, epoch:s, 1:i, 0:i, limits:b, features:s) |
| open | request:i, epoch:s, clientNonce:s, abiMajor:i, replyPort:i | opened(request, epoch, session:i) |
| begin | epoch, session, tx:i, output:i, expectedRevision:i, totalBytes:i | ack(epoch, session, tx, -1:i) |
| chunk | epoch, session, tx, index:i, bytes:b | ack(epoch, session, tx, index) |
| commit | epoch, session, tx | committed(epoch, session, tx, output:i, revision:i) after successful installation |
| abort | epoch, session, tx | aborted(epoch, session, tx) |
| status | epoch, session, request, queriedTx:i | status.reply(epoch, session, request, mode:i, revisions:b, state:s, code:s, message:s) |
| present | epoch, session, sequence:i, mode:i (-1 Legacy, 0..3 output, 4 quadrants) | presented(epoch, session, sequence, mode) |
| close | epoch, session, request | closed(epoch, session, request) |

IDs/requests/transaction numbers are positive signed int32, output is 0..3,
revision is nonnegative, initially 0, and increases once per accepted commit. ABI major must be 1. Epoch and
nonce are nonempty ASCII of at most 64 bytes; port is 1..65535. Open retries with
same sender endpoint, nonce and reply port return the existing handle; a different
owner fails. Epoch/session guard all subsequent requests. Closing releases the
session; retrying a closed handle currently reports `E_HYDRA_SESSION`.

`limits` is exactly five big-endian u32 values, in order: maximum IR bytes
(65536), chunk bytes (2048), nodes (256), depth (64), expanded operations (4096).
`features` is `staging-ir-v1;wgsl-v1:<comma-separated supported registry names>;output-o0-v1;outputs-temporal-v1;numeric-v1;clock-v1;image-sources-v1;video-ffmpeg-v1`. The 52 names and their defaults/types are in `renderer/src/hydra/registry.rs`; discovery order is not an ABI guarantee.
This narrower discovery payload supersedes the ADR's provisional full-limit
payload for the staging feature only. Future compiler/source/presentation
capabilities must explicitly document their additional fields before advertising.
Revisions blob is four big-endian int32 values ordered o0..o3 (independent revisions). Mode -1 is Legacy, 0..3 selects an output and 4 selects quadrants. `state` is uploading, committed, failed
or unknown. For a committed transaction, message contains its decimal revision. Status does not claim that an evicted
transaction never executed; it reports unknown.

One upload per session, at most 64 KiB retained in up to 32 chunks. Chunks have
index 0..ceil(totalBytes/2048)-1; all except the last must have 2048 bytes, and the
last has the exact remainder. Identical duplicates acknowledge without changing
data. Conflicting duplicates, invalid sizes and incomplete commits fail. Idle
expiry is 10 seconds, absolute lifetime 30 seconds. Expiry is polled by the main
dispatcher even without more requests. Device recovery aborts uncommitted uploads.

A begin retries only its exact current metadata. New transaction IDs strictly
increase. Abort/commit/expiry release upload storage; retain the latest 32
terminal results plus a transaction high-water mark. Stale begin never allocates
again. Commit retry returns its retained success/failure without repeated installation.
Expected revision is checked before preparation; stale input cannot replace output.
Presentation sequences increase monotonically; an exact duplicate selection
acknowledges without changing state, and older sequences fail as stale.
An incomplete commit keeps staging so missing chunks can still arrive.

Errors currently use the existing `/scshader/v1/error` schema with subsystem
`hydra`, resourceID 0, stable codes `E_HYDRA_VERSION`, `E_HYDRA_SESSION`,
`E_HYDRA_STALE`, `E_HYDRA_LIMIT`, `E_HYDRA_UPLOAD`, `E_HYDRA_IR`,
`E_HYDRA_CAPABILITY`, and `E_HYDRA_COMPILE`. They share existing diagnostic
rate/message limits. Retained status reasons are bounded to 3072 UTF-8 bytes
without splitting a character, so the reply stays within 4 KiB. Parse
errors use `E_PROTOCOL`. Query status to reconcile lost/suppressed commit errors;
replies are bounded. The ADR's richer Hydra error tuple is reserved for later
node/output diagnostics; clients must not assume it exists yet.

The IR layout is HYIR 1.0 from ADR 001. The decoder accepts 32 procedural, seven combine, eleven combineCoord, output-only src and zero-argument native prev
operations with scalar arguments (tag 0,
f32) and the literal vec4 argument for `sum` (tag 5, four f32 components),
encoded in network order. Combine arguments begin with an expression reference (tag 2, earlier node ID
as u32). All seven combine and eleven combineCoord operations compile, including nested branches. Numeric declarations and dynamic scalar arguments use HYIR 1.1 as specified below.
Media sampling and other unsupported argument forms fail explicitly. `src` accepts tag 4 followed by a u8 output slot 0..3.
The decoder verifies size/version, category/arity/types,
finite f32 inputs, unique positive node IDs, earlier-node references, root reachability,
no trailing data, depth and expansion bounds before any candidate installation.

SC client: `SCHydra.new(server)` negotiates when the server is already running;
`waitForReady(action, onFailure)` reports transport-session readiness.
`expr.out(output, action)` uploads a snapshot and returns an asynchronous result.
Each awaited reply retries up to three attempts of 0.5 seconds. Commit timeout
queries status; unknown outcomes report `E_HYDRA_UNKNOWN`. One upload is allowed
per controller; local snapshots remain available offline. `free` stops only the
controller's tasks/responders and sends close without stopping the ShaderServer.

Batch `params` and media sampling remain unimplemented; single-value `parameter` updates are available. SC hush uses four sequential ordinary commits and a presentation request.
See [four-output semantics and limitations](../docs/hydra/OUTPUTS.md). The SC
client maintains accepted revisions and reconciles a timed-out commit through
status. `present` is immediate-only at this stage. A new output is only made
visible after explicit presentation selection; an accepted replacement of a selected
output becomes visible on its next frame. Close releases the output and selects Legacy.

Output cycles are valid temporal reads. Fixed numeric output order gives self/lower-index reads the previous image and higher-index reads the older image (n−2 during uninterrupted rendering). Blank slots sample transparent black and blank selection is allowed. Native `prev` reads the destination’s previous image. Successful replacements reset that output’s paired history; resize/recovery reset all histories. See [the temporal contract and native differences](../docs/hydra/TEMPORAL_SEMANTICS.md).

Display resources are prepared before presentation sequence/mode changes. A preparation failure returns E_HYDRA_LIMIT and retains the previous selection and sequence.

Step 14: `SCHydra.clear(output, action)` is a client convenience using an ordinary transparent-solid commit. It increments only that output revision and retains selection. Hush uses four sequential clear calls. Matching live expression pipelines share compiled resources; no protocol fields change. See [live editing](../docs/hydra/LIVE_CODING.md).

Step 15 advertises `numeric-v1`, accepts HYIR 1.1 numeric declarations and scalar tag 1, and implements `parameter(epoch:s,session:i,sequence:i,output:i,revision:i,parameterID:i,value:f)` with ack. [PARAMETERS.md](../docs/hydra/PARAMETERS.md) defines the bounded layout, supported operators, immediate control ordering and native differences. Earlier parameter-unavailable statements describe prior checkpoints.

Step 16 adds `clock(epoch:s,session:i,sequence:i,bpm:f,speed:f,phase:f,setPhase:i)` returning ack, timed admission of clock/parameter controls, and HYIR 1.1 numeric kinds 4/5/6 for beat/frame/delta. [CLOCK.md](../docs/hydra/CLOCK.md) defines units, bounds, ordering, reset and application timing. Structural timed requests still fail atomically.

Step 18 makes parameter sequence guards independent per `(output revision, parameter ID)`, using a fixed 4×256 table cleared on accepted output replacement. Distinct controls may interleave without rejecting each other; same-parameter stale updates remain rejected. Wire fields do not change.

Step 19 advertises `image-sources-v1` and adds immediate `image(epoch:s,session:i,sequence:i,slot:i,path:s)` with ack, empty-path clear, per-slot retry/sequence guards and E_HYDRA_IMAGE. HYIR sampler tag 3 is source s0…s3, distinct from output tag 4. [IMAGES.md](../docs/hydra/IMAGES.md) defines decoding, lifecycle, aspect/color policy and limits; earlier media-unavailable statements describe previous checkpoints.

Step 20 advertises `video-ffmpeg-v1` (adapter available; an external FFmpeg executable
is still required). All video requests are immediate-only and session/owner scoped:

| Request | Arguments | Reply |
| --- | --- | --- |
| video | epoch:s, session:i, sequence:i, slot:i, path:s, decoder:s, loop:i, rate:f | ack(epoch, session, sequence), after worker acceptance |
| video-control | epoch:s, session:i, sequence:i, slot:i, command:s, value:f | ack(epoch, session, sequence) |
| video-status | epoch:s, session:i, request:i, slot:i | video-status.reply(epoch, session, request, slot:i, state:s, position:f, decoded:i, dropped:i, error:s) |

Slots are 0…3; paths are nonempty, NUL-free and at most 4096 UTF-8 bytes. The
executable path must be absolute. Initial loop is 0/1 and rate is finite 0.1…4.
Controls are play/pause (value 0), loop (0/1), rate (0.1…4) and seek (0…86400
seconds). Image, video and video-control share one accepted sequence/signature
per slot. An exact retry acknowledges without repeating its effect; older or
conflicting sequences fail E_HYDRA_STALE. Failed preparation does not advance
that sequence. Status queries do not mutate ordering.

Preflight/control/status errors use E_HYDRA_VIDEO. Decoding is asynchronous:
accepted media can later fail, reported by video-status, while the retained
picture remains. [VIDEO.md](../docs/hydra/VIDEO.md) defines resampled timestamps,
one-frame pending queue, stable texture uploads, controls/EOS and cleanup. No
native video audio, camera/screen capture, or original-packet PTS is promised.

Step 21 advertises `capture-avfoundation-v1` only on macOS; it indicates adapter
support, not an installed decoder, device availability or permission. Requests are
immediate-only and session/owner scoped:

| Request | Arguments | Reply |
| --- | --- | --- |
| capture-scan | epoch:s, session:i, sequence:i, decoder:s | ack(epoch, session, sequence), after discovery worker acceptance |
| capture-devices | epoch:s, session:i, request:i | capture-devices.reply(epoch, session, request, state:s, error:s, count:i, repeated kind:s/index:i/name:s triples) |
| capture | epoch:s, session:i, sequence:i, slot:i, kind:s, index:i | ack(epoch, session, sequence), after capture worker acceptance |

Decoder path is absolute, nonempty, NUL-free and at most 4096 UTF-8 bytes. Discovery
retains its own sequence/signature: exact retries do not restart it; stale or
conflicting retries fail. Poll capture-devices for loading/ready/error. Up to 16
devices are returned; labels truncate to 96 UTF-8 bytes without splitting characters.
The selected full name is retained internally. Kind is camera/screen and index
0…255 must match the most recent ready enumeration. Nil/automatic selection is a
client convenience selecting the first matching returned kind. No implicit rescan
or other-device fallback is performed on source open.

Capture shares the image/video/video-control per-slot sequence guard and ack
semantics. Preparation/discovery errors use E_HYDRA_CAPTURE. Asynchronous source
failure, timeout and metadata are queried through video-status. That reply now
appends kind:s, inputWidth:i, inputHeight:i, width:i, height:i, fps:f after its
previous error:s field. Existing prefix fields are unchanged; fps is nominal
output rate and input dimensions are startup-reported (zero if unknown/file).
Shared video-control accepts play/pause for capture; unsupported seek/rate/loop
returns E_HYDRA_VIDEO without mutation. [CAPTURE.md](../docs/hydra/CAPTURE.md)
defines explicit permission handling, timing, bounded storage, disposal and the
platform matrix. Microphones/audio, ScreenCaptureKit and Windows/Linux adapters
are not implemented by this feature.

Camera follow-up: on macOS, kind=camera uses the bundled SCShader Camera app as
its own native AVFoundation permission owner; kind=screen/discovery retain
FFmpeg. Status additionally permits state `permission` while its camera prompt
is pending (up to 60 seconds). The installed example waits up to 75 seconds;
ordinary no-frame failure remains ten seconds. Existing fields/ordering do not
change. Timeout errors retain bounded native diagnostics. The local bridge and
helper protocol are internal, not new OSC endpoints. See CAPTURE.md for source
ownership, helper discovery and development build requirements.
