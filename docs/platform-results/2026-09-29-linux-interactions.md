# Linux interaction checkpoint — 2026-09-29

Development 0.0.18, CachyOS x86-64 GNU, Hyprland 0.56.2 and Vulkan. Windows x64
and macOS Apple Silicon/Intel remain release targets. Linux qualification here
is limited to CachyOS/Hyprland; Xwayland is its in-session compatibility path.
No publication or long-duration test was performed.

Production renderer SHA-256:
`eb2b2e642e8ceabf296931e07eba35564294eaa0d2486441120a79f937d81ed6`.
The ordinary release executable has no recovery hooks and is unchanged by the
checker correction below. The final documentation package used by the passing
continuation has SHA-256
`7869866053b57b593a5099b1395c95a3b1b05d57ca41ff11826e27b56ba7e2a1`.
After the checks, a documentation-only package refresh was created with SHA-256
`19bf9ab14f0d5f9d3ccb0808d7186495b6a20d60c2c6387fc480a7f8a251f22c`.
The production executable is byte-identical in both archives and retains hash
`eb2b2e642e8ceabf296931e07eba35564294eaa0d2486441120a79f937d81ed6`.

Native evidence is saved under `build/platform-tests/linux-finish-20260929-021852/`.
The first run is `native-20260929-055119/`; the first passing interaction matrix
is `native-20260929-060354/`. The subsequent `native-20260929-060950/` also passes
all GPU acceptance and SC suites below. Intermediate failures remain preserved.

| Check | Result | Scope |
| --- | --- | --- |
| Wayland / NVIDIA GTX 1050 Ti Max-Q | pass | Compositor resize, four resize/reload cycles, fullscreen, decoration flags, frame/health checks, clean close |
| Wayland / Intel UHD 630 | pass | Same complete compositor interaction suite |
| Xwayland / NVIDIA | pass | OSC resizing at 150% renderer scale, four resize/reload cycles, fullscreen, decoration flags, frame/health checks, clean close |
| Xwayland / Intel | pass | Same complete OSC interaction suite at 150% renderer scale |
| Installation | pass | Follow-up verified all 82 updated package files; the passing candidate remains installed |
| Acceptance / both GPUs / both displays | pass | All 16 combinations of traffic, trivial, reload and feedback; ten seconds each |
| Installed SC suites / both displays | pass | All 25 checks on Wayland and all 25 on Xwayland |
| Help / Wayland | pass | All 18 demos and cleanups |
| Help / Xwayland | pass after demo update | All 18 demos and cleanups; the earlier 60.980 ms late Pattern event remains recorded |
| SCDoc / both displays | pass | Updated guide indexed/rendered without warnings/errors |
| Short scheduling bounds | pass | Command/payload limits, clearing/recovery, atomic/nested scheduling and malformed input |
| Resource/flood stress | pass | 80 resource cycles, bounded protocol/command floods, continuous queue saturation; Intel Vulkan |
| Logical recovery | pass | One logical Vulkan device restart with cached resources; Intel Vulkan |
| Path edge checks | pass | Installed classes/help and renderer from spaces/non-ASCII path |

Both Wayland passes verify that GPU texture allocations follow framebuffer area
through growth and shrink, and that rendering continues after each resize.
They do not establish acceptance of Wayland client-issued OSC resize requests.
The earlier refused OSC requests remain negative evidence; the user approved
compositor-driven Wayland resizing, with Xwayland for OSC window resizing.

## Xwayland checker correction

The failing warm-up requested 900x500 in Hyprland. Its owned window and renderer
framebuffer both reached 900x500, while the renderer reported logical 600x333 at
pixel ratio 1.5. The old checker required logical 900x500 and timed out despite
that framebuffer match. Its saved status sample preceded the resize, so it is
not evidence of stale GPU allocation after that resize.

The checker now compares the appropriate logical or physical dimensions and
accounts for Hyprland's monitor scale and `xwayland:force_zero_scaling`. It
follows the upstream size conversions linked in the
[interaction procedure](../LINUX_INTERACTION_CHECKS.md). It changes no desktop
configuration, retains strict owned-window selection, and still requires OSC
requests to reach their exact logical size, compositor agreement, continuing
frames and healthy GPU allocation changes. Unknown scaling data fails explicitly.

The next run, `native-20260929-060048/`, passed Wayland again but exposed a
scaling-option parser that accepted only integer responses. Hyprland's JSON
interface also supplies boolean responses. Both forms are now handled, with
malformed values still rejected and the received option saved as evidence.
The subsequent native run passed the four interaction combinations, including
`osc_resize_completed: true` on both Xwayland GPUs.

All 23 harness tests pass, including the captured 150% case, independent monitor
scaling, both resize drivers, refused OSC requests, stale compositor geometry
and stale GPU targets. The full Python suite passes 84 of 88 discovered tests;
four native-Windows checks skip. Log:
`build/platform-tests/linux-continue-20260929-vovfUJ/python-final.log`.
These simulated regressions are separate from the native matrix recorded above.

