# Hydra staging protocol — implementation checkpoint

2026-11-12. Additive to [SCShader v1](protocol.md). This is the implemented
**staging subset** of [ADR 001](../docs/hydra/ARCHITECTURE.md). The initial compiler
is available; GPU output activation is pending. It
advertises compiler operators separately from active synthesis and emits no
`committed` reply. Supported programs compile/cache a GPU pipeline and return
`E_HYDRA_CAPABILITY`;
existing shader resources and presentation remain intact.

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
| commit | epoch, session, tx | structured error; no activation available |
| abort | epoch, session, tx | aborted(epoch, session, tx) |
| status | epoch, session, request, queriedTx:i | status.reply(epoch, session, request, -1:i, revisions:b, state:s, code:s, message:s) |
| close | epoch, session, request | closed(epoch, session, request) |

IDs/requests/transaction numbers are positive signed int32, output is 0..3,
revision is nonnegative (only 0 currently exists). ABI major must be 1. Epoch and
nonce are nonempty ASCII of at most 64 bytes; port is 1..65535. Open retries with
same sender endpoint, nonce and reply port return the existing handle; a different
owner fails. Epoch/session guard all subsequent requests. Closing releases the
session; retrying a closed handle currently reports `E_HYDRA_SESSION`.

`limits` is exactly five big-endian u32 values, in order: maximum IR bytes
(65536), chunk bytes (2048), nodes (256), depth (64), expanded operations (4096).
`features` is `staging-ir-v1;wgsl-v1:solid,osc,gradient,rotate,scale,color;activation-unavailable`.
This narrower discovery payload supersedes the ADR's provisional full-limit
payload for the staging feature only. Future compiler/source/presentation
capabilities must explicitly document their additional fields before advertising.
Revisions blob is four big-endian int32 zeros, ordered o0..o3; mode -1 is Legacy.
`state` is uploading, failed or unknown. Status does not claim that an evicted
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
again. Commit retry returns its retained failure without repeated validation.
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

The IR layout is HYIR 1.0 from ADR 001. The decoder currently accepts scalar
solid/osc/gradient/rotate/scale/color, plus noise/nested blend for AST validation.
The compiler lowers the first six; noise/blend currently return E_HYDRA_COMPILE.
All parameter declarations and other argument forms/operators fail explicitly.
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

`params`, `present`, `hush`, revision increments and committed activation are
**not implemented**. Pipeline validation/caching is now available as described in
[the compiler checkpoint](../docs/hydra/COMPILER.md). Step 06's full acceptance gate
remains pending activation/presentation in Step 08; this staging
checkpoint must not be used as evidence of rendered Hydra output.
