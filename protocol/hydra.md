# Hydra single-output protocol — implementation checkpoint

2026-11-13. Additive to [SCShader v1](protocol.md) and implementing the first
single-output portion of [ADR 001](../docs/hydra/ARCHITECTURE.md). 39 compiled
operators and `o0` activation/presentation are available. Other output/source and
parameter features remain explicitly unavailable.

All suffixes below use `/scshader/v1/hydra/`. Fields use ordinary OSC types:
`i` = int32, `s` = string, `b` = blob. All requests must be immediate, including
when nested in bundles. Reject timed structural packets before scheduling; a
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
| present | epoch, session, sequence:i, mode:i (-1 Legacy, 0 o0) | presented(epoch, session, sequence, mode) |
| close | epoch, session, request | closed(epoch, session, request) |

IDs/requests/transaction numbers are positive signed int32, output is 0..3,
revision is nonnegative, initially 0, and increases once per accepted commit. ABI major must be 1. Epoch and
nonce are nonempty ASCII of at most 64 bytes; port is 1..65535. Open retries with
same sender endpoint, nonce and reply port return the existing handle; a different
owner fails. Epoch/session guard all subsequent requests. Closing releases the
session; retrying a closed handle currently reports `E_HYDRA_SESSION`.

`limits` is exactly five big-endian u32 values, in order: maximum IR bytes
(65536), chunk bytes (2048), nodes (256), depth (64), expanded operations (4096).
`features` is `staging-ir-v1;wgsl-v1:<comma-separated supported registry names>;output-o0-v1`. The 39 names and their defaults/types are in `renderer/src/hydra/registry.rs`; their order follows the pinned inventory.
This narrower discovery payload supersedes the ADR's provisional full-limit
payload for the staging feature only. Future compiler/source/presentation
capabilities must explicitly document their additional fields before advertising.
Revisions blob is four big-endian int32 values ordered o0..o3 (only o0 can be
nonzero). Mode -1 is Legacy and 0 is o0. `state` is uploading, committed, failed
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

The IR layout is HYIR 1.0 from ADR 001. The decoder accepts 32 procedural and seven combine
operations with scalar arguments (tag 0,
f32) and the literal vec4 argument for `sum` (tag 5, four f32 components),
encoded in network order. Combine arguments begin with an expression reference (tag 2, earlier node ID
as u32). All seven combine operations compile, including nested branches. Parameter declarations,
source sampling and other argument forms/operators fail explicitly.
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

`params`, `hush`, o1..o3, source sampling and feedback remain unimplemented.
See [first output semantics and limitations](../docs/hydra/OUTPUT.md). The SC
client maintains accepted revisions and reconciles a timed-out commit through
status. `present` is immediate-only at this stage. A new output is only made
visible after explicit presentation selection; an accepted replacement of selected
o0 becomes visible on its next frame. Close releases the output and selects Legacy.
