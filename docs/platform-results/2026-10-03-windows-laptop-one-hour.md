# Windows laptop: corrected-sender one-hour results — 2026-10-03

**Both adapters passed the one-hour traffic check.** The sequence finished normally
at 23:15:50 on October 3, America/Sao_Paulo (02:15:50 UTC on October 4).
This qualifies one hour of traffic on the exact candidate below; eight-hour
qualification and the desktop RTX investigation remain open.

## Candidate and procedure

- Harness snapshot: `bfa86e62c74eb99b08ada2d7b8e2ca7ca9dc38d4`, including the
  `perf_counter()` sender-clock correction. Python 3.12.11.
- Renderer: development 0.0.18, original build from `a15b6be`, SHA-256
  `df4d0eff7dee8490482902800ee60df6329539aba2892fde3c05015ad11dacea`.
- Dell XPS 15 9570, i7-8750H, Windows 11 Home `10.0.26300`; Intel UHD 630 driver
  `27.20.100.9664`, GTX 1050 Ti Max-Q driver `32.0.15.7216`.
- Explicit D3D12 and named adapter, 3,600 seconds each, traffic only at 1,000
  uniform updates/second, high-performance preference, overlay disabled.
- The standard `run_acceptance.py` ran Intel then NVIDIA serially, without the
  earlier diagnostic wrapper's per-call instrumentation. Renderer phase and
  display-power tracing were enabled. The prior WPR policy failure meant no OS
  scheduling trace was available.
- Files were copied to a separate candidate snapshot and checked against its
  manifest before/after each adapter. The supervisor checked the standard result
  validator and the runner's `qualifies_as_one_hour_check` flag.
- AC power was confirmed at startup. A temporary system/display wake request was
  active during the sequence and cleared at exit; power-plan settings were not
  changed. Windows display-power notifications remained on.

## Measured results

| Adapter | Seconds | Updates sent | Sender-skipped ticks | Maximum sender lag | Outcome |
| --- | ---: | ---: | ---: | ---: | --- |
| Intel UHD 630 | 3600.000730 | 3,599,980 | 19 | 20.202 ms | Pass |
| GTX 1050 Ti Max-Q | 3600.000170 | 3,599,978 | 20 | 18.969 ms | Pass |

Both recorded 3,597 pong/status replies and 361 CSV samples. Maximum bursts were
16 updates. There were zero sampled renderer queue drops or scheduled rejections,
and no queue-overflow/device-loss diagnostic. Each log contains the single
intentional malformed-packet `E_PROTOCOL` probe expected by the harness.

| Adapter | Frame counter | Initial RSS KiB | Baseline RSS KiB | Peak RSS KiB | Final RSS KiB |
| --- | --- | ---: | ---: | ---: | ---: |
| Intel | 6 → 215,984 | 333,752 | 333,980 | 335,516 | 335,516 |
| NVIDIA | 5 → 215,987 | 187,412 | 190,528 | 191,296 | 182,348 |

RSS stayed within the original 512 MiB absolute and 128 MiB growth bounds.
Average send rates were 999.994 and 999.994 updates/second. The small number of
sender-skipped ticks comes from bounded catch-up; it is separate from renderer
queue drops. Both runs satisfy the unchanged 99% throughput rule,
and maximum sender lag stayed below the unchanged 250 ms limit.

The renderer logs show slow initialization only, with no later logged slow
instrumented phase. This does not measure every Windows scheduling interval.
The historical 312–437 ms sender stalls did not reproduce. A one-hour pass cannot
exclude the old Intel failure that occurred after more than five hours, and does
not establish the underlying intermittent cause or an eight-hour pass.

## Evidence and remaining work

The user authorized both one-hour runs and requested no live polling. A single
startup check confirmed Intel selection; the supervisor automatically advanced
and wrote its report. Both renderer processes and the supervisor have exited.

Raw evidence remains outside Git in Desktop folder
`SCShader-one-hour-tests-20261003-2113`: `candidate-manifest.json`, `summary.json`,
`RESULTS.md`, and `evidence/intel-one-hour/` / `evidence/nvidia-one-hour/` with
run/result JSON, CSV, renderer/console logs and power notifications. Previous
failure evidence is preserved. No source/runtime changes or new tests were added
during result review; this update records the completed authorized runs.

Next gates: the desktop GPU/monitor comparison and separately authorized
eight-hour validation. See the [investigation](2026-10-03-windows-soak-comparison.md)
for the distinction between renderer queue overflow and sender stalls.
