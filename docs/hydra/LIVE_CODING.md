# Live output replacement — Step 14

Each accepted `.out(slot)` installs one program and a fresh pair of empty history
images for that slot. Other outputs retain their programs and images. Rejected
compilation keeps the accepted program, history and revision. Acknowledgements
report installation; presentation follows on the next Hydra frame.

Compiled expression pipelines are reused by exact generated WGSL identity among
the four live outputs. Identical live expressions share a pipeline. There is no
inactive pipeline cache; obsolete handles and input bindings are dropped after
replacement, and wgpu retires submitted work safely. History textures and presenter
objects are still replaced; this step retains the documented history reset on
successful edits. Allocation bounds remain those in TEMPORAL_SEMANTICS.md.

`h.clear(\o3, action)` commits transparent black to one output, resets its two
history images and increments its revision. It keeps the current presentation
selection. `h.hush(action)` calls clear sequentially for all four slots and selects
o0. Partial failure stops the sequence. Neither operation releases the output
slot; closing/freeing the session releases all slots and the live pipeline cache.
Selecting another Hydra output keeps rendering every assigned slot. Selecting
Legacy freezes Hydra histories while its clock continues. Existing SC callbacks
report concise commit/revision or error results without an always-on log stream.

The native output helper runs 32 replacements with a rejected edit on every
iteration. It checks unrelated green pixels, bounded paired-image accounting,
shared pipeline identity for matching expressions, and wgpu allocation/ownership
counts returning to their pre-session baseline after disposal. It also retains
the temporal, rollback, resize and presentation regressions. These are short
native Apple M5/Metal checks, not a long-duration or driver VRAM qualification.
The installed output smoke verifies clear's acknowledgement and revision after
hush and redefinition. Exact results and binary hashes are in VERIFICATION.md.

Checkpoint B covers the current procedural/output/feedback/live-edit scope.
Media sources, dynamic parameters and full browser parity remain later work.
