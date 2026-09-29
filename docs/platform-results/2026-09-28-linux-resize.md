# Linux resize follow-up — 2026-09-28

SCShader **0.0.18** development candidate, resumed from the local 0.0.17
checkpoint `22b170d`. This record qualifies static checks and a native build;
it does not claim a new desktop/GPU pass. Source changes are uncommitted.

## Finding and correction

The renderer discarded `Window::request_inner_size`'s immediate size result.
The pinned winit 0.30.13 Wayland implementation can apply the size synchronously
without queuing a `Resized` event, so the GPU surface and feedback/graph targets
could retain their old dimensions. 0.0.18 handles the returned physical size
through the same resize/SC-notification path as `WindowEvent::Resized`. A returned
unchanged size, or a duplicate event, no longer clears/reallocates GPU targets.
No dependency version was upgraded; only the root package version changed in
`Cargo.lock`. The OSC contract and audio-thread separation are unchanged.

The original Linux result called the incomplete helper a discovery failure.
Reinspection of `linux-2026-09-28-01/windows-wayland-NVIDIA-02/run.json` shows an
adapter and `actual_xwayland: false`: that attempt got past discovery/floating
but recorded no completed resize. The old runner did not save the failing stage
or exception. The resize defect is established by code inspection against the
[winit contract](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.request_inner_size);
its causal link to the old live timeout remains unverified.

`tools/check_linux_interactions.py` now provides a maintained Hyprland runner:
strict live child PID/title/address ownership, bounded IPC calls and waits,
Wayland/Xwayland distinction, per-stage failure evidence, four actual resizes
during reload, compositor fullscreen confirmation, decoration request handling,
frame/health checks and clean close. Resize checks also require owned GPU texture
allocations to follow framebuffer area, preventing metrics-only false passes.
The original ignored helper and failed records remain intact.

## Candidate identity

- Native CachyOS x86-64 GNU build, pinned Rust 1.97.1, Python 3.14.7.
- Target: `x86_64-unknown-linux-gnu`; ordinary release build, no `gpu-test-hooks`.
- Renderer SHA-256:
  `eb2b2e642e8ceabf296931e07eba35564294eaa0d2486441120a79f937d81ed6`.
- Cargo.lock SHA-256:
  `99477baa12a417c9c2aee715b1bc27cb59bb40035f7b4855480ef07e5fe762f7`.
- Candidate: `dist/fmiramar-SCShader-0.0.18-linux-x64.zip`, adjacent SHA-256.
  The archive contains this report; its changing archive hash is recorded in
  the external checksum/evidence, not recursively inside this document.
- Source snapshot: `dist/fmiramar-SCShader-0.0.18-source-linux-resume-2026-09-29-r3.zip`,
  with its own checksum and per-file manifest, including uncommitted/new source.
- Existing installed 0.0.17 renderer is retained, SHA-256:
  `19333c08f035850b1ccf3f49c0673a6e3a35e5420bcd3a4072ef482de1ab43b7`.
  The 0.0.18 candidate has not replaced the working installation.

Raw evidence is in ignored `build/platform-tests/linux-2026-09-28-resume-01/`.

| Check | Result | Evidence/limit |
| --- | --- | --- |
| Rust formatting, locked/offline Clippy | pass | `rust-checks.log`; warnings denied, all targets/features |
| Rust tests | pass | `rust-checks.log`; 57 renderer and two overlay tests |
| Python tests | pass | `python.log`; 75 discovered, 71 pass, four native-Windows skips |
| New harness regression tests | pass | Included above: 13 tests cover ownership, IPC errors, saved failure records and stale GPU allocation detection |
| Native release build and notices inventory | pass/incomplete review | `package-01.log`; 152 locked dependencies, no missing text, review still open |
| Final package/source snapshot | pass | `archive-checks.json`; checksums, CRC, file-content hashes, renderer executable mode and source manifest; `package-final.log` refreshes the candidate documentation |
| ELF/dependencies | pass | `elf.txt`; x86-64 GNU, GLIBC 2.44, linked system libraries resolved; no older-distribution claim |
| CLI version and executable hash | pass | `build-identity.json`; 0.0.18 and hash above |
| Native interaction preflight | blocked | `interaction-preflight/run.json`; `hyprctl version` fails with `Couldn't set socket timeout (2)` before any renderer launch |
| GPU device access | unavailable | `build-identity.json`; this command sandbox has no `/dev/dri` or NVIDIA node |
| 0.0.18 SC/help, native interaction/acceptance/stress/recovery | not run | Requires native desktop/device access; 0.0.17 passes do not transfer |
| Other native OS targets, Xorg, other compositors/distros, manual input/listening | not run | Separate gates |

The unit tests use simulated compositor/measurement records, not GPU hardware.
No diagnostic assertion was weakened and no software rendering substituted.
No one-hour run, eight-hour sign-off, driver reset, hosted workflow, signing or
publication occurred.

## Resume point

In a native desktop with GPU and compositor access, back up/install the 0.0.18
candidate through the shared plan, then run the
[Hyprland interaction matrix](../LINUX_INTERACTION_CHECKS.md). If it fails, use
the saved stage and window/OSC observations to reduce the failure. Keep a failed
candidate separate from the retained 0.0.17 package.

After interactions pass, rerun installed SC/help, four ten-second Vulkan modes,
short scheduling/resource/flood tests and a separate hook-enabled recovery build.
Record exact hashes and per-GPU Wayland/Xwayland results. Native Xorg/manual
coverage and Windows/macOS regression remain open. Schedule a one-hour test only
after the supported-platform candidate is qualified; the user owns the final
eight-hour sign-off.

## First user-run Wayland interaction attempt

The host's Hyprland IPC and both hardware Vulkan adapters are accessible from
the user's interactive shell (Hyprland 0.56.2; Mesa Intel UHD 630 and NVIDIA GTX
1050 Ti Max-Q, driver 580.178.04). NVIDIA/Wayland launched the expected 0.0.18
binary hash and successfully completed owned-window discovery, floating,
handshake, initial OSC/window metrics, shader creation and initial status.
The first requested 640x360 resize timed out: the renderer and compositor still
reported 952x514; texture bytes remained 7,829,508. Evidence is
`build/platform-tests/linux-resume-wayland-nvidia-01/run.json` and
`renderer.log`. The run reported the window floating, but that property alone
does not prove winit has received the Wayland configure reflecting the transition.
The runner now requests and verifies a compositor-driven resize of the same
owned window before testing a selected resize path. The second native run
completed that compositor resize on NVIDIA/Wayland, changed the owned GPU
allocation and continued rendering, then timed out when the first SCShader OSC
resize remained at 900x500. This is consistent with the pinned Wayland window
state refusing client size requests while the compositor still supplies the
tiled configure state. The accepted Linux scope records native Hyprland Wayland
with compositor-driven resize; Xwayland remains the path for SCShader-issued OSC
resize requests. Both runs and their negative evidence remain in `build/`.