## Acceptance environment follow-up

After the passing interactions the candidate installed with 81 matching file
hashes. The first NVIDIA/Wayland acceptance child failed before handshake:
only Intel was listed as presentation-compatible. Unlike the successful
interaction launches, the local wrapper had removed `DISPLAY` to force Wayland.
The acceptance tools already pass `--window-system` explicitly. Their wrapper
now preserves the native display environment. The subsequent run passed all four
ten-second modes on both GPUs and both displays, including NVIDIA/Wayland, with
the same renderer hash and explicit GPU/protocol selection. The failed renderer
log and wrapper remain with the earlier run.

## Pattern demo timing follow-up

Both installed 25-check SC suites passed, as did all 18 Wayland help demos. The
Xwayland guide Pattern demo reported `E_LATE_EVENT`: one update was 60.980 ms
late. The other 17 Xwayland demos passed. The runner preserved this failure and
restored 0.0.17 before the remaining stages; the next run passed all 18 after
the documented 150 ms lead-time update.

The guide and identical README Pattern demo previously selected the controller's
20 ms default lead time. They now explicitly schedule 150 ms ahead, matching
the existing Pattern smoke test, with the response-delay tradeoff explained.
This is a demo configuration change, not a hard real-time guarantee or a change
to the renderer's 50 ms lateness diagnostic. The help observer still fails on
any renderer diagnostic. Renderer, SC classes, shader assets and protocol files
are byte-identical to the passing runtime matrix.

A focused continuation verified that unchanged payload against the prior run,
installed the new package with a backup, rerendered SCDoc and executed all help
demos on both displays, then ran the remaining short scheduling/resource/flood,
logical-recovery and path checks. In `native-tail-20260929-062528/`, all 18 help
demos and their cleanups passed on each display, and both SCDoc runs passed
without warnings/errors. Scheduling limits also passed. The package
prepared for this continuation has SHA-256
`7869866053b57b593a5099b1395c95a3b1b05d57ca41ff11826e27b56ba7e2a1`.

## Stress baseline follow-up

The same continuation failed all resource/flood modes on Intel/Wayland because
owned texture bytes rose from 7,829,508 to 15,498,260. Resource counts were zero
after cleanup, and flood replies/frame progress remained live. These totals
exactly match the renderer's four RGBA8 window targets plus fallback textures at
952x514 and 945x1025 respectively. The stress runner sampled its baseline right
after handshake and did not record window metrics. Startup compositor resizing
is therefore a strong hypothesis, not a proven leak or a verified correction.

The checker now waits for a stable window size/allocation with frame progress
before capturing its baseline. It records startup and final dimensions, fails
explicitly if geometry changes during fixed-size stress, and retains the exact
post-cleanup texture-byte assertion. It never rebases during a stress workload.
Three regressions cover startup resizing, later geometry changes, real allocation
growth at a fixed size, and a non-progressing startup deadline. The complete
Python suite passes 84 of 88 tests, with four native-Windows skips; log:
`build/platform-tests/linux-continue-20260929-vovfUJ/python-stable-window.log`.

The stabilized native rerun `native-tail-20260929-063408/` passes all four stress
modes, recovery and path checks. Stress ran on Intel Vulkan, with 80 resource
cycles, 3,999 malformed/protocol packets per flood, 511,872 offered continuous
updates, bounded diagnostics, frame progress and exact texture cleanup. Recovery
used the separate hook-enabled debug binary and recorded one logical device
restart followed by the intentional terminal second-loss diagnostic. The path
check compiled classes, ran lifecycle checks and rendered SCDoc from a copied
installation whose path contains spaces and non-ASCII characters.

The tested candidate package remains installed with 82 files matching its staged
package; the documentation-only archive refresh was not rerun as a runtime
change. The installed executable retains the production hash above. This completes the
automated short Linux gate for CachyOS/Hyprland/Vulkan.

## Resume point

The prepared continuation completed in `native-tail-20260929-063408/`. The full
original runner remains available if the runtime payload changes; already passed
suites need not repeat for documentation-only changes.

The user's successful native runs establish working desktop IPC/GPU access.
The agent command session separately returns `Couldn't set socket timeout (2)`
for `hyprctl -j version` and has no `/dev/dri`; this is not a host setup failure.
Native outcomes must come from saved desktop results, not that command session.

Manual input/appearance/listening, minimize/restore, sleep/wake, mixed-monitor
behavior and display-rate observations remain open. The exact 0.0.18 candidate
still needs a separately scheduled one-hour validation; the user's eight-hour
sign-off is the final release gate. Dependency notice review, signing/notarization
decisions, macOS regression/Apple Silicon and any other-platform support remain
outside this Linux short qualification. Nothing has been published.
