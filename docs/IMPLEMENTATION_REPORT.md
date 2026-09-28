# SCShader: implementation, architecture, verification, and remaining work

Architecture baseline: 2026-09-25, development **0.0.16**, including the installed-example correction. Current development is **0.0.17**; see [STATUS.md](STATUS.md) and the [Windows corrections record](platform-results/2026-09-28-windows-x64.md) for newer native evidence. This is a working development system, not a completed v0.1.0 release.

This report covers SCShader only. FMMatrix, PMMatrix, and the other projects in the surrounding workspace have separate implementations and release requirements. It distinguishes implemented features, measured verification, outstanding release work, and intentionally deferred features.

## 1. What SCShader is

SCShader lets SuperCollider control GPU-generated visuals using familiar objects, controls, clocks, and Patterns. SuperCollider remains the composition environment; a separate native application owns the graphics device and window.

It is **not a GPU UGen running inside the audio server**. No rendering, shader compilation, window handling, or graphics-driver calls are added to `scsynth`'s real-time audio callback. Audio analysis uses ordinary audio-server Synths, whose results reach the visual system through the language process.

The implemented core includes live shader creation, typed controls, interpolation, timed Pattern updates, hot reload, static images, feedback, linear multipass graphs, audio analysis, FFT/waveform streams, window/input control, diagnostics, and limited device recovery. The macOS x64 package is installed and locally tested. Windows/Linux preparation exists, but native runtime qualification remains pending.

Process separation reduces the consequences of an ordinary renderer crash: it does not require terminating the audio server. It does not eliminate competition for CPU/memory or guarantee that an operating-system or graphics-driver failure cannot affect the wider machine.

## 2. Architecture and data flow

```text
SuperCollider IDE / sclang
  | composition, Shader objects, Pbind, clocks, resource ownership
  |
  +---- SynthDefs / audio control --------------------> scsynth
  |                                                      |
  |<--- SendReply: analysis, FFT, waveform snapshots -------+
  |
  +---- localhost UDP/OSC commands and typed blobs ---> Rust renderer
  |                                                      |
  |<--- ready, status, reflection, errors, input events ----+
                                                         |
                                                  wgpu + native GPU API
                                                         |
                                                   native visual window
```

### Language-side control

The installed extension supplies twelve public classes:

| Class | Responsibility |
| --- | --- |
| `ShaderServer` | Owns the renderer child process, handshake, OSC connection, clock estimate, status, diagnostics, and shutdown. |
| `ShaderWindow` | Configures one renderer-owned window and its supported runtime/input properties. |
| `ShaderDef` | Describes a shader source file, source language, name, and optional declared controls. |
| `Shader` | Represents a live shader resource: typed values, glides, queries, reload, bindings, and freeing. |
| `ShaderToy` | Describes an original or otherwise appropriately licensed fragment using the supported Shadertoy-style wrapper. |
| `ShaderTexture` | Loads a static image for the current source-texture binding. |
| `ShaderBuffer` | Transfers bounded Float32 arrays into a GPU data texture. |
| `ShaderBus` | Supplies mapped visual-rate scalar values, coalescing updates and optionally smoothing them. |
| `ShaderAnalysis` | Extracts amplitude, pitch/confidence, centroid, flatness, onset, and zero-crossing information from a mono audio bus. |
| `ShaderFFTTexture` | Streams raw FFT magnitudes from an audio bus into a `ShaderBuffer`. |
| `ShaderWaveformTexture` | Streams bounded recent waveform snapshots into a `ShaderBuffer`. |
| `ShaderGraph` | Validates and commits a connected linear multipass chain, with a restricted feedback arrangement. |

`Pbind` uses the registered `\shader` Event type. A separate `Pshader` wrapper was not necessary for the implemented Pattern integration. A named shader still requires a real registered definition; names in the original design sketches are not automatically installed presets.

### Transport and threading

The renderer listens on loopback UDP, not on a public network interface. Protocol messages use the `/scshader/v1/` namespace; temporary feasibility aliases remain for the original low-level interface.

An OSC receiver thread checks packet shape, decodes commands, and admits whole packets to a bounded queue. The window/render event loop consumes commands, manages resources, schedules future work, and submits GPU work. Successful replies and structured errors travel back to the language client.

The important bounds are:

