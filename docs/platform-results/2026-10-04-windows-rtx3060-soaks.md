# Windows RTX 3060 one-hour and eight-hour traffic results — 2026-10-04

The exact 0.0.18 renderer passed a one-hour D3D12 traffic soak on the RTX 3060.
The following eight-hour attempt failed after 2,482.785 seconds when the renderer
reported `E_QUEUE_FULL` and dropped one continuous update. It does not qualify as
an eight-hour pass. The failure reproduces the open desktop queue-service issue;
the cause has not been fixed.

## Candidate and conditions

- Source: clean `main` at `f36e4547b7854775b2b3ccd6e26c8207a7cab3e7`, version 0.0.18.
- Release renderer SHA-256:
  `cc78217490017d3175eb97083efde56a6c3726fc86257a1154c03d2a19a5ae34`.
- `Cargo.lock` SHA-256:
  `99477baa12a417c9c2aee715b1bc27cb59bb40035f7b4855480ef07e5fe762f7`.
- Windows 11 Pro x64, build 26300; Rust 1.97.1 (`x86_64-pc-windows-msvc`),
  Python 3.11.9, MSVC build tools; native AMD64 execution.
- NVIDIA GeForce RTX 3060, driver 32.0.16.1088, D3D12, high-performance
  selection. AC display-off and sleep timers were set to Never.
- Traffic mode only, overlay disabled, 1,000 updates/second, using the standard
  acceptance runner and its queue-drop and memory checks. The sender used
  `perf_counter()` backed by QueryPerformanceCounter.
- The serial runs started 2026-10-04 21:43 local time (America/Sao_Paulo).
  The one-hour run finished at 22:43; the failed eight-hour attempt ended at
  23:25.

## Results

| Run | Measured duration | Updates sent | Sender-skipped ticks | Max sender lag | Frames | RSS initial / peak / final | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| One hour | 3600.001 s | 3,599,919 | 81 | 38.211 ms | 4 → 215,772 | 181,460 / 189,912 / 119,184 KiB | Pass |
| Eight hours requested | 2482.785 s | 2,482,610 | 176 | 83.276 ms | 4 → 148,770 | 181,104 / 187,628 / 183,276 KiB | Fail: `E_QUEUE_FULL` |

The one-hour manifest reports `passed: true` and
`qualifies_as_one_hour_check: true`; it is not an eight-hour qualification. The
eight-hour manifest reports `passed: false` and both qualification flags false.
Its failure reason is `bounded OSC queue full; dropped 1 continuous update(s)`.
The sender-lag guard did not trigger. Memory remained under the configured limits
in both runs. A single queue-drop diagnostic still fails the acceptance gate.
The runner stopped on the first diagnostic; during shutdown, the renderer log
recorded another queue-full warning and reported three total dropped updates.
The final ten-second CSV sample before failure (at 2480.202 seconds) showed zero
sampled drops, so the trigger occurred between samples.

This run had timing tracing disabled and did not record the physical monitor,
window placement, or display-adapter routing. The power timeouts rule out normal
idle sleep/display shutdown as configured causes, but they do not explain the
queue-service pause. A prior traced reproduction observed a 256 ms command-service
gap without identifying the Windows operation behind it; see the
[timing investigation](2026-10-03-windows-soak-comparison.md).

The eight-hour attempt is already the requested eight-hour test. It stopped after
about 41 minutes; do not count it as complete or repeat it on this unchanged
binary. Earlier RTX 3060 runs also failed on `E_QUEUE_FULL`, including the
107-second run documented in the [prior desktop investigation](2026-10-03-windows-soak.md).
This result reconfirms the symptom but does not identify a root cause or establish
that the one-hour pass predicts eight-hour stability.

## Other checks in this session

- `cargo fmt --check`, strict all-target/all-feature Clippy, and locked Rust tests
  passed (59 renderer tests and 2 overlay example tests).
- Python unittest discovery completed successfully: 132 discovered, 131 passed,
  one skipped. The captured transcript preserves the command exit code but not
  the skip reason.
- The locked release build passed and produced the renderer hash above.
- All four ten-second D3D12 modes (traffic, trivial, reload, feedback) passed on
  both the RTX 3060 and Radeon RX 580.
- Scheduling stress, runtime stress (including 80 resource cycles and protocol,
  command, and continuous-traffic floods), and feature-gated logical-device
  recovery passed. Logical recovery does not test a physical GPU reset.
