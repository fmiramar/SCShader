# Hydra output protocol — implementation checkpoint

2026-11-13. Additive to [SCShader v1](protocol.md) and implementing the first
output/temporal portion of [ADR 001](../docs/hydra/ARCHITECTURE.md). 50 procedural/composition/modulation operators plus output-only `src` and native `prev`,
four paired output histories and presentation are available. Media sources remain unavailable. Step 15 adds the bounded numeric API below.

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
`features` is `staging-ir-v1;wgsl-v1:<comma-separated supported registry names>;output-o0-v1;outputs-temporal-v1;numeric-v1;clock-v1`. The 52 names and their defaults/types are in `renderer/src/hydra/registry.rs`; discovery order is not an ABI guarantee.
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
