# Windows desktop and laptop soak investigation — 2026-10-03

## Findings

There are two observed failure mechanisms. The desktop RTX 3060 overflows the
renderer command queue; the laptop Intel and NVIDIA runs abort because the test
sender falls behind. The RX 580 has a completed eight-hour traffic pass. None of
these results establishes that a GPU is defective.

| Computer / GPU | Historical result | What stopped the run |
| --- | --- | --- |
| Desktop / RX 580 | Passed 28,800.002 seconds | Completed 28.8 million updates; no drops |
| Desktop / RTX 3060 | Failed after 107.355 seconds in the latest attempt | `E_QUEUE_FULL`; sender lag only 3.986 ms |
| Laptop / Intel UHD 630 | Failed after 20,199.89 seconds | Sender lag 311.996 ms exceeded the 250 ms guard |
| Laptop / GTX 1050 Ti Max-Q | Failed after 15.781 seconds | Sender lag 436.999 ms exceeded the same guard |

Historical sources: [desktop soaks](2026-10-03-windows-soak.md),
[desktop topology](2026-09-30-windows-dual-gpu.md), and
[laptop attempts](2026-10-03-windows-uhd630-gtx1050ti-soaks.md).
The desktop evidence was reviewed from these committed reports. Its original
October 3 logs and hardware were not available on this laptop for new measurement.

## Desktop: queue service and display routing

The user confirmed that RTX 3060 and RX 580 are both installed inside the
desktop; they are not external GPU enclosures. The September 30 inventory has
the primary and right monitors on RX 580, and the middle monitor on RTX 3060.
The long-run evidence does not identify the renderer window's monitor.

The renderer's incoming queue holds 256 commands (`renderer/src/osc.rs`). At
1,000 uniform updates/second, an initially empty queue has approximately 256 ms
of capacity, slightly less when accounting for other commands. Queue consumption
is tied to `about_to_wait` and redraw callbacks in `renderer/src/app.rs`.
A pause in that thread can therefore overflow the queue while the UDP receive
thread and sender keep working.

The earlier diagnostic captured exactly this mechanism: about 256 ms since the
last command consumption, with the renderer between instrumented callbacks for
240 ms. It did not identify the Windows operation responsible. A healthy average
frame rate and ten-second samples do not exclude such a short pause. Closing the
window after the diagnostic is the harness's cleanup, not proof of a GPU crash.

**Hypothesis to test:** an RTX-rendered window on an RX-driven monitor exercises
presentation between adapters, while an RX-rendered window on that monitor does
not. Microsoft documents [presentation between rendering and display adapters](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/supporting-caso).
This is a comparison variable, not an established cause. Microsoft also documents
that [Present can wait on window message processing](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-present).
The captured `event_loop` phase does not itself prove a Present stall.

Changing AC display-off to Never did not resolve RTX failures, and a short
failure occurred with the display reported on. Display timeout alone is therefore
insufficient to explain the desktop evidence. Actual physical monitor power
and display routing still need to be recorded during reproduction.

## Laptop: a confirmed sender pacing problem, plus an unresolved long stall

This investigation used a Dell XPS 15 9570, i7-8750H, Windows 11 Home
`10.0.26300`, Python 3.12.11, Intel driver `27.20.100.9664` and NVIDIA driver
`32.0.15.7216`. Read-only display enumeration found one active primary display
on Intel UHD 630. The GTX renders while Intel owns the display in this session.
The later owned-window snapshot reported 120 DPI (125% scale), whereas the
desktop inventory reported 96 DPI (100%).
AC power and 100% battery were observed; current AC display-off and sleep were
both Never. These observations do not reconstruct every condition of the old run.

Local `time.get_clock_info` reports:

| Clock | Windows implementation | Reported resolution |
| --- | --- | ---: |
| `time.monotonic()` | `GetTickCount64()` | 15.625 ms |
| `time.perf_counter()` | `QueryPerformanceCounter()` | 0.0001 ms |

