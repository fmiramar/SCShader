# SCShader and Scintillator: comparison and future additions

Assessment date: **2026-09-27**. SCShader baseline: the local **0.0.16 working
tree**, including the documentation repair and platform handoff. Scintillator
baseline: historical source revision **a50b6f98ed6ce758529d0f11c998bdd92e470efb**.

## 1. Conclusion

Neither project is simply a better version of the other.

**Scintillator has the stronger SuperCollider-native visual composition model:**
visual unit generators, SynthDef-like definitions, reusable instances, ordered
groups/layers, animation tables, and an offscreen image workflow. **SCShader is
better aligned with directly authoring and performing shader files:** WGSL,
constrained GLSL/Shadertoy, typed reflected controls, watched reload, and analysis
of existing SuperCollider audio buses. The evidence and limits behind these
judgments are detailed below.

For this project's intended use—SuperCollider-controlled visuals for sessions of
up to roughly one hour—my recommendation is to **keep SCShader's architecture**,
complete its native-platform milestone, and selectively adopt ideas for authoring,
compositing, animation, and visual testing. Rebuilding Scintillator inside SCShader
would be a different project, not a small compatibility improvement.

This is a source/documentation assessment, **not a head-to-head benchmark**.
“Better” below means better for the named workflow or a demonstrated feature
advantage, not proven higher FPS, lower latency, or greater reliability everywhere.

## 2. What was actually compared

The [linked GitHub repository][sc-repo] is archived. Its default branch was cleared
on 2023-11-15 and now points to a different website. I inspected the preceding
[source-bearing commit][sc-commit], dated 2021-01-30. Its Quark/CMake version is
still 0.0.8, but it includes changes after the 2020 release notes, including tweens.
Consequently this document compares that **specific development snapshot**, not
just the released 0.0.8 package. [Revision history][sc-history], [Quark metadata][sc-quark].

The relocation notice is not proof that every successor or fork is abandoned.
This assessment does not establish the state of a post-move implementation,
continued availability of historical binary downloads, or compatibility of that
old source with current operating systems. No Scintillator installer, executable,
build script, or test suite was run; its public source was downloaded only for
inspection. Upstream links below are pinned to the inspected commit.

For SCShader, I checked the actual classes, renderer, control/graph implementation,
and the [implementation report](IMPLEMENTATION_REPORT.md),
[verification record](VERIFICATION.md), and [current status](STATUS.md). Its local
macOS x64 evidence is substantial, but native Windows, Linux, and Apple Silicon
qualification is still pending. The earlier 0.0.14 one-hour pass does not qualify
0.0.16. Neither the comparison nor Scintillator's historical release history changes
those facts.

## 3. Architecture: related principles, different abstraction levels

Both use a separate visual server controlled from sclang over OSC. Neither is
fundamentally a GPU renderer running inside scsynth's audio callback. Process
separation is a shared design strength, not a unique SCShader advantage.
[Scintillator architecture][sc-readme]; [SCShader architecture](IMPLEMENTATION_REPORT.md).

| Aspect | SCShader now | Scintillator at the inspected revision |
| --- | --- | --- |
| User describes | A shader file and its controls/resources | A function constructing a graph of visual generators |
| Main language objects | ShaderDef, Shader, ShaderGraph, ShaderServer | ScinthDef, Scinth, VGen, ScinGroup, ScinServer |
| Compilation path | WGSL or supported GLSL → Naga/wgpu pipeline | sclang VGen graph → YAML definition → generated GLSL → shaderc/SPIR-V → Vulkan |
| Native renderer | Rust; wgpu, winit, rosc | C++17; Vulkan, GLFW, liblo |
| macOS graphics path | wgpu Metal backend | Vulkan through MoltenVK |
| Other graphics paths | D3D12 default on Windows; Vulkan default on Linux; explicit alternatives | Vulkan; optional SwiftShader software device for offscreen work |
| Composition unit | Fullscreen shader pass; current graph is one chain | Visual synth instances drawn in an ordered node/group tree |
| Definition storage | Source-file descriptor; renderer constructs a pipeline per created Shader | Compiled definition shared by Scinth instances, with per-instance parameters |

Sources: [ScinthDef serialization][sc-def-code], [shader compiler][sc-compiler],
[instance implementation][sc-instance], [native build configuration][sc-build],
[server options][sc-options], [SCShader dependencies](DEPENDENCIES.md),
[platform selection](../renderer/src/platform.rs), and
[renderer implementation](../renderer/src/renderer.rs).