- 256 incoming packets. Continuous-update packets can be dropped under overload; drops are counted. Non-continuous lifecycle work waits for queue space.
- At most 256 OSC packet elements and 16 nested bundle levels per datagram.
- Future scheduling: 8,192 charged commands, including bundle containers, a 16 MiB charged-payload budget, and one hour of lookahead.
- A malformed bundle is rejected before any earlier valid member can execute. This is parse/admission atomicity, not rollback of resource or GPU errors during execution.

Dispatch is cooperative so command processing yields to drawing. A single expensive compilation or atomic bundle can still take time. These limits do not impose a global memory quota on every live shader, image, or driver allocation.

### Timing

OSC bundle timestamps are translated once into the renderer's monotonic clock. Future work is ordered by deadline and packet order. Repeated ping/pong samples provide the language-to-renderer clock estimate; timed markers measure command application.

Visual timing is frame-oriented, not audio-sample accurate. Applying a command is also not the same measurement as the display scanning out a pixel. Events more than 50 ms overdue produce a late-event diagnostic. The renderer interpolates glides between control updates, avoiding a requirement for the language to send every animation frame.

### GPU resources and shader layout

Each live shader retains its own validated pipeline and control state. WGSL is the native language. Naga parses supported WGSL/GLSL and supplies real type/offset reflection; layout is not guessed from the order of SuperCollider Event keys.

The current resource interface uses group 0 bindings for built-in uniforms, one source image, its sampler, one non-filterable Float32 data texture, and an optional custom-control uniform block. The custom block is flat and bounded: up to 127 members, 64 UTF-8 bytes per name, and 16 KiB of uniform storage. Nested structures, arbitrary arrays, and unrestricted GPU-resource reflection are not implemented.

The ten control kinds are float, signed integer, unsigned integer, logical boolean, `vec2`/`vec3`/`vec4`, and `mat2`/`mat3`/`mat4`. Binary OSC payloads preserve integer bits and tightly packed component values; the renderer adds GPU layout padding, including the different `mat3` column stride. Logical booleans use annotated unsigned storage. Large unsigned values need integral Float literals in SuperCollider because its Integers are signed 32-bit.

`set`, `setn`, `setAt`, `get`, `getn`, and `glide` are implemented. Floating controls support step, linear, and cubic smoothstep interpolation; integer/boolean controls use step changes. Matrix interpolation is component-wise, not a rotation-aware decomposition. See [the exact control contract](CONTROLS.md).

### Images, feedback, graphs, and audio data

Static PNG/JPEG/PNM images are decoded and uploaded. Images are bounded to 4096 pixels per axis, or the adapter limit if smaller. Failed image loading is reported and retains an explicit diagnostic placeholder rather than silently substituting unrelated user content.

Feedback uses alternating render targets, so a pass does not read and write the same texture simultaneously. History begins black and resets on resize or device recovery. A graph currently forms one connected linear pass chain; general branches, DAGs, arbitrary edge formats, and arbitrary feedback edges are not supported.

Float arrays become one-row `R32Float` textures with exact, non-filtered sampling. Buffers are capped at 16,384 values; outgoing data is chunked to avoid oversized macOS UDP datagrams. FFT data contains raw magnitudes, not phase, mel bands, or chroma. Waveform streams are visual-rate snapshots, not an audio-rate shared-memory oscilloscope.

Analysis and stream Synths run after ordinary source Synths at the audio server's default-group tail. Buffer/SynthDef preparation and stream startup must synchronize before consumers start. The corrected examples provide that ordering explicitly.

### Error handling, monitoring, and recovery

All live file-backed shaders are watched four times per second. Replacement source and reflection must validate before replacing a pipeline. Invalid edits keep the last valid pipeline, controls, and reflection. Successful reload preserves compatible control values and glides; new or type-changed fields reset as documented.

The optional statistics overlay is off by default and is drawn only for final presentation, outside feedback history. It uses an original small bitmap font rather than adding a UI/font framework. CPU frame timing and owned-texture estimates are available; GPU execution timing is explicitly unavailable, and the texture estimate is not total VRAM usage.

Ordinary structured diagnostics and their stderr entries share a fixed-storage token bucket: a burst of 20, then 20 per second. Suppressed diagnostics are counted in status; message bodies are capped at 4096 UTF-8 bytes plus a marker. Critical GPU loss/fatal diagnostics bypass the budget. Successful replies and commands are not rate-limited by this mechanism.

