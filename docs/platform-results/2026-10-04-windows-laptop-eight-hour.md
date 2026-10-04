# Windows laptop eight-hour traffic results — 2026-10-04

**Intel UHD 630 and NVIDIA GTX 1050 Ti Max-Q both passed 28,800 seconds of
traffic on the exact candidate below.** The serial sequence ran from October 3
at 23:50:24 to October 4 at 15:50:30, America/Sao_Paulo. Both owned renderers and
the supervisor exited normally. The user requested unattended execution, and
there was no repeated assistant polling during either run.

## Candidate and conditions

- Development renderer 0.0.18, built from `a15b6be`, SHA-256
  `df4d0eff7dee8490482902800ee60df6329539aba2892fde3c05015ad11dacea`.
  This is the same executable as the earlier failed laptop attempts and the
  subsequent successful short and one-hour runs.
- Corrected sender uses `perf_counter()` / QueryPerformanceCounter. Python
  3.12.11. The manifest records source `a4cf5acf7f5533d0400442fe37757ab868bb1e7b`.
  The snapshot was copied from the one-hour candidate, originally taken at
  `bfa86e6`; all seven harness files were verified identical to `a4cf5ac`.
- Dell XPS 15 9570, i7-8750H, Windows 11 Home `10.0.26300`; Intel driver
  `27.20.100.9664`, NVIDIA driver `32.0.15.7216`. The active laptop display is
  driven by Intel; each run explicitly selected and reported its rendering GPU.
- D3D12, high-performance preference, overlay disabled, traffic only,
  1,000 uniform updates/second, 28,800 seconds per adapter. The standard
  acceptance runner retained its 250 ms sender-lag guard, 99% rate requirement,
  queue-drop checks, fresh-reply/frame checks and memory limits.
- AC power and full battery were observed at startup. The supervisor requested
  system/display wakefulness while running and cleared it at exit. No power-plan
  setting changed. Windows display-power notifications remained on.
- Renderer phase and display-power tracing were enabled. Windows OS scheduling
  tracing remained unavailable after the prior WPR policy rejection. The earlier
  diagnostic wrapper's per-call instrumentation was not used for acceptance.

## Measured results

| Adapter | Measured seconds | Updates sent | Sender-skipped ticks | Maximum sender lag | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| Intel UHD 630 | 28800.001044 | 28,799,902 | 98 | 19.310 ms | Pass |
| GTX 1050 Ti Max-Q | 28800.000920 | 28,799,804 | 196 | 22.435 ms | Pass |

Intel delivered 99.999660% of the requested update count; NVIDIA delivered
99.999319%. Sender-skipped ticks account for 0.000340% and 0.000681% respectively.
They are the sender's bounded catch-up behavior and are distinct from renderer
queue drops. Maximum bursts were 16 on each GPU; both passed the unchanged rate
and lag requirements.

Each run recorded 28,773 pong/status replies and 2,881 CSV samples. Every CSV
sample reports zero continuous queue drops and zero scheduled rejections. There
was no queue-overflow or device-loss diagnostic. Each renderer log contains only
the one intentionally malformed OSC probe as an error (`E_PROTOCOL`). The only
logged slow instrumented phase was startup initialization.

| Adapter | Frame counter | Initial RSS MiB | Peak RSS MiB | Final RSS MiB | Final minus warmed baseline MiB |
| --- | --- | ---: | ---: | ---: | ---: |
| Intel | 7 → 1,728,171 | 327.31 | 331.61 | 331.61 | +5.16 |
| NVIDIA | 7 → 1,728,167 | 187.79 | 191.37 | 182.89 | -7.83 |

These are renderer process working-set measurements, not GPU VRAM measurements.
Both remained within 512 MiB absolute and 128 MiB growth limits. The data does
not show unbounded memory growth during these eight hours; it cannot exclude
every possible leak in other workloads or durations.

## Independent evaluation and reporting discrepancy

Post-run evaluation rechecked all eight candidate-file hashes, invoked the
standard result validator for both adapters, and cross-checked the per-adapter
runner JSON against the traffic JSON. It also checked duration, requested mode,
throughput, lag, frame progress, RSS and the full CSV series. Both independent
per-adapter `run.json` records have `passed: true` and
`qualifies_as_eight_hour_check: true`.

The local supervisor has a reporting bug: its top-level
`summary.json.qualifies_as_eight_hour_check` was initialized to false and never
updated. Its two per-adapter flags are true. This stale aggregate flag does not
reflect the validated individual results. Original raw JSON and the executed
supervisor are preserved; a separate `evaluation.json` records the independently
verified passes and explains the discrepancy. The four-mode suite flag is
correctly false: trivial, reload and feedback were not run for eight hours.

## Interpretation and remaining scope

The laptop now has positive eight-hour traffic evidence on both GPUs with the
corrected sender. Neither earlier 312–437 ms sender stall recurred, including
after the elapsed time at which the old Intel attempt failed. This is evidence
that the tested configuration sustains the requested workload for eight hours.
It does not isolate the cause of the historical stalls: application tracing,
display wakefulness and system activity also differ between attempts, and there
is no OS scheduling trace from the old failures. The demonstrated clock pacing
correction must not be described as a proven sole cause of those stalls.

The desktop RTX 3060 queue-overflow failure remains unresolved and requires the
desktop GPU/monitor comparison. Its packaged executable differs from this laptop
candidate. These passes cover the exact laptop binary, named adapters, D3D12 and
traffic mode under the recorded conditions. They do not qualify other modes,
sleep/wake, other platforms, or final release readiness.

## Evidence

All raw files remain local in Desktop folder
`SCShader-eight-hour-tests-20261003-2349`: `candidate-manifest.json`, `summary.json`,
`RESULTS.md`, `evaluation.json`, the audit script, and
`evidence/intel-eight-hour/` / `evidence/nvidia-eight-hour/`.
The binary/harness snapshot and original failed attempts are preserved. No new
GPU run was started during this evaluation. Only reviewed documentation is synced.

Related evidence: [original laptop failures](2026-10-03-windows-uhd630-gtx1050ti-soaks.md),
[investigation](2026-10-03-windows-soak-comparison.md),
[one-hour passes](2026-10-03-windows-laptop-one-hour.md), and
[desktop results](2026-10-03-windows-soak.md).