Scintillator also divides computation into **frame**, **shape**, and **pixel**
rates, compiled into compute, vertex, and fragment stages respectively. That is
a useful UGen-like way to express how often work must occur. It does not establish
an unrestricted particle/compute API or a complete 3D scene engine. Its public
shape implementation in this snapshot is a subdividable quad.
[Draw process][sc-draw], [shape implementation][sc-shape].

Important distinction: a graph of mathematical operations **inside a shader**,
a tree of **layered synth instances**, and a graph of **render-to-texture passes**
are three different things. Scintillator's VGens/ScinGroups and SCShader's
ShaderGraph should not be counted as equivalent implementations of “graph support.”

## 4. Where SCShader is better suited or demonstrably broader

### 4.1 Direct shader development and typed control

SCShader lets the user edit WGSL directly, or use its documented GLSL/Shadertoy
subset. Reflected controls include float, int, uint, logical bool, vectors, and
square matrices. Layout and padding come from Naga, not argument order. Live edits
preserve matching values/glides; invalid edits retain the last valid pipeline.
This is a strong fit for shader programmers and custom mathematical visuals.
[Control and reload contract](CONTROLS.md).

Scintillator's inspected parameter update path converts numeric control values to
floats; its multidimensional VGen signals are not equivalent to a typed external
uniform interface. Its graph authoring is excellent for its own abstraction, but
the inspected public interface is not the same as loading arbitrary user shader
files with reflected matrices and integers. This is a concrete SCShader advantage
for that workflow, not a claim that vectors are absent from Scintillator.
[Control updates][sc-node-set], [definition construction][sc-def-code].

SCShader's compatibility advantage has limits: all four iChannel aliases currently
refer to the same source texture, and GLSL is a constrained parser-supported subset.
It cannot honestly advertise universal Shadertoy compatibility.
[Shader compatibility implementation](../renderer/src/renderer.rs).

### 4.2 Integration with the audio already running in SuperCollider

SCShader provides amplitude, pitch/confidence, spectral centroid/flatness, onset,
and zero-crossing analysis, plus FFT-magnitude and waveform snapshot streams. These
read SC audio buses through ordinary Synths and pass bounded visual data through
sclang. Audio composition and analysis can remain in one existing SC setup.
[ShaderAnalysis](../Classes/ShaderAnalysis.sc),
[FFT stream](../Classes/ShaderFFTTexture.sc),
[waveform stream](../Classes/ShaderWaveformTexture.sc).

Scintillator has real PortAudio input support; it must not be described as having
no audio integration. However, its AudioStager explicitly contains stereo/60 Hz
assumptions and uploads recent audio samples to an image. That is a different and
narrower facility than the analyzed-bus API above. For an existing SC piece,
SCShader avoids opening a second audio-device pipeline just to obtain those
features. For visuals driven directly by a microphone without scsynth, direct
PortAudio could instead be convenient. [Audio staging][sc-audio].

SCShader's bridge is not free: language/OSC traffic has costs, and waveform snapshots
are not audio-rate shared memory. Neither path is proven faster by this review.

### 4.3 Explicit local operational contracts

SCShader has documented bounds for incoming and future work, whole-packet admission,
drop/suppression counters, timed command markers, and renderer-side control glides.
It also implements last-good reload, surface recreation, and one bounded logical
device restart with resource restoration and a defined feedback reset.
[Protocol](../protocol/protocol.md), [stability](STABILITY.md), [recovery](RECOVERY.md).

These are valuable, inspectable contracts for live work. They are **not proof of
universal reliability superiority**: Scintillator has its own OSC implementation,
synchronization, tests, and crash-reporting machinery. In particular, liblo-based
transport must not be dismissed as “no timed OSC.” Equivalent overload, timing,
and failure benchmarks were not run. [Dispatcher][sc-dispatcher],
[crash-report policy][sc-crash].

### 4.4 Reproducible dependency selection and explicit packages

SCShader pins Rust and its Cargo dependency graph and has architecture-checked
packagers. Its end-user package does not silently download a renderer during class
initialization. This fits controlled, offline performance setups once installed.
[Dependencies](DEPENDENCIES.md), [packaging](RELEASE.md).

Scintillator uses CMake, submodules, and downloaded native dependency bundles.
Submodules can be pinned, so they are not inherently unreproducible; nevertheless,
the inspected external liblo build tracks a moving master branch. Its automatic
installer adds another network-dependent component. SCShader's dependency workflow
is more explicitly locked here, although wgpu still brings a substantial transitive
graph and SCShader's notice review remains unfinished.
[External dependencies][sc-third-party], [installer][sc-installer].