The renderer can recreate a lost presentation surface and attempt **one in-process GPU-device restart**. CPU snapshots restore last-valid source, values/ramps, decoded images, data buffers, bindings, graph selection, IDs, clocks, and relevant window state. GPU feedback/intermediate history cannot be restored and clears to black. Recovery is reported to the language.

A failed recovery, second device loss, or fatal GPU error exits with code 70. This is not automatic reconstruction after a complete process crash, nor a guarantee against a stuck driver call. See [the recovery contract](RECOVERY.md).

## 3. What was used

These are the versions pinned in this project, not a claim about the newest available upstream releases.

| Component | Pinned version / role |
| --- | --- |
| SuperCollider language and audio server | Language integration, Patterns, clocks, SynthDefs, FFT/analysis, and `SendReply`; local tests used SuperCollider 3.14.1. This is not a verified minimum-version claim. |
| Rust | Toolchain 1.97.1, edition 2024; native renderer implementation. |
| `wgpu` | 30.0.0; GPU abstraction and rendering. |
| Naga | Used through the wgpu ecosystem for parsing, validation, GLSL compatibility, and control reflection. |
| `winit` | 0.30.13; native window creation and desktop events. |
| `rosc` | 0.11.4; OSC encoding/decoding. |
| `image` | 0.25.8 with PNG/JPEG/PNM support; static image decoding. |
| `bytemuck` | 1.25.2; conversion of GPU data structures to bytes. |
| `pollster` | 1.0.1; resolving one-time asynchronous GPU initialization/error-scope work. |
| Python | 3.11+; standard-library verification, process ownership, package checks, stress/soak tools, and notice auditing. |
| Shell / PowerShell / GitHub Actions | Native platform packaging and configured CI/release workflows. |
| SCDoc | Indexing and rendering the installed help pages. |

Direct Rust versions are exact in `renderer/Cargo.toml`; `renderer/Cargo.lock` fixes the transitive graph. No new dependency was needed for typed reflection, interpolation, or the overlay. An independent Objective-C Metal probe and Rust GPU probes were development investigation tools, not a second application runtime.

The platform defaults are Metal on macOS, D3D12 on Windows, and Vulkan on Linux. Explicit supported alternatives include Vulkan on Windows and an experimental OpenGL path on Linux. Linux can force X11 or Wayland. There is no silent change to another graphics API when an explicitly selected backend fails.

SCShader is an original implementation of the project's architecture plan using these libraries, not a wrapper around another complete visual application. Shipped shader fixtures are original. The supported Shadertoy wrapper supplies familiar inputs and `mainImage`, but all four `iChannelN` aliases currently refer to the same source texture; it is not universal Shadertoy compatibility. Some SC-specific compatibility inputs are placeholders rather than automatic audio analysis.

The project declares GPL-3.0-or-later. Dependency declarations, upstream references, and the unresolved notice work are documented separately in [DEPENDENCIES.md](DEPENDENCIES.md) and [NOTICE_AUDIT.md](NOTICE_AUDIT.md). A dependency declaration or text-presence check is not a completed distribution review.

## 4. Implementation history

| Development versions | Work completed |
| --- | --- |
| 0.0.1–0.0.2 | Feasibility GPU window, original fullscreen shader, loopback OSC, bounded incoming queue, protocol replies, process ownership, handshake/status/ping/quit, and initial testing. |
| 0.0.3–0.0.6 | Shader resources and safe reload, timestamped scheduling and clock estimation, native window/input controls, and normal `Pbind` integration. |
| 0.0.7–0.0.10 | Static textures and feedback; visual buses and audio analysis; Float32/FFT/waveform streaming; independent shader state and linear multipass graphs. |
| 0.0.11 | Parser-based GLSL and constrained Shadertoy support, Quark metadata, architecture-specific packaging, checksums, and release automation configuration. |
| 0.0.12 | Typed reflection, exact serialization/layout, glides and queries, all-live-shader reload, and fixes for stale shutdown timers and restart state. |
| 0.0.13 | Bounded future scheduling, packet validation/atomic admission, scheduling stress, strict measured soaks, memory investigation, Metal submission workaround, and notice-audit tooling. |
| 0.0.14 | Overlay, expanded status, logical GPU-device recovery, surface recreation, placeholders, readback tests, data-transfer corrections, pinned notice supplements, and completed exact-candidate one-hour validation. |
| 0.0.15 | Windows/Linux backend/display implementation, Wayland fullscreen behavior, portable executable discovery and process cleanup, explicit package target/header guards, and native-test handoff. |
| 0.0.16 | Resource churn and incoming/error-flood testing, diagnostic rate/message limits and suppression accounting, deterministic full-queue tests, and corrected installed examples with complete execution coverage. |

