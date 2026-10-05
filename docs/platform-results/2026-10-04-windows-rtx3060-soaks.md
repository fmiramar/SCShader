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