The sender previously scheduled 1 ms updates using the first clock. A 30-second
control without a renderer or network traffic measured clock advances of 15/16 ms
and bursts of up to 16 due updates. This pacing difference exists independently
of GPU selection. It matches the [CPython 3.12 implementation](https://github.com/python/cpython/blob/v3.12.11/Python/pytime.c).
The desktop setup recorded Python 3.14.0, whose documented
[monotonic clock uses QueryPerformanceCounter](https://docs.python.org/3.14/library/time.html#time.monotonic).

The coarse clock explains batching; its 15.6 ms resolution does **not** explain
the entire 312–437 ms historical stall. The old logs lack per-operation or OS
scheduling traces. Intel still averaged 999.982 updates/second overall, with zero
drops in all 2,020 CSV samples; it failed the separate instantaneous lag guard.
NVIDIA averaged approximately 865.091 updates/second and had already skipped
1,692 ticks. It sent substantially less traffic than requested.

System events between 19:20 and 19:35 local time contain no recorded display
driver reset, sleep/resume or processor-throttling event. This does not rule out
driver delays, CPU scheduling, other applications, thermal limits or blocking
inside the sender. The elevated `powercfg /requests` query was unavailable;
historical process power requests were not established. No OS trace was captured
at the original failure, so assigning a specific process or driver would be a guess.

## New controlled short measurements

Source baseline `c1ec0c7293efd78bd3d82092911d2681dc325d04`.
All new GPU checks reused the laptop's original executable unchanged:
SHA-256 `df4d0eff7dee8490482902800ee60df6329539aba2892fde3c05015ad11dacea`,
development 0.0.18, D3D12, requested named adapter, 1,000 updates/second.
Optional renderer and display-power tracing was enabled. Tests ran serially.

| 60-second diagnostic | Clock | Maximum sender lag | Largest burst | Skipped | Outcome |
| --- | --- | ---: | ---: | ---: | --- |
| Intel | Original monotonic | 16.000 ms | 16 | 0 | Pass |
| NVIDIA | Original monotonic | 16.000 ms | 16 | 0 | Pass |
| Intel | Performance counter | 3.154 ms | 4 | 0 | Pass |
| NVIDIA | Performance counter | 3.769 ms | 4 | 0 | Pass |

The comparison wrapper changed the soak module's scheduling clock and measured
sleep/send/receive/RSS/logging call durations. Instrumentation adds some overhead;
these are sequential diagnostics, not a benchmark or long-duration qualification.
Display notifications stayed on, and no queue overflow or device loss occurred.
The original 250 ms sender failure did not reproduce in this sample.

`tools/soak_uniforms.py` now uses `perf_counter()` consistently for its elapsed
time, scheduling and health deadlines, and records interpreter/clock metadata in
result JSON. The 250 ms guard, 99% throughput requirement, bounded catch-up and
drop checks are retained. This corrects the demonstrated pacing issue; it does
not establish a fix for the rare sender stall or RTX queue failure.

After the correction, all four ten-second modes passed on each laptop adapter
(eight checks: traffic, trivial, reload, feedback). Traffic maximum bursts were
three; maximum lag was 2.430 ms on Intel and 2.617 ms on NVIDIA. No renderer,
driver, power setting, dependency, installed extension or version was changed.
No one-hour/eight-hour check or full unit suite was run in this investigation.

Raw evidence is in the user's new `SCShader-soak-investigation-20261003` Desktop
folder, outside source commits: `evidence/control-original`, `intel-original`,
`nvidia-original`, `intel-qpc`, `nvidia-qpc`, `intel-corrected-preflight`, and
`nvidia-corrected-preflight`. The prior `8hourtest` evidence is preserved.
Local diagnostic scripts and the display inventory are saved alongside it.
The updated local wrapper also captures initial monitor/window geometry and
accepts an explicit starting position for the desktop comparison. Its first
smoke hit a helper import error before traffic; that failed setup is preserved.
After correcting the import, a ten-second NVIDIA smoke with position `(64, 64)`
passed and captured `DISPLAY1` ownership. This is additional short evidence only.

The desktop used renderer hash
`b04ad1c4ed00589b5fc7deda79dcfb3ccdb8a9d58214e928437b7c5851e6e5ff`;
the laptop executable differs. Therefore the historical desktop/laptop outcomes
are not a controlled comparison of GPUs alone: executable, interpreter, Windows
build, CPU, drivers and display routing all differ.

## Next discriminating checks

1. On the desktop, record current display-to-adapter mapping. With one unchanged
   timing-enabled executable and the corrected sender, run short serial trials
   for RTX on its own monitor, RTX on an RX monitor, RX on its own monitor, and RX
   on the RTX monitor. Set placement before traffic begins and capture native
   window metrics. Keep monitors on and window state fixed for the first matrix.
2. If RTX alone fails regardless of display ownership, inspect the captured phase
   and Windows message at overflow. If failures follow display routing, focus on
   presentation between adapters. A `queue_full` snapshot is needed before
   changing the renderer's threading or queue design.
3. For another laptop sender stall, record an OS scheduling trace with sender
   sleep/send/receive/sample durations and renderer phase tracing. Python documents
   that [sleep may overrun because of other system activity](https://docs.python.org/3.12/library/time.html#time.sleep).
   A high-resolution clock measures delays accurately; it does not guarantee
   that Windows schedules the thread on time. Compare with the no-renderer control.
4. Once the intermittent causes are understood, schedule longer validation on
   one fixed candidate. Keep the failed runs and RX pass attributed to their exact
   artifacts. Increasing queue capacity or relaxing the guard would change the
   acceptance target and would not explain these failures.

The desktop matrix, OS trace at a natural stall, and renewed long-duration
qualification remain open. The local clock correction has short-run evidence only.