The detailed sequence, including corrections to earlier implementations, is in [CHANGELOG.md](../CHANGELOG.md).

### Important problems found and corrected

**Metal memory growth.** An earlier one-hour test failed after about 410 seconds because RSS exceeded the growth limit. Reduced experiments isolated the problematic behavior to the tested wgpu/Metal multipass submission path. The workaround submits passes separately on Metal while retaining ordering; it does not add per-frame CPU waits or periodically restart the renderer. Exact GPU pixel tests checked graph/feedback ordering. Later isolated one-hour tests passed. This is a measured workaround, not proof of an upstream bug on every device. See [MEMORY_INVESTIGATION.md](MEMORY_INVESTIGATION.md).

**Unbounded error output.** The 0.0.15 error floods produced roughly 4,000 replies and log entries in two seconds. The 0.0.16 diagnostic budget reduced each final error-flood case to 59 replies/log entries, with 3,939 suppressed and counted. Rendering and health replies remained responsive. Ordinary soaks now fail on suppression so rate limiting cannot hide unexpected errors.

**Examples that rendered as help but did not run.** User reports exposed fake renderer/shader paths, undeclared `audioBus`/`fft` names, missing graph prerequisites, and a stale default-controller override. The earlier smoke only executed the first guide block and substituted document paths for file examples. Rendering help was insufficient evidence.

The correction provides complete setup and cleanup in the guide, ten class pages, and README; fresh controllers avoid inherited bad paths. Audio demos include a looping file, bus, playback Synth, synchronization, and stream setup. An original spectrum-bar shader makes FFT data visible. Files 01–04 work when copied into an unsaved document. Cleanup stops producers and clears their scheduled work before freeing target shaders. The native renderer executable was not changed by this documentation/assets correction.

## 5. What has actually been verified

The current recorded local evidence includes:

- 55 Rust unit tests and warnings-as-errors Clippy for the renderer checkpoint.
- 49 Python tests after adding help-example validation and eight portable source-handoff tests; the complete suite also passes from a fresh source-ZIP extraction.
- All 24 installed SuperCollider checks passed at the renderer checkpoint. The modified example-file checks and SCDoc were rerun after the documentation correction.
- All 18 help/README demos and 18 cleanup blocks executed verbatim: **36 blocks**, with installed assets, no active document path, and a deliberately bad path left on the default controller. Four standalone example-file setup blocks also passed without path substitution.
- All twelve class help pages and the project guide indexed/rendered without SCDoc warnings/errors.
- Eighty resource create/bind/render/free cycles returned live counts and owned-texture bytes exactly to baseline; measured RSS growth after warmup was 1,180 KiB.
- Both short error-flood cases passed their reply/log bounds and responsiveness checks. A valid-update run offered 511,616 updates in about two seconds without observed application queue drops.
- Future-scheduler saturation/recovery passed. Full incoming-queue admission is tested deterministically; the live valid-update run did **not** fill that queue.
- Logical-device injection/readback checked restored pixels/resources, clocks, ramps, scheduled work, overlay isolation, surface recreation, and second-loss exit behavior.
- Four three-second 0.0.16 Metal smoke modes passed. These are short regressions, not one-hour or repeated-resize qualification.
- Windows x64 MSVC and Linux x64 GNU all-target/all-feature checks compiled/type-checked on macOS. They did not link or execute native Windows/Linux applications.
- The local macOS x64 archive passed target/header, checksum, ZIP-integrity, and system-library checks. A previous archive was preserved before replacing its broken documentation.

The ordinary current renderer SHA-256 is:

```text
f9a16109461cac1d8c56ea698fd53820d6013866d5739ae9f29f1d231ec82d0b
```

The completed 0.0.14 one-hour run used a different exact binary: 3,600,000 updates in 3600.005 seconds, continued frames/replies, no queue drops, and 11.1 MiB post-warmup RSS growth. **That pass does not qualify 0.0.16.** Failed/incomplete/overlapping earlier runs remain negative evidence rather than being relabeled as passes.