## 5. Where SCShader is currently worse or missing useful capabilities

### 5.1 Higher entry cost for an SC musician who does not write shaders

Scintillator supplies visual oscillators, coordinates, vector operations, image
samplers, and output generators that can be combined inside sclang. The user can
think in a familiar SynthDef/UGen style without managing a shader ABI first.
SCShader's raw-file workflow is more demanding for this audience, and its small
fixture set is not a broad visual instrument library. This is probably
Scintillator's most important user-experience advantage.
[VGen overview][sc-vgens], [tutorial][sc-guide].

### 5.2 Real layer ordering and grouping

Scintillator provides groups, ordered insertion, reordering, tree queries, and
group cleanup. Its graphics pipeline alpha-blends successive Scinth draws. This
directly supports overlapping independently controlled visual objects.
[ScinGroup][sc-group], [blend state][sc-pipeline].

SCShader retains multiple shader resources but currently presents the selected
shader or a single connected pass chain. Its pass pipeline uses replacement
blending, and ShaderGraph rejects branches/multiple inputs. Creating several
Shader objects therefore does **not** establish a general layered visual scene.
A user can implement specific composition in a shader, but that is not the same
as a built-in layer/group API. [ShaderGraph](../Classes/ShaderGraph.sc),
[renderer](../renderer/src/renderer.rs).

### 5.3 Rich animation curves and multi-segment sequences

Scintillator's ScinTween describes levels, segment durations, easing curves, and
looping; the server builds lookup textures sampled by tween VGens. This offers
piecewise animation and spatial curve sampling, not just target-to-target smoothing.
SCShader's step/linear/smoothstep glides are useful but substantially narrower.
[ScinTween][sc-tween], [tween/pipeline construction][sc-def-native],
[SCShader glides](CONTROLS.md).

The mechanisms should not be confused: a per-frame CPU-interpolated uniform and
a texture lookup performed independently at many pixels solve different problems.
SCShader can write arbitrary easing mathematics into WGSL already; what it lacks
is the convenient reusable language/server facility.

### 5.4 Offscreen output and broad image-regression infrastructure

Scintillator implements non-window rendering, explicit frame/time advancement,
and screenshots in non-realtime modes. Its test tools render a corpus and compare
images against reference outputs, including group order. This is a useful model
for reproducible visuals and debugging. It is not evidence of a complete movie
recorder or editor. [Offscreen implementation][sc-offscreen],
[screenshot API][sc-server], [image comparisons][sc-image-tests].

SCShader has specialized GPU readback/recovery probes and an offscreen development
memory probe, but no comparable supported user-facing capture/offline-render API
or broad golden-image corpus. Its live tests prove valuable things that screenshots
cannot, such as responsive scheduling and resource cleanup; both kinds are needed.
[Verification](VERIFICATION.md), [renderer probes](../renderer/examples/).

Scintillator's image test driver explicitly uses SwiftShader and odd-sized targets.
That improves repeatability and catches row-stride assumptions, but **software-rendered
CI does not qualify native GPU drivers or window presentation**. A future SCShader
test suite should preserve that distinction. [Image generation tests][sc-image-generation].

### 5.5 Richer image bindings and adapter selection

Scintillator constructs descriptor bindings for multiple fixed/parameterized image
samplers and tween images. SCShader's current resource ABI exposes one source
image and one Float32 spectrum texture, with restrictive sharing semantics. Mixing
independent images/data sources is consequently a real SCShader limitation.
[Scinth bindings][sc-instance], [SCShader resource layout](../renderer/src/renderer.rs).

Scintillator also exposes GPU selection by device name. SCShader exposes backend
and Linux display selection but currently requests a high-performance compatible
adapter without a public device selector. Explicit adapter reporting/selection
would help dual-GPU machines and reproducible platform tests.
[Scintillator options][sc-options], [SCShader startup](../renderer/src/main.rs).

### 5.6 Background preparation and reusable definitions

Scintillator's Async worker pool parses/loads definitions and images off the main
render path; definitions are built once and shared by instances. In SCShader,
source loading/pipeline creation and image creation are called through the render
event loop, and creating another Shader constructs another pipeline. The underlying
GPU implementation may cache work, but SCShader has no explicit shared-definition
cache equivalent in this code. [Async jobs][sc-async], [definition registry][sc-root],
[SCShader dispatch](../renderer/src/app.rs), [creation path](../renderer/src/renderer.rs).

