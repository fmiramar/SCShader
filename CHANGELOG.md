# Changelog

## 0.0.18 - 2026-09-28

- Handle the applied physical size returned immediately by winit resize requests, including Wayland requests that do not generate a later resize event. Resize the GPU surface/feedback/graph targets and notify SC through the existing event path.
- Preserve targets and feedback history when an ignored request or duplicate event reports the same size.
- Add a scoped Hyprland interaction runner with per-stage failure evidence, strict child/window ownership, separate Wayland/Xwayland records, and a check that GPU texture allocations follow actual framebuffer resizing.

This is a development candidate. Static checks and packaging do not replace the pending native GPU/SC regression; the installed 0.0.17 checkpoint remains separate.

## 0.0.17 - 2026-09-28

- Fixed Windows IDE startup: choose FXC explicitly for the supported D3D12 shader subset, avoiding incompatible DXC DLLs discovered in another application's working directory. Startup errors now reach the Windows SuperCollider Post pane.
- Made the OSC boot handshake idempotent: duplicate ready replies no longer replay window state or revive a stopped controller.
- Added explicit NVIDIA/Intel adapter and power-preference selection, corrected Windows executable/path/process handling, and added native package installation with backups.
- Hardened Windows verification: bounded UDP startup retries, native working-set sampling, isolated SC caches, explicit test audio devices, portable reload fixtures, and job-owned process trees that clean descendants after parent exit.
- Made flood health probes precede catch-up bursts while retaining the same offered load and live-response limits; Windows short stress/recovery results and earlier failures are recorded separately.

This remains a development candidate; see the Windows platform results for measured coverage.

## 0.0.16 - 2026-09-25

- Documentation correction: replaced placeholder paths and prerequisite-only snippets in the guide, ten class pages, and README with installed-asset demos and explicit cleanup. Added complete looping-audio analysis/FFT setup and an original spectrum-bar shader. Examples 01–04 now also work from unsaved IDE documents and avoid stale default-controller paths.
- Added verbatim execution of every runnable help/README setup and cleanup, including a deliberately misconfigured default controller. Cleanup clears future Pattern/bus updates before freeing their shader; rendered help alone no longer counts as example-execution coverage. The native renderer binary is unchanged by this correction.
- Added short saved-evidence resource-churn and incoming OSC flood regressions. The 0.0.15 baseline exposed unbounded protocol/command-error replies and logging.
- Added one shared diagnostic token bucket (burst 20, 20/second) across receive/render threads and senders, bounded UTF-8 message bodies, and a `suppressedDiagnostics` status counter. Critical GPU loss/fatal diagnostics bypass the rate limit; successful replies and commands remain outside it.
- Made ordinary soak verification fail on diagnostic suppression, so rate limiting cannot hide errors behind a passing test. Existing status field order is preserved and older renderer replies default to zero suppression.
- Added deterministic full incoming-queue tests for whole-bundle rejection, drop accounting, and admission after draining. Live valid-update floods record observed saturation separately from offered load.

This is development hardening, not final release or one-hour qualification. Native Windows/Linux runtime tests, remaining notice review, and the user's final eight-hour sign-off remain separate work.

## 0.0.15 - 2026-09-24

- Prepared explicit desktop GPU defaults (Metal, D3D12, Vulkan), optional supported backend overrides, and forced Linux X11/Wayland selection. Device recovery retains backend/display choice; GLES receives the compositor display handle.
- Removed the primary-monitor requirement for default fullscreen, allowing the Wayland compositor to choose the display.
- Hardened packaging with OS/CPU/Rust-host checks, explicit target builds, ELF/PE/Mach-O inspection, and portable build hashes. Expanded native Windows/Linux CI unit-test coverage; hosted/native runtime checks remain deferred.
- Made renderer discovery aware of installed/Quark locations, environment/PATH, and Windows executable suffixes. Updated source examples/tests, Windows child termination, SC harness discovery/cleanup, and saved backend/display test requests.
- Added the Windows/Linux native-test handoff. This candidate has not inherited 0.0.14's one-hour qualification; no new long test is automatically launched.

## 0.0.14 - 2026-09-24

