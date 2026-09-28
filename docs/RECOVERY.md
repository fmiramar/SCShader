# Diagnostics and renderer recovery

The optional `ShaderServer.showStats` panel uses an original 5x7 bitmap font and
a small texture refreshed at most four times per second. It adds no font/UI
dependency. Its final presentation-only pass is excluded from graph/feedback
textures; on Metal it follows the existing separate-submission workaround.
Disabling it releases its GPU objects. The panel scales to fit small windows.
CPU timing is render-call duration, not GPU execution time. GPU timing is
explicitly unavailable; texture bytes are an owned-allocation estimate, not VRAM
usage. Last compile result is the most recent compile attempt; an invalid edit
still leaves the previous valid pipeline rendering.

## Device restart contract

Each renderer process permits **one** recovery attempt. A wgpu device-lost
callback records the loss; the main event loop stops normal work, drops the old
device/surface/resources, creates a new device and surface, and restores CPU-held
state. No OS-level GPU reset is requested and no audio-server work is involved.

Restored state includes last-valid source text (even if the file is now invalid
or missing), source-watch timestamps, typed values and active interpolation
ramps, decoded image pixels, Float32 buffer values, graph selection, bindings,
feedback enablement, mouse state, overlay choice, clocks, frame count, and IDs.
The application retains scheduled commands and window settings. Ramp deadlines
continue in the original monotonic clock, so a completed ramp advances to its
target on the next frame. Source files are not re-read during recovery.

Feedback/intermediate GPU image history cannot be reconstructed and clears to
black. `/scshader/v1/gpu/recovered` and `ShaderServer.recoveryAction` report this
reset. Live Shader handles remain valid. A recovery pause may make events late or
overload the bounded incoming queue; ordinary diagnostics/counters remain active.
Cached decoded image data costs four CPU bytes per pixel, in addition to GPU
storage. Images are limited to 4096 pixels per axis (or the device limit if lower).
Image decode errors keep the requested ID with a small diagnostic placeholder.

Failed reinitialization, a second loss, uncaptured validation/OOM errors, or an
unrecoverable presentation error produce `E_GPU_FATAL` and exit **70**. Ordinary
startup/argument failures exit 1; normal close/quit exits 0. A lost presentation
surface is recreated; another loss before a successful acquisition is fatal.
This is not a guarantee against a stuck driver call or a whole-process crash.
Automatic process restart/scene reconstruction remains later scope.

## Native regression

The test-only feature is absent from normal packaging. Its loss endpoint destroys
only the test process's logical wgpu device. Pixel readback uses a bounded GPU
wait and is also absent from ordinary builds.

```sh
cargo build --manifest-path renderer/Cargo.toml --locked --features gpu-test-hooks
python3 tools/check_gpu_recovery.py --renderer renderer/target/debug/scshader-renderer
# Restore the ordinary feature set before general testing/installing:
cargo build --manifest-path renderer/Cargo.toml --locked
```

The owned-child check verifies exact GPU pixel restoration for a two-pass graph
using images, Float32 data, and typed controls. It deliberately invalidates source
files and removes the image after loading, checks an inactive shader's ongoing
ramp, clock/frame continuity, pending scheduled marker, overlay exclusion from
feedback, missing-image placeholder, and second-loss exit code 70.

Logical-device injection is not a physical GPU/driver reset, and a successful
Metal run is not qualification for Vulkan/DX12 or other hardware. Those remain
native platform gates alongside long-duration acceptance.
