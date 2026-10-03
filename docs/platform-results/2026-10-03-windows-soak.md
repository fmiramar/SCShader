# Windows dual-GPU soak results and investigation — 2026-10-03

Status: both user-requested GPU runs finished. The RX 580 completed eight hours;
the RTX 3060 failed early on an incoming OSC queue drop. The Windows long-run gate
remains open because the RTX failure cause is unresolved.
Source baseline: `579a246536e00ca09f90c63b5ff32ecc41b00f97`, plus this checkpoint's
timing instrumentation and traffic-sender changes described below. Version 0.0.18,
Windows x64 MSVC, NVIDIA GeForce RTX 3060 / D3D12, driver 32.0.15.9636, and
Radeon RX 580 Series / D3D12, driver 30.0.13023.4001.
The installed extension has not been replaced by these diagnostic builds.

## Preserved failures

All three October 2 attempts requested 28,800 seconds of traffic at 1,000
uniform updates/second against the unchanged packaged renderer SHA-256
`b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff`.

| Local evidence directory under `build/platform-tests/` | Measured seconds | Outcome |
| --- | ---: | --- |
| `2026-10-02-rtx3060-eight-hour` | 1144.529 | E_QUEUE_FULL; shutdown reports 3 dropped updates |
| `2026-10-02-rtx3060-eight-hour-retry-01` | 952.906 | E_QUEUE_FULL; shutdown reports 2 dropped updates |
| `2026-10-02-rtx3060-eight-hour-retry-02` | 894.918 | E_QUEUE_FULL; shutdown reports 3 dropped updates |

The harness rejects the first diagnostic and then closes its owned renderer.
Window closure therefore does not establish a renderer crash. All runs reported
the requested NVIDIA adapter, normal sampled frame progress and bounded RSS
before failure. Ten-second sampling does not exclude a brief stall between samples.
The third attempt capped sender catch-up bursts at 16 and skipped one stale tick;
it still failed, so that adjustment did not resolve the underlying issue.

## Instrumentation and short reproduction

`SCSHADER_TRACE_TIMING=1` enables optional stderr timing scopes for command
draining, event callbacks, hot reload, frame acquisition, submission, presentation
and GPU polling. The OSC receive thread can snapshot the main thread's current
phase and time since the last consumed command while it is stalled. Slow phase
logs are bounded by first-four/power-of-two sampling. Windows tracing records the
last dispatched native message ID and only system-command parameters, never keys
or input text. Protocol fields, queue bounds and diagnostic failure checks remain
unchanged. These are wall-clock timings, not GPU execution measurements.

