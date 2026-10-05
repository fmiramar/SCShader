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

If RTX 3060 qualification remains important, the next useful work is to isolate
the profiler-associated RSS growth and find a lower-overhead capture, not to
repeat the same instrumented 45-minute attempt or unchanged eight-hour soak:

1. Preserve the current strict failure guards and `--adapter NVIDIA` selection.
   Add a timestamped record when sender lag approaches its 250 ms limit so a
   harness pause can be separated from a renderer queue-service pause.
2. Use Microsoft's Windows Performance Toolkit (WPT) for a cross-vendor ETW
   capture: WPR records CPU scheduling and graphics events, and WPA/GPUView
   analyzes them. At the earlier check, `wpr.exe` was available and exposed
   `GPU` and `DesktopComposition` profiles; WPA and GPUView were not found in
   that workspace. They were found in the 2026-10-05 Windows session. A custom
   WPR profile now records DxgKrnl present/GPU-scheduling events with zero ETW
   loss. Its dump still contains many system/DWM process IDs, so treat it as
   system-wide WDDM context. The combined Nsight D3D12/WPR run hit the renderer
   RSS guard at 15 minutes; the matched WPR-only control passed 15 minutes.
   Do not repeat the same instrumented 45-minute run until its RSS overhead is
   addressed. A CPU-scheduling profile needs a practical-size smoke test. See
   [WPT](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/using-gpuview)
   and [GPUView installation](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/installing-gpuview).
   The current WPR profile captures WDDM events but omits CPU context switches;
   an all-process scheduler profile was too large even in a short smoke. Nsight
   captured D3D12 API/workload events, but its combined run failed the RSS guard.
   Keep Nsight to short diagnostics until a lower-overhead mode passes a matched
   control. AMD Radeon GPU Profiler targets AMD GPU workloads and would not
   diagnose the renderer's selected RTX adapter.
   See [Nsight Systems](https://developer.nvidia.com/docs/drive/drive-os/7.0.3/public/nsight/nsight-systems/UserGuide/index.html)
   and [Radeon GPU Profiler](https://gpuopen.com/manuals/rgp_manual/).
   If system profiling remains unavailable, expand the renderer trace around
   `about_to_wait`, redraw/event callbacks, frame acquisition, submit, and
   `Present`, and keep the queue-full snapshot of phase, time since receive, and
   last native message.
3. Change one condition per follow-up. Use the captured phase and Windows trace
   to decide whether to compare a second display route or a different backend;
   the completed 30-minute route pair already shows that neither route alone
   deterministically reproduces the fault.
4. After a root cause is identified and a fix is covered by a regression check,
   rerun the short suite, schedule a one-hour validation, and leave the
   eight-hour soak as the final release gate.

Increasing the queue size or relaxing the sender/queue acceptance criteria would
hide symptoms without identifying the stall, so neither is a useful diagnostic
change.
