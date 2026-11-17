# Local video source — Step 20

`h.video(\s0, path, decoder, true, 1, action)` opens a local file using an explicit
absolute path to an installed FFmpeg executable. This is opt-in: starting SCShader,
using procedural visuals or loading images does not discover or start a decoder.
No FFmpeg/GStreamer library, executable or codec is added to the Rust dependency
graph or distributed package. Video audio is ignored; scsynth is uninvolved.
Wait for each callback before sending another structural request.

## Adapter choice and distribution

The comparison preceded implementation. FFmpeg's CLI can deliver raw RGBA frames
over a pipe with a small standard-library subprocess adapter. The local macOS
installation provides FFmpeg 9.0.1, built with GPL/version3 options. GStreamer is
a maintained alternative, but its launcher is absent here and an integration
would additionally require selecting/provisioning its runtime and codec plugins.
Neither is assumed present on another computer. This first adapter uses the
available FFmpeg executable supplied by the caller; no dependency or lockfile
changes are needed.

[FFmpeg's licensing page](https://ffmpeg.org/legal.html) explains that the license
depends on enabled components; optional GPL components change the result.
[GStreamer's licensing FAQ](https://gstreamer.freedesktop.org/documentation/frequently-asked-questions/licensing.html)
describes the LGPL framework and separate plugin/dependency considerations.
External provisioning avoids silently adding a bundled media runtime, but is not
a license clearance for every FFmpeg build. A future bundled decoder needs its
own exact-build notice/source and redistribution review. The existing renderer
distribution/notice gates remain in force.

## Playback and timing

`video` acknowledges worker acceptance, **not** successful decoding. Poll
`h.videoStatus(\s0, action)` until frames arrive, or inspect its error. The example
waits for a frame before announcing readiness. An invalid/missing file or decoder
fails preflight; unsupported media/codecs and process startup failures are reported
asynchronously in status. A newly accepted video replaces the prior decoder;
if it later fails, the last source picture remains. This does not restore the
previous decoder. Open a replacement explicitly to retry.

`h.videoControl(\s0, command, value, action)` supports:

| Command | Value and behavior |
| --- | --- |
| `\pause` | 0; stop/reap the decoder and retain the last uploaded picture. |
| `\play` | 0; restart decoding at the retained position. After EOS, restart at zero. |
| `\loop` | 0 or 1; disable/enable input looping and restart from the retained position. |
| `\rate` | 0.1…4; positive playback multiplier, restarting from the retained position. |
| `\seek` | 0…86400 source seconds; restart there if playing. While paused, retain the picture until play. |

The adapter uses FFmpeg's `fps=30` filter, preserves aspect with black letterboxing
into **640×360 RGBA8**, and emits tightly packed rows of 2560 bytes. It resamples
variable frame rates to a constant 30 fps. Source timestamps are therefore derived
from this resampled timeline (`seek offset + frame index / 30`), rather than
exporting original packet PTS. The worker schedules frame publication against
`Instant`, at intervals `1 / (30 × rate)` after decoder launch. Decoder startup
and keyframe-seek latency are additional. The position is the last published
source timestamp, not a display-vsync timestamp. While looping it is an unwrapped
timeline and can exceed clip duration. No duration or loop-phase query is supplied.

Video timing is independent of Hydra clock speed/BPM and of audio scheduling.
The render loop takes the newest ready frame each draw; if several become due
before rendering consumes them, older ready frames are replaced and counted as
dropped. Each slot has exactly one pending frame, plus the worker's current frame
and the renderer's retained CPU/GPU picture. No unbounded Rust frame queue exists.
FFmpeg and codec internals have their own buffering/allocation; this is **not** a
hard process-memory bound for arbitrary media. Four sources can own four decoders.

Clean EOS reaps the decoder, reports `ended` and retains the last picture. Decoder
failure reports `error` with a bounded diagnostic tail. Status returns `ok`,
`state` (loading/playing/paused/ended/error), `position`, `decoded` (frames published
to the ready slot), `dropped` (ready frames overwritten) and `error`. Counters are
cumulative per opened video and saturate at signed int32 maximum on the wire.

## GPU and lifetime

Decode, filter, pipe reads, pacing and child-process wait run on workers outside
the render and audio threads. The renderer only uploads a ready RGBA frame on
draw. After its first frame, a video reuses the same source texture and live
bindings; dimensions remain fixed. Texture uploads preserve encoded channel
values using RGBA8Unorm, with the same upright row orientation and sampling policy
as [images](IMAGES.md). Hydra coordinate/effect operations then process the source.
Resize retains the source; recovery preserves the worker and reconstructs GPU
resources from retained pixels. Source-specific recovery injection is NOT RUN.

Pause/control interrupts only the owned decoder process. The worker reaps it
before starting a replacement. Explicit source clearing, successful image
replacement, video replacement, session free and normal renderer close stop and
join the owned worker and reap its child. Cleanup can briefly delay the control
path; its latency is not a real-time guarantee. A forced OS kill/crash is outside
normal cleanup guarantees. Hush and Legacy selection retain sources, including
continuing video playback; use pause or clearSource when decoding should stop.
No shell is used, network protocols are disabled, and only caller-selected local
files/executables are opened. Stderr is drained independently with a bounded tail.

## Scope and reproducible checks

Native qualification is limited to macOS arm64 / Apple M5 / Metal and the local
FFmpeg build. GStreamer, Windows, Linux, other codecs, HDR/color-management parity,
hardware decoding, original VFR timing, audio playback and camera/screen sources
are NOT RUN or outside this adapter. This does not claim browser Hydra parity.

`examples/hydra/06_video_file.scd` selects a local file with a dialog, checks common
macOS decoder locations (or `SCSHADER_FFMPEG`), loops it through scale/rotate and
provides ordered pause/play/rate/seek/EOS/status/clear/cleanup blocks. It works
from an unsaved IDE document using installed shader assets; no private media is
distributed. Use a short 16:9 clip for the first visual check.

The native lifecycle check is opt-in. Create an owned one-second fixture under
ignored `build/`, then set `SCSHADER_TEST_VIDEO` to its absolute filename and
`SCSHADER_FFMPEG` to the decoder executable:

```sh
ffmpeg -nostdin -hide_banner -loglevel error -f lavfi \
  -i testsrc2=size=160x90:rate=30 -t 1 -c:v ffv1 build/clip.mkv
cargo test --manifest-path renderer/Cargo.toml --locked \
  native_video_lifecycle -- --ignored --nocapture
python3 tools/run_sc_checks.py hydra_video_smoke --output-dir build/video-sc
```

The lifecycle check covers bounded overwrite, loop/pacing, pause/resume beyond a
loop, rate/seek/EOS, restart, decoder failure and owned PID disappearance after
drop. Production Metal pixels cover 32 in-place uploads through unchanged bindings
and bounded texture bytes. The installed SC check executes the actual example
startup, effect, loop/controls/EOS/resize, replacement with an image, clear and
session disposal. See [VERIFICATION.md](../VERIFICATION.md) for measured results.