This is an architectural opportunity, not measured evidence that Scintillator never
stutters. For SCShader, profile compile/load stalls before changing threading.
Background preparation needs bounded queues, cancellation, version/generation checks,
safe GPU ownership, and last-good atomic replacement—not simply another thread.

### 5.7 Historical distribution experience

Scintillator's release notes describe macOS, Linux AppImage, and Windows binaries;
its historical CI includes all three systems. That is more distribution history
than SCShader currently has. It does not prove those old packages work on current
systems, Apple Silicon, or native Wayland. Conversely, SCShader's newer backend
selection and packaging scripts do not substitute for native testing.
[Release notes][sc-releases], [CI definition][sc-ci],
[SCShader platform milestone](PLATFORM_MILESTONE.md).

## 6. What is mainly different, rather than automatically better or worse

- **Rust/wgpu versus C++/Vulkan.** SCShader delegates backend portability and much
  validation to wgpu; Scintillator manages Vulkan more directly. This changes
  maintenance boundaries and available control. Rust alone does not establish
  faster graphics or eliminate driver failures. MoltenVK adds a different
  translation path on macOS, but no measured overhead comparison was made.
- **Shader text versus a visual synthesis language.** Text is flexible for shader
  specialists; VGens are convenient for SC composers. Both can be good interfaces.
  An optional front end could generate SCShader-compatible WGSL without replacing
  its renderer. It would still require a real compiler/validation design.
- **Local UDP versus UDP and TCP.** SCShader deliberately binds loopback UDP;
  Scintillator creates liblo UDP and TCP listeners, and its source notes a desire
  to restrict binding more narrowly. Wider connectivity is useful only with a
  remote/multi-client requirement. It also adds deployment/security concerns; TCP
  does not inherently solve display timing. [Dispatcher][sc-dispatcher],
  [startup binding note][sc-main], [SCShader OSC](../renderer/src/osc.rs).
- **Manual package installation versus automatic binary installation.** Automation
  reduces initial steps when its download service works. Explicit packages make
  architecture/version choice and offline deployment more visible. Neither should
  be confused with the corrected auto-discovery of an already installed renderer.
- **Recovery versus crash analysis.** SCShader tries limited in-process device
  recovery; Scintillator integrates Crashpad and an explicitly user-approved upload
  workflow. They serve different phases of failure handling. Scintillator's policy
  explicitly rejects automatic uploads, so calling it automatic telemetry would
  be inaccurate. [Crash policy][sc-crash], [SCShader recovery](RECOVERY.md).

## 7. Future additions that are worth considering

These are **proposals**, not implemented features or a replacement for the current
platform-first plan. Effort ratings are relative engineering scope, not time quotes.

| Proposal | Value for this project | Effort / recommended place |
| --- | --- | --- |
| Original shader preset library with SC helpers, previews, and typed defaults | High: makes the existing engine immediately useful without learning shader layout | Low–medium; first user-facing add-on after platform qualification |
| Adapter enumeration/selection and a local diagnostic summary | High for compatibility work and dual-GPU diagnosis | Medium; focused follow-up to native test findings |
| Broader deterministic image regression tests | High: catches visually wrong output even when status tests pass | Medium; incremental engineering improvement |
| Multi-segment control automation/envelopes | High for live composition and repeatable movement | Medium; extend existing controls before a new graph language |
| Independent texture slots and a small ordered compositor | High for combining images, audio data, and visual layers | High; explicit resource-ABI/composition milestone |
| Public screenshot, then deterministic offline frame rendering | Medium–high for documentation, artwork export, and rehearsals | Medium–high; grow from readback infrastructure |
| Background shader/image preparation and pipeline sharing | High if real workloads reveal compile/load stalls | Medium–high; profile first, preserve reload/recovery invariants |
| Small optional VGen-like WGSL front end | Potentially high for SC-only users; uncertain demand | High; prototype separately after presets prove insufficient |

### Suggested boundaries and acceptance criteria

**Preset/helper add-on.** Start with original gradients, oscillatory patterns,
coordinate transforms, image effects, and audio-reactive examples. Provide complete
setup/cleanup and visual previews. Reuse ShaderDef/Shader and existing reflected
controls. Do not copy upstream imagery or promise new classes before implementation.
Success: a musician can produce several useful results using only short SC examples.

