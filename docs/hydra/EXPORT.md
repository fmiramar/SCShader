# Screenshots, recording and presentation — Step 23

Export is opt-in and captures the **Hydra presentation**, not the desktop or
camera. No screen-recording permission or audio server is needed. Export is
inactive by default: no export target, staging buffer, map or encoder exists
until a request starts one. The existing `render(\o0)` / `render(nil)` selection
controls the captured single output or quadrants, with the established layout:
o0 top-left, o1 bottom-left, o2 top-right, o3 bottom-right.

Use [11_export.scd](../../examples/hydra/11_export.scd) after recompiling classes.
Wait for `EXPORT READY` after startup, then run numbered blocks 2…7 in order.
These blocks use one FIFO task runner, so submitting the next block immediately
queues it until the prior export is finished. `EXPORT STEP QUEUED/START/DONE`
messages distinguish scheduling from completion. Cleanup queues after recording
finalization. Each export receives a fresh sequence suffix, including repeated
button/block execution. A failed task reports its error and clears remaining
queued tasks. It creates a capture
folder in the user's home directory, prints its location, saves a PNG, records
approximately five seconds, then saves a smaller single-output PNG. Files have a
per-start name and existing files are never overwritten. Startup alone exports
nothing. Cleanup may be run repeatedly; finish recording before cleanup to obtain
a finalized MP4.

## API and formats

Wait for each structural request callback before another request. An accepted
start is not a completed file: poll `exportStatus` for `finished` or `error`.

| API | Behavior |
| --- | --- |
| `h.setResolution(width, height, action)` | Set actual render pixels independently of the window; dimensions 1…4096 within device limits. |
| `h.setResolution(0, 0, action)` | Restore following the window's physical pixel dimensions. |
| `h.screenshot(absolutePath, action)` | Save the next available Hydra presentation as RGBA8 PNG. |
| `h.startRecording(absolutePath, encoder, fps, action)` | Start MP4 recording through an explicitly supplied absolute FFmpeg path; integer FPS 1…60, default 30. |
| `h.stopRecording(action)` | Stop accepting frames and begin asynchronous encoder finalization. |
| `h.exportStatus(action)` | Return state, path, width, height, FPS, written/drop counts, staging-buffer count and error. |

PNG supports odd dimensions. Exports are limited to **2048×2048**, even if the
renderer supports larger outputs. MP4 requires even width and height. Paths must
be new, absolute, in an existing directory and end in `.png` or `.mp4`.
Acceptance is serialized: one screenshot or recording at a time. Finish before
starting another export, changing render resolution or switching to Legacy mode.
The latest terminal status remains until another accepted export or session close.

