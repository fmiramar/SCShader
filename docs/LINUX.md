# Linux setup

SCShader builds natively for GNU/Linux x64. Use the pinned Rust toolchain and
lockfile. See the [Linux result record](platform-results/2026-09-28-linux-x64.md)
for the tested distribution, drivers, display paths, and remaining checks.

## Prerequisites

- Rustup; `rust-toolchain.toml` selects Rust 1.97.1, rustfmt, and Clippy. Cargo is
  included in the toolchain.
- A native C/C++ toolchain and linker, pkg-config, and Python 3.11 or newer.
- SuperCollider with both `sclang` and `scsynth`, and a working audio backend.
  The receiving CachyOS desktop uses PipeWire's JACK compatibility library.
- Vulkan loader and hardware drivers, plus the X11/Wayland and xkbcommon libraries
  needed for the selected desktop. Distribution package names vary.

Use a writable Linux source directory. Windows build outputs cannot be reused
as Linux executables. Keep the original source and its history when moving from
a shared Windows partition; leave generated Windows build directories behind.

Run `vulkaninfo --summary` in the actual desktop session before GPU checks.
Sandboxed command runners can hide `/dev/dri` and NVIDIA device nodes even when
host drivers work. If no adapter is found, repeat the check with authorized host
device access before changing drivers. Builds and static checks do not require
GPU access.

## Build and install

From the project root:

```sh
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
SCSHADER_ARTIFACT_PREFIX=fmiramar bash tools/package_linux.sh x64
```

The Linux packager uses Python's standard ZIP support and does not require the
external `zip` program. It preserves the renderer's executable permissions in
the archive. Generated packages and checksums are under `dist/`.

Extract the package and place its inner `SCShader/` directory in the directory
reported by `Platform.userExtensionDir` in SuperCollider. Back up an existing
SCShader installation outside Extensions before replacing it. Some ZIP
extractors discard Unix permissions; ensure `SCShader/renderer/scshader-renderer`
is executable. Recompile the IDE class library after installation.

The renderer is discovered automatically from the installed classes. Start with
the installed `ShaderServer` help example or the project guide. Examples use
installed shader assets and also work from unsaved IDE documents.

The local CachyOS build requires GLIBC 2.44, as measured from its ELF version
requirements. It is not an Ubuntu 24.04 compatibility artifact. Build on the
intended older distribution before claiming support for its runtime libraries.

## GPU and display selection

Linux defaults to Vulkan. In SuperCollider, choose a GPU and display connection
before booting a fresh controller:

```supercollider
(
v = ShaderServer.new(\linuxShader);
v.rendererArgs_(["--backend", "vulkan", "--window-system", "wayland", "--adapter", "NVIDIA"]);
v.waitForBoot { |server| [server.backend, server.device].postln };
)
```

Cleanup:

```supercollider
(
v.free;
)
```

Use `"Intel"` to require the Intel GPU, or `"x11"` to require an X11 connection.
Adapter matching must be unique. An unavailable requested adapter or display
fails explicitly. X11 inside the tested Hyprland session uses Xwayland; this is
separate from a native Xorg session. Wayland window placement is controlled by
the compositor, which may ignore global position requests.

## Short verification

For the 0.0.18 resize correction, start with the
[Linux interaction procedure](LINUX_INTERACTION_CHECKS.md). Its Hyprland runner
checks the owned window and GPU allocation after each resize/reload and records
the exact stage and observations on failure. It requires native GPU and compositor
IPC access. The installed 0.0.17 build and its prior evidence remain separate.

Follow the [Linux plan](plans/LINUX.md) and [shared checks](plans/COMMON.md), using
the installed renderer. Keep live checks serial and use fresh evidence folders.
For every acceptance command, explicitly pass `--seconds 10`: the runner's
default duration is one hour. Scripted SC checks can use
`QT_QPA_PLATFORM=offscreen` for the language process; the independent renderer
still opens a real native window on the requested display connection.

Native IDE interaction, listening, additional monitors, sleep/wake, other
distributions and physical driver resets require their own observations. The
result record distinguishes them from automated short checks.