**Automation add-on.** Add bounded segment lists, looping and a small easing set for
float/vector controls. Define retarget, cancel, time-base, reload, and recovery
behavior. Keep integer/bool transitions discrete. Matrix interpolation must remain
explicitly component-wise unless a separate transform-aware type is introduced.
Start with CPU evaluation per frame; add GPU lookup curves only for a demonstrated
per-pixel sampling need. Success: deterministic segment boundaries and repeated
cycles without extra per-frame OSC traffic.

**Texture/compositor milestone.** First define independently owned input slots;
then add ordered layers with opacity and a few blend operations. Specify linear
versus display color space and straight versus premultiplied alpha. Keep routing
and cleanup inspectable. Test layering order, resize, freed inputs, and recovery
with reference images. A small compositor is more immediately valuable than
unrestricted DAGs and arbitrary feedback cycles.

**Image testing/capture.** Begin with fixed source, time, dimensions, and inputs;
test odd image widths, texture orientation, control layout, and feedback ordering.
Use exact assertions for deliberately exact fixtures and justified tolerances for
floating-point rendering across different backends. A software baseline must remain
separate from native driver tests. Public capture then needs bounded asynchronous
readback/encoding and clear failure replies; offline export additionally needs a
deterministic clock and a policy for audio input and feedback history.

**Optional graph front end.** Limit a first prototype to constants, typed controls,
coordinates, arithmetic, color construction, and a few oscillators. Generate ordinary
WGSL accepted by the existing renderer. Keep source diagnostics and a text escape
hatch. Do not begin by cloning every VGen, rate system, YAML schema, and node command.
Success: materially less code for real user patches, with useful errors and no
regression in direct shader authoring.

## 8. What is not worth doing now

“Not worth it” here means low return for the current SCShader scope—not that these
features are useless to every audiovisual project.

1. **Replace Rust/wgpu with Scintillator's C++/Vulkan engine.** This discards working
   integration and validation while creating a second dependency/build migration.
   Reconsider only for a measured, otherwise unsolved backend limitation.
2. **Build a full Scintillator API/protocol emulator.** Graph serialization, VGen
   dimensions/rates, instance/group semantics, and assets require substantive
   translation. Shared use of OSC does not make either project drop-in compatible.
   A small importer for specific owned compositions is more defensible if requested.
3. **Duplicate scsynth with direct PortAudio, another FFT stack, or GPU audio DSP.**
   Existing SC bus analysis solves the present workflow. Reconsider direct input
   only for a standalone visual-server use case or measured transport bottleneck.
4. **Add compute merely because Scintillator uses compute internally.** Per-frame
   VGen evaluation is not the same requirement as particle simulations or arbitrary
   storage buffers. Add compute around an actual workload with resource/scheduling
   contracts, not as a parity checkbox.
5. **Bundle a full media stack solely because upstream uses FFmpeg.** Inspection
   verified image decoding/encoding and screenshot workflows, not a finished
   camera/video-editing system. Movie playback, seeking, timing, decoding threads,
   device permissions, and distribution obligations need a separate use case.
   [Image decoder][sc-image-decoder], [image encoder][sc-image-encoder].
6. **Automatic downloads during class compilation or a hosted crash service.**
   Prefer predictable explicit installs and a user-reviewed local diagnostic export
   first. A remote service creates an ongoing privacy/operations burden. If an
   installer is later desired, make it explicit, authenticated, architecture-aware,
   and recoverable; do not reproduce historical delivery assumptions blindly.
7. **General DAGs, many independent windows, remote rendering, or shared memory
   before a concrete need.** These can be worthwhile later, but each adds ownership,
   ordering, synchronization, and platform work. They do not repair today's missing
   native compatibility evidence and are not justified by one-hour usage alone.
8. **Promise universal Shadertoy compatibility.** Independent channels and selected
   well-tested passes are achievable incremental steps; supporting every external
   shader, buffer arrangement, and media dependency is an open-ended commitment.

## 9. What cannot be ranked honestly yet

There is no common-machine benchmark here for FPS, command-to-visible latency,
compile hitches, CPU/RAM/VRAM, shader count scaling, or crash frequency. Do not infer
these from implementation language, repository age, test counts, or feature breadth.
Scintillator's generated stages may avoid redundant work; SCShader's narrower
runtime may avoid other costs. Both are hypotheses until measured.

