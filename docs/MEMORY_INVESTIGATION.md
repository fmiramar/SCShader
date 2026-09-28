# Metal multipass memory investigation — 0.0.13

Local evidence from macOS 15.7.2, x86_64, AMD Radeon RX 580. This remains an open
final-release gate, not a claim that long-duration stability has passed.

## Observations

- The first release traffic smoke sent 60,000 updates in 60 seconds with advancing
  frames and no unexpected renderer errors or reported queue drops/rejections.
  RSS rose from 37,712 KiB to 64,032 KiB.
- An otherwise idle 90-second run rose from 41,972 KiB to 73,268 KiB, approximately
  3.4 MiB per ten seconds after startup. This rules out high-rate OSC as a necessary
  trigger, but does not establish that growth continues forever.
- A macOS autorelease-pool-per-frame experiment did not change the trend and was
  removed; it added no permanent dependency.
- With `gpu-counters` enabled, live buffers, textures, bind groups, command
  encoders, and pipelines remained stable between successive 600-frame samples.
- A diagnostic explicit GPU-completion wait reduced outstanding resource counts
  but did not remove the RSS trend: 48,152 to 69,144 KiB over 60 seconds in a debug
  build. This wait is not enabled in ordinary packages.
- The macOS leak checker reported no unreachable leaked blocks. That does not
  establish bounded memory: reachable caches/retained allocations can still grow.
- The original release binary's real one-hour traffic attempt stopped after
  410.03 seconds at 185,124 KiB RSS, exceeding the 53,088 KiB warmup baseline by
  128.9 MiB. It sent 410,027 updates and received 410 pongs/status replies, with
  frames advancing from 6 to 24,562. The harness correctly failed and stopped its
  owned child; this is not one-hour acceptance.

Allocation stack logging identified repeated live allocations below
`BronzeMtlCmdBuffer renderCommandEncoderWithDescriptor`,
`amdMtl_CmdBuffer_GetRsrcMgr`, `amdMtlBronzeResourceMgrAlloc`, and
`amdMtlBronzeAllocThreadGroupLengthMgr`. The stack sample contained about one set
per rendered frame. Its profiler pause correctly failed the strict harness's
five-second reply deadline, so that particular run is diagnostic evidence only.

## Reduced reproduction and workaround

The original native `tools/metal_memory_probe.m` and Rust
`renderer/examples/memory_probe.rs` comparisons each rendered 20,001 offscreen
frames with bound uniforms and per-frame completion waits:

| Case | Initial → final RSS (KiB) |
| --- | --- |
| Native Metal, one pass | 4,728 → 6,028 |
| Native Metal, two passes in one command buffer | 4,876 → 6,932 |
| Native Metal, two passes with labels, shared-event signal and completion callback | 4,692 → 6,608 |
| Native Metal, split command buffers with those signals/callbacks | 4,800 → 6,796 |
| wgpu, two passes in one submission | 13,328 → 126,088 |
| wgpu, one pass | 12,936 → 14,308 |
| wgpu, separate pass submissions plus per-frame uploads/new bind groups | 13,240 → 15,764 |

The minimal wgpu reproduction does not open a window or use OSC. This narrows the
necessary trigger to the tested multipass submission path, rather than blaming
window presentation, uploads, or all Metal applications. It does not prove a
specific upstream ownership bug or that other devices behave identically.

SCShader now submits each intermediate render pass separately on **Metal only**.
Feedback and presentation still run the same shader with the same input; graph
passes retain their order on the same queue. There are no new CPU waits, process
restarts, or relaxed memory thresholds. Other backends retain their single frame
submission. Feedback-target initialization also uses individual clear submissions
to cover repeated resize. Extra submission overhead is a measured-workaround
tradeoff, not a claim to fix wgpu internally.

`pass_order_check` independently reads GPU pixels after 40 feedback frames, each
with a graph pass and duplicated terminal feedback/presentation passes. Both
batched and split submissions returned the exact expected pixels. This verifies
the submission-order strategy, not physical display scan-out.

The first post-workaround traffic attempt failed at 70.34 seconds on one reported
incoming-queue drop while other SC/GPU tests were running concurrently. RSS was
41,804 → 43,584 KiB, but the run remains a failure. The isolated one-hour retry
passed in 3600.00 seconds: 3,600,000 updates, 215,984 advancing frames, no queue
drops, and 42,360 → 56,256 KiB RSS (12.3 MiB growth from its 30-second baseline).
Keep the overlapping run as negative overload evidence; do not attribute its drop
conclusively to overlap or ignore it in future incoming-flood testing.

The superficially similar [upstream report](https://github.com/gfx-rs/wgpu/issues/8768)
was traced to a separate [wgpu-native FFI cleanup issue](https://github.com/gfx-rs/wgpu-native/issues/541).
SCShader uses the Rust API, not that FFI layer; this is **not evidence of the same
root cause**. The published
[wgpu 30.0.1 changes](https://github.com/gfx-rs/wgpu/blob/v30.0.1/CHANGELOG.md#v3001-2026-08-21)
do not identify a fix for this allocation path; no speculative dependency upgrade
or patched Cargo registry was used to hide the result.

## Independent correction

Per-frame uniform uploads now happen only after acquiring a presentable surface
texture. Timeout/occluded/outdated paths flush pending data uploads without
staging another frame's uniforms. This prevents unsubmitted-upload accumulation
on skipped frames, but is not presented as a fix for the visible-frame RSS trend.

## Remaining acceptance and removal criteria

Complete the exact candidate's isolated one-hour gate with unchanged bounds,
plus short multipass/resource-churn checks and native-platform verification.
The user will perform an eight-hour test on the completed final candidate as the
last release step; the previous four-by-eight-hour matrix is no longer a current
development requirement. Keep
the pre-workaround failed evidence and measure submission overhead separately.
Remove the Metal split only after the two-pass reproduction stays bounded with
the proposed dependency/backend change and pixel/soak checks pass again. Do not
raise thresholds or add disruptive periodic renderer restarts as a substitute for
fixing or honestly bounding growth. See [STABILITY.md](STABILITY.md).