Evidence also has boundaries: sent UDP traffic is not individually acknowledged execution; application counters do not measure kernel UDP loss; RSS is not VRAM; submitted frames are not display scan-out; a logical-device test is not a real driver reset. Full details and saved-result locations are in [VERIFICATION.md](VERIFICATION.md).

## 6. What remains, and why it matters

### A. Broader lifecycle and resource stress — can continue locally

Test repeated resize/minimize/restore, repeated valid and invalid reloads, graph/feedback resizing, and larger mixed resource workloads. Exercise interactions, not only each API in isolation. Seek actual incoming-queue saturation on a live renderer while distinguishing it from kernel packet loss.

**Why:** reallocating GPU targets, replacing pipelines, stopping streams, and freeing resources can interact in ways short happy-path checks miss. The newly discovered cleanup race is an example. Queue bounds alone do not prevent excessive total live-resource memory; larger workloads should inform whether aggregate quotas or clearer rejection policies are needed.

**Completion evidence:** bounded memory/resource counts, continued frame/health replies, expected rejection diagnostics, clean recovery, and no messages targeting freed resources. Do not equate sending a high packet count with proving saturation.

### B. Complete dependency notices and review — can largely continue locally

Ten macOS full-text gaps remain: `dispatch` and nine objc2-family packages. The macOS inventory covers 132 packages, including relevant build-time dependencies. Linux and Windows inventories have no recorded text-presence gaps, but remain unreviewed too.

Retrieve applicable texts with verified provenance, resolve the actual license choices/conditions and attribution material, and assemble final notices. Do not substitute an unrelated old notice or generic license template just to make the audit pass. Tagged packaging currently rejects unresolved required text gaps.

**Why:** a functioning binary and a list of SPDX declarations are not enough to prepare a properly documented distribution. This is a current release gate, not a rendering bug. See [NOTICE_AUDIT.md](NOTICE_AUDIT.md).

### C. Native platform, display, and hardware qualification — needs target machines

Build, install, and run Windows, Linux, and macOS arm64 packages. Check Windows D3D12 and any claimed Vulkan alternative; Linux Vulkan with X11 and Wayland separately; and Apple Silicon/Metal. Inspect native library requirements and test paths containing spaces/non-ASCII characters, clean startup/shutdown, unusual exits, fullscreen, mixed-DPI/multiple monitors, input, resize, sleep/wake, and display-rate timing.

**Why:** cross-target type checking cannot reveal linker/dependency failures, driver limits, compositor behavior, or native child-process cleanup problems. One macOS GPU cannot establish these claims. Some Wayland operations also differ from X11 by design. A limitation remains in Windows cleanup if descendants outlive an already-exited parent; that needs explicit native testing.

**Completion evidence:** native linked packages plus recorded on-device runtime results, not simply green cross-compilation. See [PLATFORMS.md](PLATFORMS.md).

### D. Real failure behavior — requires suitable hardware and a controlled test plan

Extend recovery qualification beyond logical injection to available real driver/device and presentation failures, without casually forcing disruptive resets on a working session. Verify useful failure reporting and behavior when recovery cannot succeed.

**Why:** destroying a logical wgpu device exercises restoration code, but cannot reproduce every driver hang, OS reset, hardware memory failure, or lost desktop connection. Recovery must not be advertised more broadly than its evidence.

### E. Exact final-candidate duration tests — after remaining changes stabilize

Run the measured one-hour development check on the actual final candidate, plus the short auxiliary stress/recovery checks. The intended application sessions last at most one hour. The user will perform the **eight-hour test as the last final-release sign-off**, after implementation and other checks are finished.

**Why:** driver retention, slow memory growth, stale callbacks, and cumulative timing problems may not appear in a brief smoke. Testing an earlier executable does not prove a changed one. Freezing the candidate before long validation avoids repeatedly qualifying intermediate builds.

No long test should start automatically. The scripts save evidence independently; continual agent polling is unnecessary. No model/usage-mode switch is implied by that workflow. See [STABILITY.md](STABILITY.md).

### F. CI, distribution, and publication — requires final evidence and authorization

Run the configured hosted jobs from a clean standalone checkout. Confirm each intended architecture package installs on its target without Rust, contains the corrected docs/assets/notices, and has valid checksums. Review the public class-name collision check again before publishing. Decide the macOS signing/notarization policy and communicate installation expectations.