A fair optional comparison would use the same GPU, resolution, refresh/VSync,
equivalent shader math, input images, and workload. Begin with short runs of a
gradient, animated procedural pattern, and sampled image. Record compile/load
times and frame-time distributions separately from steady-state throughput. Only
compare compositing or offline rendering where both implementations genuinely
support equivalent semantics. GPU submission is not display scan-out, and RSS is
not total GPU memory. Do not start additional long soaks just to produce a ranking.

## 10. Recommended decision

1. Finish **Windows, Linux, and Apple Silicon native compatibility**, retaining
   Intel regression coverage, as already agreed. [Active milestone](PLATFORM_MILESTONE.md).
2. Add the smallest high-value usability/testing improvements: presets/previews,
   adapter diagnostics, and a broader image-regression corpus.
3. Prioritize richer automation and independent texture/layer composition according
   to the first real pieces users want to make.
4. Add export or a small graph-authoring front end only when their use cases justify
   the extra architecture. Keep shader-file authoring available throughout.

In short: **learn from Scintillator's composition experience and visual testing;
do not copy its entire engine or mistake historical feature breadth for present-day
platform qualification.** SCShader's biggest immediate need is still finishing and
verifying the system it already implements.

## 11. Provenance, reuse, and transfer note

This document describes ideas and observed implementation; it imports no upstream
code or assets. Scintillator's snapshot includes a GPLv3 license text, while SCShader
declares GPL-3.0-or-later. Before any actual source, shader, image, or dependency is
reused, inspect its specific license/attribution and preserve provenance; the shared
license family is not blanket clearance for every bundled asset.
[Upstream license][sc-license], [SCShader notices](NOTICE_AUDIT.md).

The previously created source ZIP remains
`fmiramar-SCShader-0.0.16-source-handoff-2026-09-25.zip`. This comparison was written
later, on 2026-09-27, and **is not inside that unchanged ZIP**. Copy this Markdown
document alongside it for transfer, or create a separately named refreshed source
snapshot when requested. No runtime code, Git commit, publication, or long-duration
test was performed for this assessment.

### Primary upstream references

All source-file references below identify the same historical revision. Local links
refer to the current SCShader working tree. Architectural recommendations and effort
ratings in this document are judgments based on those sources, not upstream promises.

[sc-repo]: https://github.com/ScintillatorSynth/Scintillator
[sc-commit]: https://github.com/ScintillatorSynth/Scintillator/commit/a50b6f98ed6ce758529d0f11c998bdd92e470efb
[sc-history]: https://api.github.com/repos/ScintillatorSynth/Scintillator/commits?per_page=5
[sc-quark]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/Scintillator.quark
[sc-readme]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/README.md
[sc-def-code]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/classes/ScinthDef.sc
[sc-compiler]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/ShaderCompiler.cpp
[sc-instance]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/Scinth.cpp
[sc-build]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/CMakeLists.txt
[sc-options]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Classes/ScinServerOptions.schelp
[sc-draw]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Reference/ScinthDef-Draw-Process.schelp
[sc-shape]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/classes/ScinShape.sc
[sc-node-set]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/osc/commands/NodeSet.cpp
[sc-audio]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/AudioStager.cpp
[sc-dispatcher]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/osc/Dispatcher.cpp
[sc-crash]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Guides/Scintillator-Crash-Reports-And-Privacy.schelp
[sc-third-party]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/third_party/CMakeLists.txt
[sc-installer]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/classes/ScinServerInstaller.sc
[sc-vgens]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Guides/VGens-Overview.schelp
[sc-guide]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Guides/Scintillator-User-Guide.schelp
[sc-group]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Classes/ScinGroup.schelp
[sc-pipeline]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/Pipeline.cpp
[sc-tween]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Classes/ScinTween.schelp
[sc-def-native]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/ScinthDef.cpp
[sc-offscreen]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/Offscreen.cpp
[sc-server]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Classes/ScinServer.schelp
[sc-image-tests]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/tools/compare-test-images.py
[sc-image-generation]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/tools/TestScripts/makeTestImages.scd
[sc-async]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/Async.cpp
[sc-root]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/comp/RootNode.cpp
[sc-releases]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/HelpSource/Reference/Scintillator-Release-Notes.schelp
[sc-ci]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/.travis.yml
[sc-main]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/scinsynth.cpp
[sc-image-decoder]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/av/ImageDecoder.cpp
[sc-image-encoder]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/src/av/ImageEncoder.cpp
[sc-license]: https://github.com/ScintillatorSynth/Scintillator/blob/a50b6f98ed6ce758529d0f11c998bdd92e470efb/LICENSE
