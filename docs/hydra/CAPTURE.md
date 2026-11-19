# Camera and display sources — Step 21

The macOS adapter uses the explicitly provisioned FFmpeg executable's AVFoundation
input. The feasibility check preceded implementation: FFmpeg 9.0.1 lists two
built-in camera modes and one active display here; short camera and display probes
returned valid frames. No new dependency, bundled executable or browser runtime is
introduced. [FFmpeg documents AVFoundation enumeration and selection](https://ffmpeg.org/ffmpeg-devices.html#avfoundation);
its [implementation](https://github.com/FFmpeg/FFmpeg/blob/master/libavdevice/avfoundation.m)
uses AVCaptureDevice and AVCaptureScreenInput. This first screen adapter uses
that existing AVFoundation path, not ScreenCaptureKit or a system capture picker.
The external-runtime licensing/distribution decision remains [VIDEO.md](VIDEO.md).

## Explicit discovery and selection

`h.captureDevices(decoder, action)` starts a bounded worker-based enumeration and
returns an Event with `ok`, `state` and `devices`. Each device has `kind` (camera
or screen), `index` and `name`. Enumeration opens no capture session and ignores
audio devices. The worker has a five-second deadline and retains at most 16 KiB
of diagnostic text. Lists are limited to 16 devices, indexes 0…255 and names up
to 512 bytes internally; wire/display labels truncate safely to 96 UTF-8 bytes.

After successful enumeration, call `h.initCam(\s0, index, action)` or
`h.initScreen(\s1, index, action)`. Nil index selects the first enumerated device
of that kind. Explicit indexes refer to the returned list, including screen
indexes; they are not display ordinals and need not start at zero for screens.
Wrong-kind, absent and ambiguous selections fail before replacing an active source.
Enumerate again after connecting/disconnecting devices or changing displays.

The worker selects the retained full device name, avoiding a camera-index shift
when a screen is requested. Duplicate/reserved/numeric/colon-containing names
that FFmpeg could interpret ambiguously are rejected. Display names identify
AVFoundation's current display ordinal; this is not a stable physical-display ID.
The adapter does not automatically reconnect or silently choose another device.

Each init call explicitly starts capture. Its acknowledgement means worker
acceptance, not permission granted or a frame received. Poll `captureStatus` before
announcing readiness. A failed asynchronous open retains the previous picture;
it does not restore a decoder already replaced. Wait for each structural callback.

## Frames, control and lifetime

The camera requests 640×480 at 30 fps; unsupported formats may fail or be negotiated
by FFmpeg/AVFoundation. Display input uses its native dimensions. Startup-reported
native dimensions are exposed as `inputWidth`/`inputHeight`; do not infer them from
window size. Both sources are converted to 640×360 RGBA8 with aspect-preserving
black letterboxing and resampled to 30 fps. The example's small zoom crops slightly.
Output dimensions/nominal filter rate are `width`, `height`, `fps`; this fps field
is a target, not a measured hardware rate. The actual tested native sizes were
640×480 camera and 2940×1912 display. HDR, ICC and original capture-PTS parity are
not promised.

Capture shares the video worker, one pending-frame queue, encoded RGBA8Unorm color
policy and stable source texture uploads. Native reads/conversion/pacing/process
waits stay off the GPU and scsynth threads. The render loop consumes the newest
ready frame and overwrites older queued frames under lag. FFmpeg/codec/OS buffers
are additional; there is no hard process-memory bound. Up to four source slots
share image/video/capture ownership, with transient overlap during replacement.

`h.captureStatus(slot, action)` returns the video status fields plus `kind`, native
input dimensions, output dimensions and nominal fps. Live `position` is elapsed
worker time since the latest start/resume; it is not original device PTS or
display-vsync time. Hydra clock/BPM does not change capture timing.

`h.captureControl(slot, \pause, action)` stops/reaps the process and retains the
picture; `\play` opens a fresh live stream. Control acknowledgement accepts the
desired state; worker reaping follows. Live seek/loop/rate requests fail without
changing accepted state. If capture exits, or publishes no frame for ten seconds,
status becomes `error`, with a device/permission hint. The timeout also covers
an unanswered OS prompt. Clear and reopen after resolving the cause. Genuine
physical disconnect and sleep/wake remain unqualified; the timeout is tested with
an explicitly stopped, test-owned process.

Successful image/video/capture replacement, `clearSource`, session free and normal
renderer close stop/join the old worker and reap its child. Pausing is source-local.
Hush and Legacy selection retain captures and can leave the camera indicator on;
pause, clear or free when capture should stop. Resize retains source textures;
logical-device recovery retains workers and cached pixels, but capture-specific
recovery injection is NOT RUN. No automatic capture is added to startup or the
default SC/Rust suites. Live tests require explicit selection/authorization.

## Permissions and privacy

Camera and Screen Recording are separate macOS permissions. An external USB
webcam does not bypass Camera authorization. The built-in camera sufficed here.
[Apple describes camera authorization and usage descriptions](https://developer.apple.com/documentation/bundleresources/requesting-authorization-for-media-capture-on-macos).
Permissions depend on the executable/responsible launching app. Terminal success
does not guarantee IDE success. If the IDE fails, inspect the error first, then
check System Settings → Privacy & Security → Camera or Screen & System Audio
Recording for the relevant launching app. Grant only the access you intend and
reopen the source/app when macOS requests it. No TCC resets, preference changes,
permission bypass or microphone access are performed by this adapter.

Raw FFmpeg failure diagnostics and the no-frame timeout provide actionable hints;
they are not an authoritative query of another process's TCC state. If macOS
delivers masked black frames, those cannot reliably be distinguished from a dark
scene. Native deny/revoke tests are NOT RUN; existing permissions were preserved.
Screen capture covers the entire selected display, not one chosen window. It can
include private content and the renderer itself (visual recursion). No captured
camera/display frames were saved in test evidence or committed to source control.

## Capability matrix and checks

| Platform/service | Implementation | Evidence/limits |
| --- | --- | --- |
| macOS arm64 / camera | External FFmpeg AVFoundation | Apple M5 built-in camera; native worker and installed SC checks PASS. Physical IDE appearance pending. |
| macOS arm64 / display | External FFmpeg AVCaptureScreenInput | One built-in display; native worker and installed SC checks PASS. ScreenCaptureKit, window picker and additional displays NOT RUN. |
| macOS Intel | Same code path, unqualified | NOT RUN. |
| Windows camera/display | No adapter | NOT IMPLEMENTED; no silent fallback. |
| Linux PipeWire/portal/V4L2 | No adapter | NOT IMPLEMENTED; no silent fallback. |
| USB camera, denied/revoked permission, physical unplug, sleep/wake | No separate qualification | NOT RUN; failure/timeout handling and selection guards are tested. |

`examples/hydra/09_capture.scd` runs from installed assets: enumerate, start camera,
start screen, show quadrants, pause/resume, inspect status, clear and cleanup.
Run blocks in order and wait for each READY/result. It boots no audio server.
The explicit `hydra_capture_smoke` check executes the actual startup/helper, two
live sources/effects, metadata, independent pause/resume, resize, image replacement,
clear/free, wrong-kind selection and unsupported live seek.

Native worker checks are opt-in:

```sh
# Set SCSHADER_FFMPEG to the absolute installed decoder path.
# These commands explicitly activate the real camera and entire display.
cargo test --manifest-path renderer/Cargo.toml --locked \
  native_capture_lifecycle -- --ignored --nocapture
python3 tools/run_sc_checks.py hydra_capture_smoke --output-dir build/capture-sc
```

The native check uses a lit/visible scene, checks real RGBA frames, native dimensions,
pause/resume and owned PID cleanup. A SIGSTOP on its own screen process simulates
stalled input; advancing the timeout state avoids a ten-second test delay. This
does not certify a physical unplug or OS denial. See [VERIFICATION.md](../VERIFICATION.md).