- Added a disabled-by-default diagnostic overlay controlled by `ShaderServer.showStats`, with an original bitmap font, no UI/font dependency, bounded text refresh, and a presentation-only pass excluded from feedback.
- Added late-event counts, owned-texture byte estimates, overlay state, recovery count, and last compile result to status replies without moving existing fields.
- Added one controlled in-process GPU-device restart. CPU snapshots restore last-valid shader source, controls/ramps, images/data, graph/bindings, IDs, and clocks; feedback history clears explicitly. Failed recovery, second device loss, and uncaptured GPU errors exit 70. Lost presentation surfaces are recreated.
- Added `ShaderServer.recoveryAction`, explicit image-load diagnostic placeholders, and bounded image dimensions. Decoded RGBA image caching adds CPU memory for recovery; this is documented separately from GPU allocation estimates.
- Added feature-gated logical-device/surface injection and exact GPU readback tests. These hooks are absent from ordinary release builds; logical-device tests do not qualify real driver hangs/resets.
- Reduced Float32 OSC chunks to 2048 values for macOS UDP compatibility. Requested the documented 16K data-texture width where supported and added explicit adapter-limit validation and a full-size buffer smoke.
- Passed the exact-candidate one-hour traffic check and three short trivial/reload/feedback checks. Preserved the user's eight-hour test as final-release sign-off, not an automatic development suite.
- Added hash/revision-validated upstream notice supplements and packaged, explicitly unreviewed target inventories. Context-only licensing discussions do not satisfy missing-text checks; tagged packaging rejects unresolved full-text gaps.

This remains a development candidate. Current acceptance is one hour on the exact build plus short stress/recovery checks. The user will run the eight-hour test as the last final-release validation step; native cross-platform tests and full dependency notices remain separate gates.

## 0.0.13 - 2026-09-23

- Bounded future scheduling to 8,192 commands (including bundle containers), a 16 MiB charged-payload budget, and one hour of lookahead. Status now exposes payload bytes, rejected future packets, and dropped incoming continuous packets; `ShaderServer.clearScheduled` clears future work.
- Preserved OSC bundle atomic dispatch and validated whole packets before enqueue, including nested timing inheritance. Preflight limits packet width/depth before recursive decoding and rejects unsupported OSC arrays. These are ordering/parse guarantees, not rollback of GPU or resource errors at execution.
- Interleaved due and incoming work with a cooperative per-dispatch budget so queued work yields to rendering. One compile or atomic bundle can still exceed that budget.
- Added live queue-limit/recovery stress and SuperCollider cancellation tests. Reworked the soak harness to require measured duration, fresh pongs, continuing frames, error-free traffic, reload acknowledgements, and optional bounded process memory; CSV/JSON evidence includes the owned renderer's binary hash.
- Avoided staging per-frame uniforms before a surface is available and flushed pending data uploads on skipped frames. Added opt-in GPU resource counters and independent native Metal/wgpu allocation probes.
- Isolated visible-frame RSS growth to the tested Metal multipass submission path and added ordered per-pass submissions on Metal without CPU waits or renderer restarts. Added exact GPU pixel-readback coverage for graph/feedback/output ordering. The original one-hour gate failed on memory growth; long-duration acceptance of the workaround remains required.
- Added a locked, offline dependency-notice auditor with exact text hashes and upstream revision records. Missing bundled texts are reported explicitly; notice retrieval/review and final package integration remain open.

## 0.0.12 - 2026-09-23

- Added Naga-based reflection of an optional flat user-control block, with Float32, signed/unsigned 32-bit integer, logical boolean, vector, and square-matrix values. Typed blobs preserve exact bits and GPU layout padding without changing the legacy amount control.
- Added typed `set`/`setn`/`setAt`, asynchronous `get`/`getn`, and per-frame `glide` with step, linear, and smoothstep modes. Validation rejects bad component counts, non-finite/out-of-range values, and non-step integer interpolation.
- Hot reload now watches every live shader, not just the selected one, and preserves matching controls and in-progress ramps across successful reloads. Invalid edits preserve the previous pipeline and schema.
- Fixed a shutdown/restart race: a previous child's delayed termination callback cannot kill its replacement. Boots requested during shutdown are deferred, old Shader handles are invalidated, and clock estimates reset for the new renderer.
- Corrected escaped symbols in older runnable examples and made the example files work from source or packaged layouts.
- Added an original typed-control shader/example, native and SC tests, and a bounded SuperCollider test runner. This remains a development milestone, not completion of the long-soak and cross-platform release gates.

## 0.0.11 - 2026-09-13

- Added a parser-based GLSL 4.50 fragment compatibility path through wgpu/Naga, with an explicit `sourceType` on `ShaderDef` and source/stage-aware compiler diagnostics.
- Added `ShaderToy`, a constrained `mainImage` wrapper with live resolution, timing, frame, mouse, and UTC date built-ins; all `iChannelN` aliases intentionally share the one current source texture.
- Added original GLSL/Shadertoy fixtures and a local Metal smoke covering basic fragments, mouse/date, source-channel sampling, loops, derivatives, and common math. This remains a documented subset rather than universal GLSL or Shadertoy support.
- Added source Quark metadata, deterministic architecture-checked packaging scripts, checksums, explicit multi-platform GitHub Actions builds, and tagged-release automation. The local macOS x64 package is assembled and smoke-tested; no release has been published.

## 0.0.10 - 2026-09-13

- Added renderer-side retention of independent Shader pipelines and uniform state, allowing multiple live Shader resources instead of replacing the selected pipeline on every create.
- Added `ShaderGraph`, a validated connected linear multipass chain with renderer-owned intermediate targets and explicit terminal prior-frame feedback.
- Added graph protocol/parser coverage and a Metal two-pass graph smoke. General branches, DAGs, per-edge formats, and arbitrary feedback edges remain intentionally unsupported.

