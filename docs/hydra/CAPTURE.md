# Camera and display sources — Step 21

macOS camera capture now runs in the bundled **SCShader Camera** app using native
AVFoundation. The helper owns its camera permission and supplies an explicit
`NSCameraUsageDescription`. The original FFmpeg camera path passed CLI checks but
the user reported no frames from the SC IDE; their installed SC 3.14.1 bundle has
no camera usage declaration. This is a launch/permission integration limitation,
not evidence that the user denied access. Screen capture was confirmed working
in the IDE after its separate macOS permission prompt.

The helper is launched through LaunchServices when a camera is explicitly opened;
it does not modify SuperCollider's bundle or reset TCC permissions. Screen capture
and enumeration retain the external FFmpeg/AVFoundation adapter. No new Cargo
dependency, bundled FFmpeg or browser runtime is introduced.

## Discover and select

`h.captureDevices(decoder, action)` explicitly enumerates FFmpeg's AVFoundation
video devices, ignoring audio. It returns `ok`, `state` and `devices`; each entry
has `kind`, `index` and `name`. Enumeration does not start a capture session. It
runs on a worker with a five-second deadline, at most 16 KiB retained diagnostics,
16 devices and indexes 0…255. Full names are limited to 512 bytes internally;
display/wire labels truncate to 96 UTF-8 bytes without splitting characters.

`h.initCam(\s0, index, action)` and `h.initScreen(\s1, index, action)` start the
selected source. Nil index selects the first enumerated matching kind. Explicit
indexes come from the returned list; screen indexes are not display ordinals.
Wrong-kind/absent/ambiguous selections fail before replacing an active source.
Re-enumerate after changing devices/displays. Full names are retained internally;
there is no silent other-device fallback or automatic reconnect.

An init acknowledgement accepts a worker, not a frame or an OS authorization.
Poll `captureStatus` for progress. A failed asynchronous open retains the previous
picture; it does not restore a decoder already replaced. Structural requests must
wait for their callbacks.

## Native camera and frames

`renderer/SCShaderCamera.app` is built from the included Swift source with the
macOS SDK. AppKit runs its main loop and AVFoundation checks camera authorization,
requesting it only for an explicit camera open. No microphone is opened. The helper
selects the retained camera name, requests the VGA capture preset and 30 fps where
supported, and reports its actual first-frame dimensions. Preset and hardware
negotiation can differ: the follow-up native check reported 1920×1080, while the
previous FFmpeg camera check reported 640×480. Inspect inputWidth/inputHeight.

Native BGRA rows convert to upright opaque **640×360 RGBA8** with nearest-neighbor
aspect fit and black letterboxing. A padded-row synthetic test checks RGB order,
orientation and letterbox alpha. AVFoundation discards late frames; the renderer
pacer caps publication at the nominal 30 fps. It does not duplicate slow camera
frames. Screen input still uses FFmpeg's fps/scale/pad filter, tested at native
2940×1912 and the same RGBA output size. HDR/ICC and browser numerical parity
remain unqualified.

A private random Unix socket in a mode-0700 directory carries framed messages and
one fixed-size picture at a time. The renderer owns the directory, bridge worker
and decoder lifetime; the helper exits when its controller connection closes.
Native reads/conversion and process waits stay off renderer/audio threads. Rust
retains one pending frame, its working frame and the last CPU/GPU picture; old
pending frames are dropped under lag. Native/FFmpeg/OS buffering is additional,
so this is not a hard process-memory bound. Uploads retain the source texture and
bindings. Resize keeps sources; capture-specific GPU recovery injection is NOT RUN.

## Permission, status and stopping

On first use, macOS may ask to allow **SCShader Camera**. The helper has its own
permission entry, independent of Terminal or SuperCollider. It requests only
video access. Apple requires the usage declaration and describes authorization
[here](https://developer.apple.com/documentation/bundleresources/requesting-authorization-for-media-capture-on-macos).
An external webcam uses camera authorization too; the built-in camera sufficed.
No SC bundle edits, permission bypass, TCC reset or automatic permission grant is
performed. Ad-hoc development signing is not signing/notarization qualification;
a changed helper build can cause macOS to ask again.

`h.captureStatus(slot, action)` returns state, position, decoded/dropped counts,
error, kind, inputWidth/inputHeight, width/height and nominal fps. Camera state
`permission` means its OS prompt is pending; the helper gets up to 60 seconds
for that decision. The example waits up to 75 seconds and prints the permission
owner. Once authorized, the ordinary ten-second no-frame timeout applies. Errors
retain bounded native/decoder diagnostics rather than discarding them at timeout.
This is not a TCC query for arbitrary apps; masked black frames cannot reliably
be distinguished from a dark scene. True deny/revoke remains NOT RUN.

`captureControl(slot, \pause, action)` releases that capture and retains its
picture; `\play` starts a new live stream. Live seek/rate/loop fail without
changing accepted state. Position is worker elapsed time since start/resume;
nominal fps is not a measured hardware rate or device PTS. Hydra BPM/speed is
independent. Screen permission still belongs to its launching app, and capture
covers the entire selected display. Including the renderer creates recursion.

`clearSource`, successful source replacement, free and normal close release the
owned worker/bridge/helper. Hush/Legacy retain captures: explicitly pause, clear
or free when the camera should stop. The example's cleanup function is safe to
run repeatedly, including while shutdown is in progress, and blocks restart until
CAPTURE STOPPED. Camera/screen frames are not saved in evidence. Explicit live
checks are excluded from default suites.

## Checks and capability matrix

| Scope | Evidence and remaining limits |
| --- | --- |
| Native camera helper / macOS arm64 | Native frames, sizes, channel/stride/orientation proof, pause/resume and bridge/helper PID cleanup pass; IDE visual retest pending. |
| FFmpeg display / macOS arm64 | Native and installed checks pass; user confirms IDE display appearance and OS prompt. Additional displays/window picker/ScreenCaptureKit NOT RUN. |
| macOS Intel | Source/build path provided; native hardware NOT RUN. |
| Windows / Linux camera/display | No adapter; no silent fallback. |
| USB camera, OS deny/revoke, physical unplug, sleep/wake | NOT RUN. Stalled owned-process and cleanup tests are separate evidence. |

`examples/hydra/09_capture.scd` uses installed assets. Run startup, camera, screen,
quadrants, pause/resume/status and cleanup blocks in order, waiting for READY or
an explicit error. No scsynth boot is required. Close an old staged copy and reopen
the installed/updated file after upgrading. The smoke runs these helpers with two
live sources, selection/control rejections, resize, replacement, clear and repeated
cleanup.

For native development checks, first build the renderer and camera app with
`tools/build_camera_macos.sh`, then set SCSHADER_CAMERA_HELPER to the app path and
SCSHADER_CAMERA_BRIDGE to the renderer executable. SCSHADER_FFMPEG names the external
FFmpeg executable. The production package discovers its bundled helper and bridge
automatically. These commands explicitly activate real camera/display sources:

```sh
cargo test --manifest-path renderer/Cargo.toml --locked \
  native_capture_lifecycle -- --ignored --nocapture
python3 tools/run_sc_checks.py hydra_capture_smoke --timeout 100 \
  --output-dir build/capture-sc
```

See [VERIFICATION.md](../VERIFICATION.md) for exact binaries and measured results.
The external [FFmpeg adapter](VIDEO.md) licensing decision remains applicable to
screen/discovery provisioning.