The MP4 adapter supplies RGBA8 raw frames to external FFmpeg, requests H.264
`libx264`, `ultrafast`, CRF 20 and `yuv420p`, and disables audio. Its build must
support that encoder/container. Codec/process failures are asynchronous status
errors, retaining the visual output; FFmpeg stderr is not captured into an
unbounded log. No FFmpeg executable/library, codec or dependency is added to the
package. Existing [external-runtime and licensing caveats](VIDEO.md#adapter-choice-and-distribution)
apply. `SCSHADER_FFMPEG` selects an encoder for the example; supported macOS
Homebrew locations are checked if it is unset. Other platforms require their own
encoder path and native verification.

PNG retains encoded Hydra RGBA channel values and upright presentation orientation.
MP4 is lossy, with chroma subsampling and no alpha. Neither format includes the
window border, debug overlay, desktop, cursor or audio. The exported canvas has
the render resolution; it excludes window letterboxing. No HDR or color-management
qualification is added.

## Resolution and aspect

Before an explicit resolution is chosen, the previous behavior remains: Hydra
histories follow window physical pixels, including Retina scale. A fixed
resolution recreates all four output histories only when dimensions change and
clears their two banks. Programs, numeric state, source images and selected output
remain; the Hydra clock is not reset. Repeating the same dimensions keeps history.
Failure retains the previous valid size/programs.

Fixed-resolution presentation preserves its aspect ratio, adding black bars when
the window/fullscreen has another aspect. Export dimensions are unchanged by
logical window resizing, DPI or fullscreen. The existing `ShaderWindow` size,
fullscreen, borders, vsync and metrics remain the window-control API. Selecting
an output or quadrants while recording is allowed and affects subsequent frames.
Restoring automatic resolution can produce dimensions above the export limit;
set a smaller fixed resolution before exporting again.

## Timing, bounds and lifecycle

Recording samples the live renderer using monotonic wall-clock deadlines.
When macOS hides/minimizes a window or surface presentation is unavailable,
active export renders Hydra offscreen at its capture deadlines. Export and its
completion polling do not require a window frame to be presented. Hidden export
uses paced event-loop wakeups and the same single readback buffer; a pending map
prevents additional capture-frame submissions. Hydra time continues, and its
frame uniform includes offscreen export frames. The window presentation counter
counts actual presentations, separately from these exports. Restoring the window
resumes presentation. Offscreen work is submitted only for active export.

Capture does **not** drive the Hydra clock and provides no offline deterministic-render
mode. A busy readback or writer drops samples; missed deadlines are also counted.
There is no catch-up allocation or render-thread encoder wait. The movie's constant
frame rate is the requested FPS, so its duration is **writtenFrames / FPS**.
Dropped samples shorten the movie relative to elapsed wall time. Start/stop and
write-completion times are not audiovisual alignment or GPU latency measurements.
Closing can discard the final in-flight sample. Recordings auto-stop after ten
minutes; start another recording explicitly if needed.

Each active export owns one RGBA presentation texture and **one reusable padded
GPU readback buffer**, capped by the export dimensions (at most 16 MiB each).
Readback is mapped asynchronously after submission. At most one frame waits in
the bounded writer queue, one is being written, and one is being read/handed off.
PNG encoding and potentially blocking FFmpeg pipe writes run on a worker, outside
the render/audio threads. Row-padding removal and handoff run on the renderer;
large exports can still add CPU/GPU load. External encoder internals have their
own memory use; no hard whole-process memory budget is claimed for FFmpeg.

A screenshot requests shutdown after ten seconds without completion. An ongoing
filesystem write is not forcibly interrupted; it finishes asynchronously. Stop
permits five seconds for
normal encoder finalization; a stalled pipe/process is killed after the bounded
shutdown deadline and reaped. Terminal completion/error releases the readback and
presentation resources. Session close, renderer quit or logical-device recovery
aborts active export. A partial/invalid destination can remain after cancellation
or failure; choose a new filename for a retry. Recovery does not silently restart
a recording across a discontinuity.

## Evidence

Native Apple M5/Metal tests cover valid odd-size PNG, exact RGBA/quadrant
orientation, single output, fixed pixel dimensions across window resize, H.264
frame count/duration, rejected overwrite/odd/busy/resolution requests, stalled
encoder shutdown, successful screenshot after encoder failure and repeated cleanup.
A macOS native minimization test reproduces the old zero-frame/empty-PNG failure,
then verifies PNG pixels and an animated MP4 with the window still minimized,
unchanged presentation count and successful window restoration. Actual example
blocks 2…7 are executed back-to-back through their shared FIFO runner and all
files are decoded. These close gaps in the original visible-window-only tests.
Native GPU accounting returns all eight tracked resource classes to baseline after
one-shot export; window letterboxing is checked in linear/sRGB targets. The
opt-in checker is:

```sh
python3 tools/check_hydra_export.py --ffmpeg /absolute/path/to/ffmpeg \
  --ffprobe /absolute/path/to/ffprobe --output-dir build/export-check-new
```

It requires a fresh output directory. `--renderer` and `--sclang-config` can select
a source/candidate build; omit the renderer override for installed discovery.
The generic SC suite excludes this file-writing/external-encoder check by default.
See [VERIFICATION.md](../VERIFICATION.md) for the tested binary and negative
evidence. Physical IDE/fullscreen appearance, custom export under device loss,
Windows/Linux/Intel hardware, other encoders/formats and long recordings remain
NOT RUN. This is a short native development checkpoint.

The dedicated macOS regression requires already-granted Accessibility permission
and operates only its owned test renderer. It does not change permissions/settings:

```sh
python3 tools/check_hydra_export_occluded.py --renderer /absolute/path/to/renderer \
  --ffmpeg /absolute/path/to/ffmpeg --ffprobe /absolute/path/to/ffprobe \
  --output-dir build/export-minimized-new
```
