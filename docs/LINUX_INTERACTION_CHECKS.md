# Linux window interaction checks

The next Linux gate is real resize/reload/fullscreen behavior. The 0.0.17
Wayland/Xwayland short suites passed, but their ten-second feedback runs ended
before the first scheduled resize. The 0.0.18 resize correction needs its own
native validation; previous binary results do not qualify it.

## Automated Hyprland check

Run from the source directory in a native Hyprland desktop with GPU access.
The runner requires `hyprctl` with the Lua API (Hyprland 0.55+), Python 3.11+,
and the ordinary production renderer. It uses no recovery hooks or additional
Python dependencies. Set `renderer` to the absolute installed candidate path
after backing up the prior extension outside the Extensions directory.

```sh
python3 tools/check_linux_interactions.py --renderer "$renderer" --adapter NVIDIA --window-system wayland --resize-driver compositor --output-dir build/platform-tests/interactions-wayland-nvidia-01
python3 tools/check_linux_interactions.py --renderer "$renderer" --adapter Intel --window-system wayland --resize-driver compositor --output-dir build/platform-tests/interactions-wayland-intel-01
python3 tools/check_linux_interactions.py --renderer "$renderer" --adapter NVIDIA --window-system x11 --output-dir build/platform-tests/interactions-xwayland-nvidia-01
python3 tools/check_linux_interactions.py --renderer "$renderer" --adapter Intel --window-system x11 --output-dir build/platform-tests/interactions-xwayland-intel-01
```

Choose adapters actually present and new output directories. Run commands one at
a time; inspect a failure before starting another. Each command opens one owned
window, floats it, performs four resizes during watched reloads, enters/leaves
fullscreen, toggles the requested decoration state, and closes with exit 0.
Every wait is bounded. `run.json` records the executable hash, actual backend,
adapter/display, completed checks, failure stage and last observations.

The Wayland commands use the approved Hyprland scope: resize through the
compositor. The Xwayland commands use SCShader's OSC resize requests. The
default `--resize-driver osc` remains strict and fails when the compositor
refuses a client resize; it never falls back automatically. Each record names
the driver and sets `osc_resize_completed` only after all four OSC resizes pass.
In Fish, assign a path with `set renderer /absolute/path/to/scshader-renderer`,
or pass the executable path directly after `--renderer`.

The resize check requires both renderer and compositor dimensions, continuing
frames, and GPU texture allocation changing with framebuffer area. This catches
a window whose reported dimensions change while its GPU targets stay stale.
Fullscreen must reach the compositor's actual fullscreen state, not just the
renderer request flag. Decoration checks establish flag handling and continued
rendering; inspect the actual border appearance manually.

The runner starts borderless to avoid confusing client-side decoration geometry
with content dimensions. After making the window floating, it resizes that owned
window through Hyprland once and waits for both compositor and renderer sizes,
frame progress, and changing GPU allocations. This lets the Wayland configure
transition settle before the selected resize path is checked. The runner selects by the
live child PID and exact test title, pins the resulting address, and rechecks
ownership before every compositor mutation. It refuses ambiguous/missing
ownership and never falls back to the focused window or a title-only match. It
changes no compositor configuration. Saved window details are restricted to
the owned process. A PID namespace mismatch must be resolved by running in the
same native session, not by relaxing ownership.

This uses Hyprland's [hyprctl JSON/Lua interface](https://wiki.hypr.land/configuring/core/advanced-configuration/using-hyprctl/)
and [address-scoped window resize dispatchers](https://wiki.hypr.land/0.56.0/Configuring/Basics/Dispatchers/).
An unavailable IPC socket or GPU is a blocked environment check. This helper
does not establish GNOME/KDE behavior, native Xorg, or all-distribution support;
`x11` in this Hyprland session is explicitly recorded as **Xwayland**.

## Why the renderer needed a correction

`winit::Window::request_inner_size` can return an applied physical size immediately
without sending a later `Resized` event. In 0.0.17 that return value was discarded,
leaving surface/feedback/graph resizing dependent on a later event. The 0.0.18
implementation processes the returned size through the same path as an event.
It uses the actual returned size, including a compositor-refused request, and
does not recreate targets for an unchanged size. See the pinned
[winit resize contract](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.request_inner_size).

The earlier saved helper attempts lacked failure-stage diagnostics. The first
0.0.18 attempt reached the client resize and timed out. The updated second run
proved compositor resizing, GPU allocation changes and frame progress, then
confirmed that the first SCShader OSC resize remains at 900x500. Hyprland's
Wayland configure state leaves the pinned winit request constrained in this
session, so native Wayland is recorded with compositor-driven resizing; use the
Xwayland path when SCShader-issued OSC resizing is required. The synchronous
resize correction remains covered by the explicit compositor path and by the
strict Xwayland OSC path.

## Manual IDE check and remaining coverage

Use the installed `examples/09_window_interaction.scd` in the SuperCollider IDE.
Evaluate its setup block, then its cleanup block when finished.

| Action | Expected observation |
| --- | --- |
| Move/click near corners | Animated output and normalized mouse coordinates near 0 or 1 |
| F, then Escape | Fullscreen entry and return to a usable window |
| R | Reload acknowledgement; unchanged shader source keeps the same appearance |
| V twice | Statistics appear and disappear |
| Float and drag-resize | Output follows content size and continues animating |
| Focus another application and return | Focus callbacks and a usable renderer |
| Close/cleanup | Only the example's renderer exits |

Run the installed `ShaderAnalysis` and `ShaderFFTTexture` help demos individually
with working audio, listening and checking visible response, then clean them up.
Record physical input, visual correctness and listening separately from script
results. Minimize/restore where supported, native Xorg, mixed-DPI monitor moves,
sleep/wake and other compositors remain separate checks. The runner neither
simulates these actions nor qualifies them through a successful resize check.

After a passing interaction matrix, repeat the shared SC/help, ten-second
acceptance, short stress and logical-recovery checks on the exact 0.0.18 candidate.
Windows/macOS need regression for the shared resize path. One-hour scheduling
and the user's eight-hour final test remain later gates.