**Why:** local success can depend on an existing cache, installation, or toolchain. Native packaging must not accidentally label a host binary as another platform, omit resources, or require undocumented runtime libraries. Signing/notarization and checksums address different distribution concerns; a decision and clear instructions are needed.

The workflows exist, but hosted runs and all native artifacts have not been verified here. Publication requires explicit authorization and the final sign-off. **Nothing has been published.** See [RELEASE.md](RELEASE.md).

## 7. Deferred capabilities, not prerequisites for the first release

The plan explicitly allows v0.1.0 before shared memory, compute shaders, full Shadertoy support, an embedded Qt `ShaderView`, video decoding, or camera input. Those must not be counted as completed, but they also should not indefinitely block a clearly scoped first release.

Other broader capabilities remain later work: independent multiple windows, automatic full-process scene restoration, general graph DAGs, richer/independent texture bindings, more complete analysis representations, advanced graph scheduling, and remote renderer operation.

These are substantial additions. Multiple windows require per-window resources and lifecycle rules; graphs need dependency and feedback semantics; video/cameras need decoding, timing, format conversion, and platform permissions; shared memory needs synchronization and ownership rules; full scene reconstruction needs an explicit replayable state model. They should receive separate design and test milestones rather than be treated as small missing switches.

Several originally later-stage features—Float32 buffers, FFT streaming, a limited graph, input events, and constrained GLSL/Shadertoy support—already have implemented subsets. That does not establish completion of their unrestricted forms. Nor does completing the first-release subset mean every aspirational feature in the original plan is finished.

## 8. Recommended order from here

Priority updated at the user's request on 2026-09-25; see the
[native-platform milestone and agentic plans](PLATFORM_MILESTONE.md).

1. Build, install, test, and fix Windows x64, Linux x64, and macOS Apple Silicon natively; retain Intel Mac regression coverage. Use short checks first and record display/backend/hardware limitations.
2. Complete remaining short resize/reload/resource-interaction tests, notice gaps, and distribution/signing review.
3. Obtain authorization for hosted CI checks; this local handoff authorizes no remote writes, tags, or publication.
4. Freeze the candidate, package it, and schedule its exact-binary one-hour and short auxiliary checks with the user.
5. Hand that exact final candidate to the user for the eight-hour sign-off.
6. Publish only after explicit approval, with accurate supported-platform and limitation notes.

The installed-example repair is complete and locally verified. Reopen existing help tabs to see regenerated content. Run one demo at a time, use its cleanup, and wait for the renderer window to close before the next demo; audio-example cleanup deliberately leaves unrelated audio and the audio server alone.

## 9. Source and maintenance map

These paths refer to the source checkout; binary packages contain the executable rather than the Rust source.

| Location | Contents |
| --- | --- |
| `Classes/` | SuperCollider control/resource API and Pattern integration. |
| `renderer/src/main.rs`, `platform.rs` | Startup, arguments, native backend/display selection. |
| `renderer/src/osc.rs`, `protocol.rs`, `scheduler.rs` | Transport, validation, incoming admission, and future scheduling. |
| `renderer/src/app.rs` | Event-loop orchestration, command dispatch, replies, and recovery coordination. |
| `renderer/src/renderer.rs`, `controls.rs` | GPU resources/pipelines, graphs/feedback, snapshots, reflection, and interpolation. |
| `renderer/src/diagnostics.rs`, `overlay.rs` | Shared error budget/message limits and presentation-only statistics panel. |
| `renderer/shaders/`, `examples/`, `HelpSource/` | Original visual fixtures, runnable examples, twelve class pages, and one project guide. |
| `tests/`, `renderer/examples/`, `tools/` | Unit/live tests, readback/memory probes, strict help/soak/stress runners, notice audit, and package validation. |
| `.github/workflows/ci.yml` | Configured clean-checkout checks, native package matrix, and explicitly tagged release workflow. |
| `licenses/`, `docs/` | Pinned notice supplements, contracts, measured results, limitations, and release handoff. |

For a short checkpoint use [STATUS.md](STATUS.md); for chronological changes use [CHANGELOG.md](../CHANGELOG.md); for the evidence behind claims use [VERIFICATION.md](VERIFICATION.md). This report is a dated snapshot, not a substitute for rerunning the relevant checks when code, assets, dependencies, or release scope change.
