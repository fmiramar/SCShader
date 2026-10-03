# Windows UHD 630 and GTX 1050 Ti Max-Q soak attempts — 2026-10-03

**Status:** both requested eight-hour traffic runs failed before the target
duration. Neither qualifies as an eight-hour check. This is a Windows 11 x64
development test of SCShader 0.0.18, not final-release sign-off.

The four ten-second D3D12 preflight modes (`traffic`, `trivial`, `reload`, and
`feedback`) passed on each adapter. Both full attempts used the same renderer
from source commit `a15b6be5bb71aaa91bd2c0ea9a26edee0ea51f47`, SHA-256
`df4d0eff7dee8490482902800ee60df6329539aba2892fde3c05015ad11dacea`, Python
3.12.11, and the D3D12 backend. The renderer logs confirmed the selected device.

| Adapter | Driver | Measured duration | Updates sent | Frames at failure | Skipped updates | Max sender lag | Outcome |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| Intel UHD Graphics 630 | 27.20.100.9664 | 20,199.89 s (5 h 36 m) | 20,199,532 | 1,212,005 | 46 | 311.996 ms | Failed: traffic sender exceeded its lag limit |
| NVIDIA GeForce GTX 1050 Ti with Max-Q Design | 32.0.15.7216 | 15.781 s | 13,652 | 789 | 1,692 | 436.999 ms | Failed: traffic sender exceeded its lag limit |

The harness aborts traffic mode when the Python sender falls more than 250 ms
behind its next scheduled update; see the [sender-lag check](../../tools/soak_uniforms.py#L347).
The Intel run delivered nearly its requested cumulative 1,000 updates/second,
but one 311.996 ms scheduling gap still invalidated the run. The NVIDIA attempt
hit the same guard during startup. Process RSS remained within the harness limit
(333,020 KiB peak on Intel and 190,288 KiB on NVIDIA); sampled incoming-queue
drops remained zero. The renderer frame counters advanced and the logs contain
no `E_QUEUE_FULL`, device-loss, or other renderer failure diagnostic; each has
only the expected malformed-packet protocol probe. This records a traffic-sender
timing failure, but does not establish its root cause or qualify either GPU for
eight hours.

Raw `run.json`, mode JSON, CSV, console, and renderer logs remain local and
ignored under `build/eight-hour-sequence-20261003-135026/`. They are not included
in this source commit.
