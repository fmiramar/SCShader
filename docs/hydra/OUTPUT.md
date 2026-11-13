# First Hydra output

2026-11-12. Step 08 implements one `o0` program in the existing renderer window.
The [installed example](../../examples/hydra/01_osc.scd) uses automatic renderer
discovery and works when evaluated from a string without a saved sketch path.
Recompile the SC class library after installing these new classes, then evaluate
the parenthesized example. The native development package has been installed on
the current host; its previous extension is preserved outside Extensions under
ignored `build/backups/`.

`out(\o0, action)` uploads the complete snapshot, validates IR/WGSL, creates a
candidate pipeline and all output/presentation bindings, then installs it before
the next render pass. Only after successful installation does the renderer
increment the revision and reply `committed`. The callback receives `ok`,
`transaction`, `output` and `revision`; errors include `code` and `message`.
Failed or stale updates retain the accepted program and revision. Repeating a
commit returns its recorded result without installing it twice. Replacements
remain selected if `o0` was already selected, and do not reset the session clock.

`h.render(\o0, action)` selects the committed image; `h.render(\legacy, action)`
returns to the existing Shader/ShaderGraph state. No Hydra frame is rendered while
Legacy is selected. Legacy feedback indices likewise freeze while Hydra is
selected. The session clock continues in either mode, starting on session open;
the current defaults are speed 1 and BPM 30. Time/speed/BPM editing comes later.
One control request/upload may be in flight per SC controller. Call the next
operation from its callback rather than issuing overlapping exchanges.

Rendering uses one owned Rgba8Unorm `o0` texture, then a separate presentation
pass. On Metal it follows the existing ordered per-pass submission workaround.
The presenter samples the completed image, handles its Y orientation and compensates
for sRGB surfaces so stored output values are not inadvertently gamma-encoded twice.
The diagnostics overlay follows presentation and is excluded from the output.
Owned texture-byte metrics include `o0`. There is no feedback or `src(o0)` capability
yet; the two-history temporal model remains Step 13 work.

Resize creates a new validated output before replacing the old one. Each axis is
bounded to min(4096, device limit), so one output uses at most 64 MiB; oversized
native-window resizes retain/scalably present the previous resolution and emit a
rate-limited E_HYDRA_LIMIT warning with the retained dimensions. A structured resolution/capability query is still later work.
Ordinary native resize is tested. Device recovery retains the accepted CPU IR,
presentation selection and session clock, reconstructing the output on the new
device; this added path has not been exercised by device-loss injection here.
`h.free` stops its tasks/responders, releases the session/output and selects Legacy;
it does not stop ShaderServer. `v.quit` closes the process/window.

The stage currently supports only o0 and the six initial compiler operators.
Other output slots, feedback, sequences/live parameters, sources, hush and
custom functions are unimplemented. This checkpoint is not Hydra parity.

## Evidence

Native Apple M5 / Metal: successful commit and presentation, rejected edit,
revision-preserving replacement, resize, Legacy/o0 switching and cleanup pass
in the short SC smoke. The existing graph smoke passes. The presenter readback
example verifies all 54 pixels (three 3x3 cases for each of Rgba8Unorm and
Rgba8UnormSrgb), within two 8-bit channel values of independent equations, after
rejected candidates. Both normal installed classes and installed renderer were
used for the final SC checks; the installed example passes automatic discovery.

Initial manual ZIP extraction lost the executable bit and caused boot/fork errors;
restoring the bit resolved them. The ZIP itself correctly stores executable
permissions. Negative logs are retained. No other extension or personal SC
configuration was changed. Manual visual observation, long-running stability,
other backends, and real/device-loss recovery are not qualified by these checks.
See [verification](../VERIFICATION.md) for exact hashes and evidence paths.
