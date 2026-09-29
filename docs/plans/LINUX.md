# Linux agentic plan — x64 first, optional native arm64

Release scope: CachyOS x86-64 GNU under Hyprland with Vulkan; Xwayland is an
in-session compatibility path. Other distributions, compositors, native Xorg,
arm64 and GL are deferred.

Status: the 0.0.17 native CachyOS x64 package, Wayland/Xwayland Vulkan matrix,
SC/help and short stress/recovery checks passed. The 0.0.18 synchronous resize
correction needs native revalidation; see the
[resume record](../platform-results/2026-09-28-linux-resize.md). Follow [COMMON.md](COMMON.md).
This is the user's selected next platform after Windows interaction checks;
read the [0.0.17 handoff](../LINUX_HANDOFF.md) for the carried changes.

## L1. Establish the native environment

Use an interactive GNU/Linux desktop with SuperCollider, Python 3.11+, rustup,
a native C/C++ linker/toolchain, pkg-config, and the selected display/graphics
driver libraries. Inspect build failures and install the distribution's actual
required development packages; do not blindly copy another distribution's package
names or use a headless CI session as a presentation test.

Record distribution/version, kernel, `uname -m`, `rustc -vV`, SC version, GPU/driver,
`XDG_SESSION_TYPE`, compositor/window manager, and session availability. Record
display protocol names, not personal socket/user paths. Set:

```sh
target=x86_64-unknown-linux-gnu
package_arch=x64
```

For the optional **native arm64** follow-up only, use
`target=aarch64-unknown-linux-gnu` and `package_arch=arm64` on an actual arm64 GNU
host. Use a separate result record/archive; do not generalize x64 results. Musl,
32-bit Linux, and generic all-distribution compatibility are not claimed.

## L2. Link, package, and inspect runtime requirements

Run COMMON static checks, then:

```sh
export SCSHADER_ARTIFACT_PREFIX=fmiramar
python3 tools/package_support.py --target "$target" --check-host
bash tools/package_linux.sh "$package_arch"
```

Inspect the resulting ZIP/checksum and `build-info.json`. On the locally built
trusted binary inspect `file`, `readelf -h`, `readelf -d`, and
`readelf --version-info`; use `ldd` only for the executable you just built. Record
interpreter, needed shared libraries, actual required GLIBC versions, and any
missing system libraries. The configured CI target is Ubuntu 24.04, not a proven
minimum distribution. Fix packaging or document the tested baseline accurately;
do not claim older glibc support from successful linking on a newer distribution.

Install the extracted extension, set `extension`, `renderer`, and `sclang` to the
installed paths, then run COMMON SC/help checks. Preserve installed executable
permissions and test launch with spaces/non-ASCII paths. If audio tests cannot
boot, record/configure the receiving SC audio environment rather than suppressing
the audio assertions or changing unrelated system services without approval.

## L3. Separate display/backend qualification

Current resume point: run the [Hyprland interaction checks](../LINUX_INTERACTION_CHECKS.md)
on the candidate in a native desktop with GPU/IPC access, then repeat the critical
suites below. The 0.0.17 ten-second feedback modes did not reach their scheduled
resize, so they cannot close this gate. Native Xorg remains a separate open track.

Run these in sessions where the requested display protocol is actually available:

```sh
python3 tools/run_acceptance.py --renderer "$renderer" --backend vulkan --window-system x11 --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/linux-vulkan-x11-01
python3 tools/run_acceptance.py --renderer "$renderer" --backend vulkan --window-system wayland --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/linux-vulkan-wayland-01
```

X11 under Xwayland is distinct from native Xorg; record which was used. Repeat the
critical installed SC/help suite in the relevant desktop sessions. Selection is
explicit and must not silently fall back. If a required session is unavailable,
mark that path blocked and resume there later or obtain an explicit narrower
support decision. Default-session SC checks do not prove forced display coverage.

Run COMMON short stress/recovery on the default backend/session and record its
identity. Recovery debug binary:
`build/recovery-target/<target>/debug/scshader-renderer`.
An optional `--backend gl` acceptance run uses a separate result/output directory;
GL is experimental and its shader/adapter limits may differ from Vulkan.

Investigate in order:

1. Vulkan instance/adapter/surface setup, retained display handle during recovery,
   native package dependency loading, and clear missing-driver diagnostics.
2. Wayland fullscreen without a primary monitor, logical/framebuffer scale,
   resize/configure, input coordinates, focus and minimized/surface states.
   Wayland may ignore global window positioning: document actual semantics and
   correct platform assumptions in tests without removing observable checks.
3. X11 resize/event-loop/monitor behavior, including Xwayland differences.
4. SC child-process cleanup/reboot, audio/FFT streaming, shader reload, graph and
   feedback ordering, overlay isolation, suspend/resume and different refresh rates.
5. A second driver/vendor or GPU where available; do not substitute software Vulkan
   for a native GPU claim. Hybrid-GPU selection needs its own explicit record.

## L4. Exit gate

GNU x64 linked/installed package and Vulkan critical suites pass; X11/Wayland have
separate evidence or a user-approved narrowed support scope. Dependencies/minimum
tested distribution and missing hardware are explicit. Optional GL and native
arm64 do not silently inherit a pass. Return fixes/results for other-target
regression; leave long tests and publication to their later authorized milestones.