## 0.0.9 - 2026-09-13

- Added `ShaderBuffer`: bounded Float32 arrays transfer as big-endian OSC blobs into one-row non-filterable `R32Float` GPU textures, with the fixed `spectrum` binding and buffer status accounting.
- Added controlled-rate `ShaderFFTTexture` raw-magnitude and `ShaderWaveformTexture` recent-snapshot streams. Audio-server replies use standard `SendReply`; only language-to-renderer data uses blobs.
- Added parser coverage for buffer blobs, a GPU buffer upload/bind/free smoke, and isolated live FFT/waveform stream smoke coverage. The FFT extractor groups demand streams within SuperCollider's 32-input limit.

## 0.0.8 - 2026-09-13

- Added language-owned `ShaderBus` controls with coalesced mapped updates and optional 60 Hz smoothing; mappings for one Shader are sent as one renderer bundle.
- Added `ShaderAnalysis`, a controlled-rate `SendReply` bridge for amplitude, pitch/confidence, centroid, flatness, onset, and zero-crossing analysis from an explicit mono audio bus.
- Added repeatable local SuperCollider smoke coverage for a private-bus sine source and documented the required Routine/synchronization and source-ordering behavior.

## 0.0.7 - 2026-09-13

- Added initial static PNG/JPEG texture loading through the maintained Rust `image` decoder, a `ShaderTexture` language resource, and the `source` texture binding.
- Added renderer-owned ping-pong render targets for `Shader.feedback(\\previous, amount)` or `\\framebuffer`; targets are cleared and reallocated safely on resize.
- This remains a narrow texture ABI: one static source texture and one feedback source. General texture reflection, video/camera sources, buffers, FFT data, and render graphs remain later work.

## 0.0.6 - 2026-09-13

- Added the standard SuperCollider `\\shader` Event type for `Pbind` Pattern control of live Shaders, ShaderDefs, and registered definition names.
- Pattern uniform updates use timestamped renderer bundles after `latency`, defaulting to `ShaderServer.latency`; `shaderID` and `window` target selection are supported.
- Added `Shader.setAt`, name lookup for live shaders, Pattern help, and an end-to-end Pattern timing smoke. A separate `Pshader` wrapper remains deliberately deferred.

## 0.0.5 - 2026-09-13

- Added `ShaderWindow`, a single renderer-owned native-window controller with startup and runtime logical size, title, position, fullscreen, borderless, VSync, cursor, and front controls.
- Added logical/framebuffer/pixel-ratio metrics plus opt-in normalized mouse, logical keyboard, resize, and focus callbacks. Mouse movement is coalesced to one event per rendered frame.
- Added window protocol coverage, a SuperCollider window smoke, and documentation of the current one-window limitation and runtime monitor-switching boundary.

## 0.0.4 - 2026-09-12

- Added OSC-timetag scheduling: bundle events are mapped once onto the renderer's monotonic clock, ordered by timestamp and packet order, and applied before the next eligible render frame.
- Added repeated ping/pong clock-offset estimation to `ShaderServer`, scheduled-event status counts, a timing-marker reply for measurements, and `E_LATE_EVENT` diagnostics for events more than 50 ms overdue.
- Added timing queue/unit coverage plus SuperCollider timing smoke and benchmark scripts. They measure renderer event-application error, not display scan-out or sample-accurate visual timing.

## 0.0.3 - 2026-09-12

- Added a single active WGSL shader resource with language-side `ShaderDef` and `Shader` objects, resource IDs, typed float control, create/reload/free acknowledgements, and compact reflection for `amount`.
- Added file-change polling for safe hot reload: a new pipeline is validated before replacing the active pipeline, so an invalid edit retains the last valid image and emits `E_SHADER_COMPILE`.
- Added an end-to-end shader-resource lifecycle smoke covering automatic reload, explicit reload, invalid-source fallback, and free.

## 0.0.2 - 2026-09-12

- Added the v1 status request/reply and renderer frame/status counters.
- Added `ShaderServer`, including local executable launching, v1 readiness handshake, process-exit tracking, ping/pong `.sync`, status callbacks, clean quit, and `Cmd-.` cleanup for child processes.
- Added SuperCollider class/lifecycle checks covering boot, status, ping, and quit against the local renderer.

## 0.0.1 - 2026-08-20

- Added the macOS feasibility renderer with a 1280×720 `wgpu` window and an original fullscreen WGSL shader.
- Added a loopback-only OSC receiver, bounded command queue, one float uniform, v1 and Phase-0 message aliases, and machine-readable ready, pong, and error replies.
- Added protocol tests, a SuperCollider round-trip example, invalid-shader recovery fixture, and a configurable 60-minute soak harness.