- Installed SuperCollider class/help checks and manual listening/display checks
  were not part of this run sequence. Prior Windows desktop results remain
  separately recorded.

## Handoff

The desktop eight-hour gate remains **incomplete**. Investigate and fix the
renderer queue-service failure, add regression coverage, and rerun short checks
before scheduling another long soak. The development 0.0.18 candidate is not a
final release candidate. This traffic-only result makes no claim about other
modes, GPUs, drivers, or physical device recovery.

Raw evidence is retained locally under ignored `build/` paths:
`build/preflight-20261004-212339/`,
`build/soak/2026-10-04-nvidia-rtx3060-one-hour/`, and
`build/soak/2026-10-04-nvidia-rtx3060-eight-hour/`. No raw logs or binaries are
included in the source commit.

## Follow-up traced display-routing diagnostics — 2026-10-05

These follow-ups used the same 0.0.18 renderer SHA above, explicitly selected
`--adapter NVIDIA`, with DX12, 1,000 updates/second, timing tracing enabled,
high-performance power preference, and AC display/sleep timeouts disabled.
They do not change the failed eight-hour outcome or identify the queue-service
root cause.

### Display routing and short controls

Read-only Windows monitor enumeration during this session found the current
topology differs from the earlier desktop record: `\\.\DISPLAY1` is the left
RTX 3060 output at `[-1920, 0, 0, 1080]`; `\\.\DISPLAY5` is the center primary
Radeon RX 580 output at `[0, 0, 1920, 1080]`; `\\.\DISPLAY6` is the right RX 580
output at `[1920, 0, 3840, 1080]`. The first assumed right-side test position
(`1984,64`) was on `DISPLAY6`, not an RTX-driven display; its results are
therefore recorded as RX-output routing below.

All 60-second traced route controls passed. Both renderer adapters (RTX 3060 and
RX 580) ran on the left RTX output, and both ran on each of the two RX-driven
outputs. Runs sent 59,999–60,000 updates, had maximum sender lag of 2.81–3.76 ms,
and showed no runtime slow-phase or queue-full trace. Native window snapshots
confirmed the actual monitor for every route. These short controls show that
both render adapters can present on each tested output; they are not long-run
qualification.

### Thirty-minute traced traffic

| Renderer adapter | Display output | Duration | Updates | Skipped | Max sender lag | Peak RSS | Outcome |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| RTX 3060 | RX 580 primary (`DISPLAY5`) | 1800.001 s | 1,800,000 | 0 | 8.524 ms | 184,760 KiB | Pass |
| RTX 3060 | RTX 3060 left (`DISPLAY1`) | 1800.002 s | 1,799,944 | 56 | 39.790 ms | 164,384 KiB | Pass |

Both logs recorded the NVIDIA adapter selected by the renderer, no runtime
`slow=` timing events after initialization, and no queue-full diagnostics. The
first row's window was placed at `(64,64)` on the RX-driven primary; the second
was placed at `(-1856,64)` on the RTX-driven left monitor.

### Interrupted 45-minute traced diagnostic

The next RTX 3060/DX12 run targeted the prior eight-hour failure window and was
requested for 2,700 seconds on the RX-driven primary. It stopped after
830.779 seconds (13 minutes 50.8 seconds) because the Python traffic sender
exceeded its 250 ms pacing guard: maximum sender lag was 704.582 ms. The run sent
829,993 updates, skipped 81 ticks, and reached 187,552 KiB peak RSS. The
acceptance result is **failed/incomplete**; it is not a 45-minute pass.

The renderer log confirms `NVIDIA GeForce RTX 3060` on DX12 and records only a
491.116 ms initialization phase above the 50 ms trace threshold. It contains no
runtime slow-phase or `E_QUEUE_FULL` record. The final sampled queue/drop fields
were zero. Thus this attempt did not reproduce the renderer's queue-full fault;
it failed independently because the test sender itself paused. The reason for
that sender pause is unknown. No renderer process remained after the harness
closed the run.