The soak harness also records sender lag and, when tracing on Windows, a
`traffic.power.jsonl` file with native display-power notifications. Observation
does not modify power settings. This uses Microsoft's documented
[callback registration](https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powersettingregisternotification)
and [session display status](https://learn.microsoft.com/en-us/windows/win32/power/power-setting-guids).

The first instrumented short traffic run failed after 1.660 seconds:
`2026-10-03-rtx3060-timing-smoke`. Renderer SHA-256:
`33fe207cef9c6ae26721421c543e8a70dae8bec5bb4cb3c413bbd6896c6a51ea`.
At overflow the main thread was between instrumented callbacks for 240 ms and
had not consumed a command for 256 ms. Sender lag peaked at 3.035 ms, its maximum
burst was four, and the display notification reported on. The user does not
remember whether the window was being moved; this capture alone does not identify
which Windows operation caused the gap.

The next diagnostic build adds scopes around the complete window/event callbacks
and records the native message ID. SHA-256:
`8f1d67fefb8d34f22a7d9896512e0d5f8db8f237fd9387823436d8e5e5e04030`.
All four ten-second modes pass under `2026-10-03-rtx3060-timing-smoke-02`.
The explicitly scheduled 1,800-second diagnostic under
`2026-10-03-rtx3060-timing-30m` was stopped at the user's request. Its last flushed
CSV sample is 230.015 seconds: 230,015 sent updates, 13,741 frames, no reported
drops and RSS 184,260 KiB. The display notification remained on. Interruption
terminated the owned process tree before the child wrote final JSON; the parent
record was explicitly marked interrupted after confirming those processes exited.
The CSV/logs are partial evidence, not a completed 30-minute pass.

At the start of the diagnostic investigation, Windows' AC display-off timeout
was 900 seconds and AC system sleep was disabled.
No process requested display/system wakefulness when this run began. Display
power is a plausible trigger for the older 15–19 minute failures, but the short
on-display reproduction shows that other event-loop pauses also need consideration.
The agent did not change power settings; the user's later adjustment is recorded below.

Pre-commit checks: Rust formatting and strict all-feature Clippy pass; 61 Rust tests
(59 application plus two example tests) and all 132 Python tests pass, including a
native display-notification registration/cleanup check. No test hooks or changed
dependencies are used in the diagnostic release binaries. Long-duration acceptance
of the changed renderer, the RTX investigation, other native platforms and final
release sign-off remain open. No additional long run was started during review.

## Resume after the user's model switch

The user stopped the October 3 diagnostic to switch to a lighter model and set
Windows display-off to Never. They then explicitly requested eight hours on each
GPU, with the physical monitors switched off. Before starting, the AC display-off
and AC sleep settings both read Never; DC settings were left unchanged. No power
setting was changed by the agent. The two sequential runs use the unchanged
packaged production renderer, each for 28,800 seconds of traffic at 1,000 updates
per second, D3D12, high-performance selection and an explicitly named adapter.

RTX 3060 evidence: `build/platform-tests/2026-10-03-rtx3060-eight-hour-screen-never/`.
It selected NVIDIA GeForce RTX 3060 / DX12 with renderer SHA-256
`b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff`, then failed
after 107.355 seconds (107,355 updates, frame counter 4 to 6,411, 108 pong/status
samples, peak RSS 186,616 KiB) on `E_QUEUE_FULL`. Sender lag peaked at 3.986 ms,
the maximum burst was four and no updates were skipped. The renderer's shutdown
log reports three dropped updates. It did not qualify as an eight-hour check;
the harness closed its owned render window after the failure.

RX 580 evidence: `build/platform-tests/2026-10-03-rx580-eight-hour-screen-never/`.
It selected Radeon RX 580 Series / DX12 with that same renderer SHA-256 and passed
28,800.002 seconds, sending 28,800,000 updates; its frame counter advanced from
7 to 1,726,258, with 28,777 pong/status samples. There were no queue-drop
diagnostics or skipped updates. Sender lag peaked at 7.318 ms and the maximum
burst was eight. RSS was 174,860 KiB at startup/peak and 165,792 KiB at completion.
This qualifies the exact packaged renderer on this adapter for the traffic-only
eight-hour test, not the four-mode suite or the changed diagnostic renderer.

The RX pass with AC display-off and sleep set to Never shows those settings can
coexist with a complete test on this adapter. It does not isolate why the RTX run
filled its queue; physical monitor power state was not recorded as telemetry. Each
run has its own `run.json`, CSV and renderer/console logs under the paths above.

The instrumented executable is saved at
`build/diagnostic-candidates/2026-10-03-timing-02/scshader-renderer.exe`.
Run with `SCSHADER_TRACE_TIMING=1` to preserve phase and display-power evidence;
the harness still fails on drops and requires the original rate and health checks.
Use a fresh evidence directory. Keep the distinction between this diagnostic
binary and the unchanged packaged production renderer when reporting acceptance.

## Review corrections and commit scope

Review found that a fixed 16-update catch-up cap would artificially under-drive
the harness's advertised 50,000 Hz maximum with its one-millisecond polling loop.
The cap now scales with a 16 ms traffic window: still 16 updates at 1,000 Hz,
but 800 at 50,000 Hz, with at least one update at lower rates. Regression tests
cover steady high-rate scheduling, bounded catch-up and low-rate progress.
Skipped ticks remain reported, the 99% requested-rate threshold is unchanged,
and any queue-drop diagnostic still fails the run. This is a test-sender correction,
not an RTX stall fix; it does not change the recorded historical evidence.

The development checkpoint includes reviewed tracing code, harness changes,
regression tests and this result summary. Raw JSON/CSV/logs and executable builds
remain preserved locally under ignored `build/` and `stage/`, outside the commit.
No installed extension, dependencies, version metadata or release assets changed.
