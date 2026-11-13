# Four outputs and static sampling — Step 12

Step 13 supersedes the static scheduling, cycle rejection and single-image memory
figures below with [paired temporal outputs](TEMPORAL_SEMANTICS.md). This document
retains the Step 12 checkpoint and ordered visual diagnostic instructions. The user
now confirms hush produces a full black window and invert(0.5) produces gray in
the bottom-right quadrant; both observations are expected. This is not a claim
that every item in the nine-step visual checklist was observed.

Four independent `o0..o3` programs and revisions now share one renderer session.
`expr.out(\oN, action)` replaces only its destination after validation/compilation.
`h.render(\oN)` selects one image; no-argument `h.render` shows all four.
The pinned Hydra `_render` selects all when no output is supplied; its renderAll
shader uses column order: o0 top left, o1 bottom left, o2 top right, o3 bottom right.
The native layout follows that order. `h.render(\legacy)` retains the legacy
SCShader graph. Default render changed from the first slice's o0 to quadrants.
Display resources are prepared before transport mode/sequence changes, so a
preparation failure retains the previous selection. Presentation changes retain every output program and the session clock. All
assigned outputs render while any Hydra view is selected. Legacy mode freezes
Hydra image updates; returning to Hydra uses the continuing session clock.

`h.src(\oN)` samples an output at wrapped coordinates with linear filtering,
clamped edges, and an explicit Y correction for native render texture storage.
Output references use HYIR tag 4 followed by one u8 slot (0..3), as in the ADR.
The native convenience default is o0. No media/source-slot binding is enabled.
WGSL texture sampling uses a separate group 1; purely procedural shaders retain
the previous uniform-only group 0 ABI. Sampling can be nested or transformed like
other source expressions, with intermediate RGBA left unclamped until storage.

## Deliberate Step 12 boundary

Only acyclic one-way references are accepted. A candidate replacement is checked
against the complete four-output graph before touching the accepted state.
Producers render before consumers, even when output numbers are reversed. An
unassigned output samples transparent black; committing it later updates dependent
bindings. No pass binds its own destination image, even for unused shader texture
declarations. Each producer pass ends/submits before consumer passes, retaining
the existing Metal submission boundary. Self references and multi-output cycles
fail with E_HYDRA_COMPILE and retain programs/revisions/images.

This is a provisional static-dependency mode, not the temporal contract in the
ADR. Same-frame topological ordering is a deliberate native Step 12 restriction;
it does not claim Hydra's numeric output-order/feedback parity. Step 13 must
reconcile the pinned temporal behavior and replace this mode with safe buffering.
`prev` and self-feedback are still unavailable. `src` in the inventory remains
partial because source/media sampling is later work.

## Ownership and bounds

Each assigned output owns one RGBA8Unorm target, frame uniform and pipeline.
Target dimensions are positive, at most 4096 per axis and within device limits.
Four maximum-size color targets consume 256 MiB plus a shared four-byte blank.
Replacement can transiently retain five targets (320 MiB plus blank); atomic
resize can retain both generations (512 MiB plus two blanks). These are bounded
color-image estimates, excluding driver/swapchain and existing legacy resources.
The renderer's texture-byte accounting includes every owned Hydra target.
Targets and input bind groups are created on edits/resize, never each frame.

Resize builds a complete replacement set before installing it; a limit failure
retains the previous set and reports a bounded diagnostic. Programs and accepted target dimensions are retained during
resize/device reconstruction (including a rejected oversized resize), while target images start blank and are
recomputed before presentation. The reconstruction path was updated but recovery
injection is not qualified by this step. Closing/free releases the whole set and
returns to Legacy. Blank presentation does not require a prior commit.

`h.hush(action)` performs four acknowledged transparent-solid replacements in
slot order and selects o0. Revisions increment once per replacement and time
continues. This SC convenience is sequential, not an atomic four-output edit;
on failure it stops and reports the partial result. Targets are retained for
continued use; `free` releases them. No new OSC command is introduced for hush.

## Evidence and use

`examples/hydra/02_four_outputs.scd` now provides an ordered visual diagnostic.
Run Step 0 to stop a previous copy and wait for STOPPED, then evaluate startup.
Run Steps 1–9 one at a time, waiting for ACKNOWLEDGED and inspecting each result:

1. Opaque quadrant identifiers: red o0 top left, green o1 bottom left, blue o2
   top right and yellow o3 bottom right.
2. Single-output o2: full-window blue.
3. Restore all four original animated programs and quadrants.
4. Sample o0 into o3 with invert(1): visible scaled inverted stripes, bottom right.
5. Repeat with invert(0.5): flat mid-gray in o3, as required by the formula.
6. Hush: transparent black in full-window o0; record its actual window appearance.
7. Opaque black: the full window should be black.
8. Opaque white: the full window should be white.
9. Opaque black again, for comparison with transparent Step 6. Clean up with Step 0.

Startup and each step print acknowledgements, while the runner rejects skipped
or overlapping steps. An acknowledgement means the commands were accepted;
visual PASS still requires observation. A gray Step 6 followed by correct opaque
black/white changes distinguishes the symptom from a failure to replace the
visible image, but does not by itself establish the compositor's cause. Record
unexpected quadrant locations and Step 6–9 colors rather than assuming a pass.
The installed example check executes all nine diagnostic functions serially on
its own port, with a display interval between steps and a skipped-step rejection
check. It verifies command completion/order; it does not measure desktop pixels.

Native Apple M5/Metal tests use production output, compiler and presentation code:
blank images, exact quadrant placement, single-output switching, gradient source
orientation, nested reads, CPU-checked bilinear filtering, reverse dependencies,
cycle rollback, retained state and resize/accounting pass on linear and sRGB
targets. All fifty previous procedural/composition/modulation numerical cases
still pass. SC tests cover output tags/slots, per-output commits/revisions, blank
selection, static sampling, switches, cycle rejection and hush. The transport
unit test checks all four revisions and retried commit destination identity.
See [VERIFICATION.md](../VERIFICATION.md) for exact build/hash and installed scope.
The user confirms Step 11 works perfectly. Remaining four-output manual observations,
other platforms and complete browser parity remain open. Temporal feedback is now
covered by [Step 13](TEMPORAL_SEMANTICS.md).