During testing, Task Manager briefly attributed SCShader activity to GPU-0
(NVIDIA) and GPU-1 (AMD). This is not evidence that the selected renderer adapter
changed: the run manifest and renderer startup record both identify the RTX 3060.
Task Manager reports per-process GPU engine activity and its GPU numbering;
see [Microsoft's Task Manager GPU overview](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/).
The exact cause of the AMD activity on this two-adapter desktop was not captured.

These results leave the intermittent RTX 3060 `E_QUEUE_FULL` issue unresolved.
They do not qualify the unchanged renderer for another eight-hour attempt.
Detailed raw logs, CSV/JSON manifests, and local monitor snapshots remain under
ignored `build/platform-tests/2026-10-05-*` paths and are not included in the
source commit.

## 2026-10-05 WPT/Nsight capture diagnostic

The deterministic window-position option was added to the acceptance harness so
these runs could request `(-1856, 64)`, inside the left RTX-driven display's
bounds (`DISPLAY1`, `[-1920, 0, 0, 1080]`). The candidate remained the unchanged
0.0.18 renderer SHA-256 above. Python unittest discovery passed all 134 tests
after the harness change.

WPR 10.0.26100, WPA, and GPUView were available. Custom WPR profiles are
supported by Microsoft's [recording profile documentation](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/recording-profiles);
the profile used the documented [EventProvider process filter](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/eventprovider).
The smaller profile enabled Microsoft-Windows-DxgKrnl keyword mask `0x08018000`
(GPU scheduling, DxgKrnl, and present events), level 5, without stacks or CPU
context switches. A smoke trace recorded 224,351 DxgKrnl rows across 22 event
types and contained present-queue, GPU scheduling, and VSync events. The dump
still contained events from many process IDs; 12,992 rows had the smoke's
renderer PID. This profile provides system-wide WDDM context and did not
isolate every event to the renderer.

Broad profiles were rejected before the long run because their measured
footprints were excessive: Nsight's full WDDM profile produced a 191 MB report
and 2.39 GB SQLite export per minute; its light WDDM profile produced a 99 MB
report and 1.52 GB SQLite export per minute. WPR CPU.Light plus GPU.Light
produced a 1.15 GB ETL per minute, and a CPU context-switch plus DxgKrnl profile
produced 500 MiB per minute. A first process-tree smoke also produced 2.64
million scheduler rows in 60 seconds. Its wrapper exited before stopping WPR
after Nsight emitted a non-fatal no-NVTX notice; WPR was stopped explicitly
afterward with zero lost events, but that trace was not used for size
qualification.

The revised 60-second smoke passed on the RTX 3060 at the requested position:
60,000 updates, zero skipped updates, 2.063 ms maximum sender lag, and no ETW
loss. Nsight exported 113,233 D3D12 API rows, 29,205 GPU workload rows, and
21,910 memory-operation rows. The WPR ETL was 35,651,584 bytes for 67.1 seconds;
the Nsight report was 5,594,799 bytes and its SQLite export was 15,564,800 bytes
for the 60-second profile. This made the capture size practical, but it did not
show that Nsight's in-process instrumentation would preserve the renderer's RSS
guard over a long run.

The requested 2,700-second combined WPR/Nsight run stopped at 900.082 seconds
because RSS growth exceeded the 128 MiB guard: baseline 201,668 KiB, peak
333,676 KiB, reported growth 128.9 MiB. It sent 900,071 updates, skipped 11,
reached a 16-update burst, and had 26.078 ms maximum sender lag. The renderer
log recorded no `E_QUEUE_FULL`; the sampled queue/drop counters remained zero.
The output contains an 85,701,750-byte Nsight report, a 235,036,672-byte SQLite
export, and a 355,467,264-byte WPR ETL. The ETL reports zero lost buffers and
events. The Nsight export contains 1,675,571 D3D12 API rows, 432,389 workload
rows, and 324,298 memory-operation rows. CPU context-switch tracing was disabled
because the process-tree profile generated an impractical scheduler stream.
Nsight's command returned 1 because the acceptance child failed its RSS guard;
the Nsight report was generated and WPR stopped successfully.

A matched 900-second WPR-only control used the same binary, adapter, position,
timing trace, and acceptance limits, without Nsight. It passed with 900,000
updates, zero skipped updates, 2.508 ms maximum sender lag, and RSS baseline /
peak / final of 159,400 / 159,416 / 157,240 KiB. Its 328,204,288-byte ETL also
had zero lost buffers and events. The contrast strongly implicates Nsight's
in-process instrumentation in the captured run's RSS growth; it does not prove
that the renderer has no memory issue under other conditions. Neither 15-minute
run qualifies as a one-hour check, and neither reproduces the earlier
`E_QUEUE_FULL` failure. The combined capture was not usable for a 45-minute
acceptance pass with the current RSS guard.

The combined run's timing log recorded the renderer losing focus at 260.584
seconds; frame production continued. The harness closed its owned renderer
window when the RSS guard stopped the test, rather than after a renderer crash.
All raw traces, exports, and logs remain under ignored `build/platform-tests/2026-10-05-*` paths.

## 2026-10-05 sender checkpoints and WPR scheduler capture

The acceptance harness now writes one-time sender-lag checkpoint records at
50, 100, 150, 200, and 250 ms. Each record contains a UTC timestamp, monotonic
elapsed time, observed lag, and guard limit; thresholds crossed between polling
iterations share the first observation time. The mode JSON now includes the
traffic sender PID, and the acceptance validator requires it, so future WPR
process reports can distinguish the sender from the acceptance supervisor.

`tools/wpt/SCShaderScheduler.wprp` combines kernel `CSwitch`, `CompactCSwitch`,
`ProcessThread`, and `ReadyThread` events with the existing DxgKrnl
GPU-scheduling/present keyword mask `0x08018000`. It records no call stacks and
does not use Nsight. Kernel scheduling and WDDM events are system-wide; the
profile does not isolate all events to SCShader. The timing summary can be
correlated by UTC, and xperf's context-switch process report includes the
renderer and Python processes.

A final 60-second profile smoke passed on the exact renderer hash above, NVIDIA
RTX 3060, D3D12, and the Radeon-driven primary at `(64,64)`: 60,000 updates, no
skips, and 2.080 ms maximum sender lag. The ETL was 58,720,256 bytes for 61.37
seconds with zero lost buffers/events. The sender PID was present in both the
console marker and JSON, and xperf listed it alongside the renderer. xperf also
emitted an `Invalid Event` decode warning for the ETW
`EventTraceConfigGuid` system-configuration record; its context-switch report
completed, and WPR reported no lost data. The GUID is identified in
[Microsoft's NT Kernel Logger constants](https://learn.microsoft.com/en-us/windows/win32/etw/nt-kernel-logger-constants).

A 15-minute WPR-only run on the same route passed 900.001 seconds with 900,000
updates, no skipped ticks, 5.467 ms maximum sender lag, and no queue/drop
counters. It produced a 1,939,865,600-byte ETL with zero lost buffers/events.
This covered the earlier sender-pause interval near 831 seconds without
reproducing it.

The follow-up 45-minute WPR-only run used the same 0.0.18 renderer, NVIDIA
RTX 3060, D3D12, high-performance preference, Radeon-driven primary position
`(64,64)`, 1 kHz traffic, and renderer timing trace. It passed 2,700.001
seconds with 2,700,000 updates, zero skipped ticks, maximum sender lag 6.706 ms,
and no sender checkpoint. RSS baseline/peak/final was 184,436/184,960/182,284
KiB. The renderer timing log contains only the 441.041 ms initialization phase
above its 50 ms threshold; it records no runtime slow phase or `E_QUEUE_FULL`.
The 4,105,175,040-byte ETL spans 45:02 and reports zero lost buffers/events.
xperf's process report includes the renderer and two `python.exe` processes;
this run predates the sender-PID field, so it does not distinguish the sender
from its supervisor. The updated 60-second smoke verifies that PID evidence is
now emitted for future captures.

Neither WPR-only run reproduces the queue-full or sender-pause failures. The
45-minute diagnostic covers the historic 2,482.785-second queue-failure time,
but does not qualify the unchanged renderer for one hour or eight hours and
does not identify a root cause. The profile footprint was about 87 MiB/minute
on the 45-minute run; keep raw ETLs and exports in ignored `build/` evidence.

### One-hour WPR scheduler validation on the RTX-driven display — 2026-10-05

The exact renderer then passed a bounded one-hour traffic check while the
compact WPR scheduler profile recorded the run. The renderer window was placed
at `(-1856,64)` on the left RTX-driven `DISPLAY1`. This qualifies the one-hour
traffic gate for this exact binary and route, but it does not qualify the
eight-hour release gate or prove a fix for the intermittent queue-service
failure.

- Duration: 3,600.000 seconds; 3,599,999 updates; zero skipped updates.
- Renderer: NVIDIA GeForce RTX 3060, D3D12, high-performance preference,
  SHA-256 `cc78217490017d3175eb97083efde56a6c3726fc86257a1154c03d2a19a5ae34`.
- Sender PID: 10628; maximum sender lag 2.420 ms; maximum burst 3.
- Renderer PID: 17328; RSS baseline/peak/final 158,468/159,180/154,116 KiB.
- No `E_QUEUE_FULL` or queue-drop diagnostic was recorded. The only renderer
  protocol message was the expected malformed-packet check.
- WPR captured 1:00:02.403 with zero lost buffers and zero lost events; raw
  evidence is under `build/platform-tests/2026-10-05-scheduler-1h-rtx3060-left/`.

The acceptance runner's timing-trace flag was not enabled for this run, so the
renderer timing log does not add phase-level evidence. The WPR scheduler trace
still provides system-wide CSwitch/ReadyThread and DxgKrnl events. This pass
strengthens route-specific stability evidence without identifying the cause of
the earlier `E_QUEUE_FULL` event.

## Current diagnosis and next steps

The queue is bounded at 256 commands. At the test rate of 1,000 continuous
updates/second, a full queue represents about 256 ms of unconsumed traffic, with
slightly less room for other commands. An earlier timing reproduction captured
about 240 ms between renderer callbacks and 256 ms since the last consumed
command while sender lag was only 3.035 ms. The October 4 eight-hour queue-full
failure also had only 83.276 ms maximum sender lag, below the harness's 250 ms
failure guard. Together these point to a **consumer-side service gap** as the
proximate mechanism; they do not explain what blocked or delayed the renderer's
event-loop thread.

The earlier queue-full trace captured phase `event_loop` about 240 ms between
instrumented callbacks, rather than an instrumented GPU acquisition, submit, or
present phase. That makes a Windows/winit event-loop or message-dispatch delay,
or OS thread-scheduling stall, the leading underlying suspects. A D3D12/driver
presentation wait remains possible but was not observed in that trace; Microsoft's
[DXGI Present documentation](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-present)
notes that Present can wait on a message-pump thread in some configurations.
These are hypotheses. Thirty-minute runs pass on both the RTX's own output and
an RX-driven output, so the current evidence does not show a reproducible
display-route dependency. Task Manager's GPU-0/GPU-1 activity change does not
demonstrate a renderer adapter switch. No physical GPU fault, device reset, or
adapter fallback was recorded. The October 5 RX-primary 45-minute attempt
stopped on the sender-lag guard. The later RTX-left combined WPR/Nsight capture
stopped on the renderer RSS guard after 15 minutes, while a matched WPR-only
control passed 15 minutes with stable RSS. This contrast points to Nsight
instrumentation overhead in that run, but does not prove the renderer has no
memory issue under other conditions. Neither follow-up reproduces the October 4
queue-full event. GPU-memory telemetry was not collected.

The queue-service cause remains open. Nsight stays out of longer diagnostics
because its in-process instrumentation previously hit the RSS guard. The compact
WPR scheduler profile passed a 45-minute run with no losses, but the intermittent
queue failure did not recur. Preserve the strict guards and do not treat that
non-reproduction as a fix:

1. Keep the sender UTC checkpoints, `sender_pid`, renderer timing trace, and
   `tools/wpt/SCShaderScheduler.wprp` together for any future reproduction. If a
   sender checkpoint or queue-full diagnostic occurs, use its UTC/elapsed time
   and PID to focus WPA on the sender and renderer threads. The xperf
   `EventTraceConfigGuid` decode warning remains a tooling limitation despite
   zero WPR lost buffers/events.
2. Change only one condition per follow-up. Existing 30-minute route comparisons
   and the new 45-minute primary-display diagnostic do not show a deterministic
   route dependency or reproduce the queue failure. Use a captured failure to
   choose the next route/backend comparison rather than changing queue capacity
   or relaxing acceptance guards.
3. The one-hour RTX-left validation now passes for this exact binary and route.
   After a root cause is identified and a fix has regression coverage, leave
   the eight-hour soak as the user's final release gate.

Increasing the queue size or relaxing the sender/queue acceptance criteria would
hide symptoms without identifying the stall, so neither is a useful diagnostic
change.
