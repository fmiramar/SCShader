# SCShader

Continuing development on another computer? Start with the
[source handoff instructions](START_HERE.md) and
[native-platform milestone](docs/PLATFORM_MILESTONE.md).

SCShader is an experimental, process-separated shader-control subsystem for SuperCollider. SuperCollider remains responsible for composition and scheduling while an independent Rust process owns the GPU window and rendering through `wgpu`; this keeps GPU and window-system work out of `scsynth`'s real-time audio thread.

See the [implementation and architecture report](docs/IMPLEMENTATION_REPORT.md) for completed work, technologies used, verification evidence, and what remains before release—with the reason each remaining task matters.

This directory implements the feasibility renderer plus the protocol, lifecycle, shader-resource, timed-bundle, native-window, and Pattern layers of `SC_Shader_Interface_Agentic_Implementation_Plan.md`. It opens one configurable renderer-owned window, renders an original fullscreen WGSL shader, accepts loopback OSC, and sends ready, pong, status, shader lifecycle, timing-marker, window-metrics, input, and structured error replies. The renderer uses a bounded 256-packet incoming queue and a separately bounded future schedule; continuous update packets may be dropped under overload, with counted drops and sampled `E_QUEUE_FULL` diagnostics, while lifecycle commands wait for queue space.

Structured errors and their log entries share a burst of 20 plus 20 per second; status reports how many were suppressed. Critical GPU loss/fatal errors bypass that limit. Short resource-churn and incoming-flood checks cover cleanup, responsiveness, and diagnostic bounds; see [stability testing](docs/STABILITY.md).

`ShaderServer` is the SuperCollider-side controller. It launches a renderer executable, verifies its v1 handshake, tracks unexpected process exit, supports asynchronous status and synchronous ping barriers, estimates the renderer-clock offset from repeated pings, and sends the renderer a clean quit command. `ShaderWindow` configures that controller's one native window before boot and controls its supported runtime properties; it also exposes opt-in mouse, keyboard, resize, and focus callbacks. The renderer remains a separate process: it never runs GPU work in `scsynth`.

`ShaderDef` describes a native WGSL, supported-subset GLSL, or Shadertoy-style source file. `Shader` owns an independent pipeline with Naga-reflected float, signed/unsigned integer, boolean, vector, and matrix controls; it supports typed updates, asynchronous value queries, and per-frame renderer-side glides. All live file-backed shaders are watched four times per second; a replacement must validate before changing the pipeline, so invalid edits preserve the last valid output and controls. See [typed controls and shader layout](docs/CONTROLS.md).

`Shader` also registers the normal SuperCollider `\shader` Event type. A `Pbind` may reference a live `Shader`, a `ShaderDef`, or a definition name; it sends each reflected event-key uniform as one timed OSC bundle after `\latency`, defaulting to `ShaderServer.latency`. The initial event for a definition name creates the resource asynchronously, so its retained values are sent on creation acknowledgement; pre-create the shader when first-event timing must be timestamped.

`ShaderTexture` loads a static PNG, JPEG, or PNM image in the renderer process. Use `shader.setTexture(\source, texture)` to bind it, or `shader.feedback(\previous, 0.98)` to sample the previous completed renderer frame through resize-safe ping-pong targets. This is a deliberately narrow one-source image ABI; video and cameras are future work, while data streams and linear graph routing use the resources described below.

`ShaderBus` is a language-owned visual-rate control bus. Map a channel with `shader.map(\amount, bus)` and feed it from Patterns, MIDI, GUI code, or Routines; mapped controls for the same Shader are coalesced into one timestamped renderer bundle. Its optional smoothing runs at 60 Hz in `sclang`: `0` is immediate and values closer to `1` retain more of the previous value each step.

`ShaderAnalysis` packages standard `scsynth` analysis UGens as ShaderBuses. Give it a mono source-bus index and request any of amplitude, pitch, pitch confidence, spectral centroid, spectral flatness, onset, or zero-crossing rate; it sends compact `SendReply` updates at a deliberate control rate rather than polling server buses. Start it from a Routine, because it synchronizes its generated analysis SynthDef before creating the Synth.

`ShaderBuffer` is the corresponding moderate-size data resource. It sends Float32 values in big-endian OSC blobs to an `R32Float` GPU texture and currently exposes one exact, non-filterable `spectrum` binding. `ShaderFFTTexture` streams raw FFT magnitudes into that binding, while `ShaderWaveformTexture` captures recent delayed/latch waveform snapshots. Both take a mono audio-bus index, create their `scsynth` capture Synth at the default-group tail, and must be started from a Routine.

`ShaderGraph` is the first real multi-pass renderer path. It validates and commits a connected linear sequence of live Shaders; each pass has its own pipeline and `amount` state, writes to a renderer-owned target, and supplies the next pass's `source`. Branches, arbitrary DAGs, format conversion, and per-edge dimensions are deliberately deferred. A terminal-only graph feedback edge reuses the existing prior-frame targets and warms up from black after startup or resize.

`ShaderToy` is the compatibility constructor for original Shadertoy-style GLSL fragments. It requires `mainImage(out vec4, in vec2)`, wraps it in a fixed GLSL 4.50 fragment scaffold, and sends the result through wgpu's Naga parser rather than attempting an ad-hoc language rewrite. `iResolution`, `iTime`, `iTimeDelta`, `iFrame`, `iMouse`, and UTC `iDate` are live renderer values. `iChannel0` through `iChannel3` currently all alias the one `ShaderTexture` bound as `\source`; SC-specific `scPhase` is time and `scAmplitude` is `amount`, while the remaining SC analysis names are zero. This is a documented subset, not universal Shadertoy or desktop GLSL support.

## Build and run

Install [rustup](https://rustup.rs/) once. The checked-in `rust-toolchain.toml` selects Rust 1.97.1 plus `rustfmt` and Clippy.

Windows/Linux implementation and cross-target checks are in place; native GPU
testing remains pending. Defaults are D3D12 on Windows, Vulkan on Linux, and Metal
on macOS. See [platform build/test instructions](docs/PLATFORMS.md) for backend
overrides, Linux X11/Wayland selection, packaging, and short tests on another machine.

From the `SCShader` directory:

For native GNU/Linux installation, GPU/display selection, and the tested runtime
requirements, see the [Linux setup guide](docs/LINUX.md).

For Windows installation and NVIDIA/Intel laptops, see the
[Windows setup guide](docs/WINDOWS.md). The renderer accepts `--adapter NVIDIA`
or `--adapter Intel` for explicit GPU selection; pass these through
`ShaderServer.rendererArgs_` before boot. The default remains a high-performance
preference, and the ready callback reports the actual selected device.

```sh
cargo build --manifest-path renderer/Cargo.toml --locked
cargo run --manifest-path renderer/Cargo.toml --locked
```

The renderer listens only on `127.0.0.1:57140`. To select a different loopback port or validate a shader with the feasibility interface:

```sh
cargo run --manifest-path renderer/Cargo.toml --locked -- --listen-port 57141 --width 960 --height 540 --title SCShader
cargo run --manifest-path renderer/Cargo.toml --locked -- --shader renderer/shaders/fullscreen.wgsl
```

Open and evaluate [`examples/00_feasibility_spike.scd`](examples/00_feasibility_spike.scd) after the renderer reports ready. Moving the control changes the shader immediately. Closing the renderer window or sending `/scshader/v1/quit` terminates only the visual process.

## SuperCollider control layer

Install a packaged `SCShader/` directory as a SuperCollider extension; its renderer is discovered automatically. For a source checkout, install `Classes/`, `HelpSource/`, `renderer/shaders/` (as `shaders/`), and the built executable in the extension's `renderer/` directory first. Run one example at a time and its cleanup before the next; these examples create fresh controllers so a stale path on `ShaderServer.default` does not affect them:

```supercollider
(
v = ShaderServer.new(\serverHelp);
v.waitForBoot { |server|
    [server.version, server.backend, server.device].postln;
    server.status { |receiver, status| status.postln };
};
)
```

Cleanup before starting another demo; wait for the window to close:

```supercollider
(
v.free;
)
```

`ShaderServer.sync` is a ping/pong barrier. Call its blocking form from a Routine, or supply its `action` argument for a non-blocking callback. `Cmd-.` asks renderers launched by `ShaderServer` to quit; it does not terminate manually connected processes because they have no child-process ID.

`ShaderWindow` represents the renderer's one native window. Construct it before `boot` to set its logical size, title, logical position, decorations, startup fullscreen monitor, VSync, and cursor visibility. After boot, it can resize, change title/position/decorations/fullscreen/VSync/cursor state, request actual logical/framebuffer metrics, and bring the window forward. A renderer owns only one window in this milestone: `ShaderWindow.close` therefore stops its renderer, and independently rendered multi-window setups are still out of scope.

Input is off by default. Set `inputEnabled_(true)` to receive mouse callbacks with normalized 0–1 positions, logical key callbacks, resize callbacks with logical and framebuffer dimensions, and focus callbacks. Mouse motion is coalesced to one callback per rendered frame; button, key, focus, and resize events are delivered individually. See [`ShaderWindow.schelp`](HelpSource/Classes/ShaderWindow.schelp) and [`protocol/protocol.md`](protocol/protocol.md).

[`examples/01_shader_server.scd`](examples/01_shader_server.scd) discovers the installed renderer and shows a status callback and clean shutdown. Examples 01–04 also work when pasted into an unsaved IDE document; they resolve assets from the installed classes, not the document path.

[`examples/02_shader_lifecycle.scd`](examples/02_shader_lifecycle.scd) creates a `ShaderDef` and `Shader`, changes `amount`, and explicitly reloads the source. Each live Shader has an independent pipeline; use `ShaderGraph` when more than one should form a multipass chain.

`ShaderServer.sendBundle` now honors normal SuperCollider OSC bundle timing. At receipt, the renderer maps an OSC UTC/NTP timetag once into its own monotonic clock, queues future messages in timestamp/packet order, and applies them before the next eligible frame. Events more than 50 ms late are applied immediately and reported as `E_LATE_EVENT`. `scheduleMarker` is a measurement helper: it reports renderer event-application time, not display scan-out. See [`tests/sc/timing_smoke.scd`](tests/sc/timing_smoke.scd) and [`tests/sc/timing_benchmark.scd`](tests/sc/timing_benchmark.scd).

Future work is limited to 8,192 commands including bundle containers, 16 MiB of charged payload, and one hour of lookahead. Bundles dispatch consecutively and are rejected whole on parse/admission failure; runtime resource errors are not transactional rollback. Use `server.clearScheduled` to discard pending future work, and `server.status` to inspect queue counts, bytes, and overload counters. See the [protocol limits](protocol/protocol.md#queue-clock-and-bundle-behavior) and [measured stability-test procedure](docs/STABILITY.md).

## Diagnostics and recovery

Set `ShaderServer.default.showStats = true` before or after boot for an optional live stats panel. It displays CPU frame timing, backend/device, dimensions, compile status, scheduling counters, and estimated owned texture memory. It is disabled by default and excluded from feedback history.

A lost GPU device permits one in-process restart using cached shaders, controls, images, buffers, and graph bindings. Resource IDs and clocks survive; feedback history clears to black, and `server.recoveryAction` receives a notification. Failed recovery or a second loss exits with code 70. This is separate from reconstructing a crashed renderer process, which remains deferred. See [recovery verification](docs/RECOVERY.md).

## Pattern control

```supercollider
(
var source = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/fullscreen.wgsl";
v = ShaderServer.new(\guidePattern);
v.waitForBoot { |server|
    d = ShaderDef(\guidePulse, source);
    x = Shader(d, [\amount, 0.2], server);
    ~scShaderSetup = Routine({
        var deadline = Main.elapsedTime + 5;
        while({ x.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        if(x.isRunning.not) { Error("Shader did not become ready; inspect x.lastError.").throw };
        p = Pbind(
            \type, \shader,
            \shader, x,
            \amount, Pseq([0.2, 0.5, 0.8], inf),
            \dur, 1/4,
            \latency, server.latency
        ).play;
    }).play(SystemClock);
};
)
```

Cleanup before starting another demo; wait for the window to close:

```supercollider
(
p.tryPerform(\stop);
~scShaderSetup.tryPerform(\stop);
v.clearScheduled; // discard future Pattern updates before freeing their target
x.tryPerform(\free); d.tryPerform(\free);
v.free;
)
```

The `\window` key optionally selects a `ShaderWindow` and therefore its controller; `\shaderID` selects a specific live Shader. `\dur`, `\delta`, and `\sustain` remain normal Event keys, without implicit shader gate/release behavior. A separate `Pshader` wrapper is intentionally deferred because `Pbind` already supplies the required composition behavior.

## Typed controls and smoothing

Declare a flat custom uniform struct at group 0, binding 4; the renderer reflects its names, types, and byte offsets. Existing `amount`-only shaders remain compatible. After creation, use `x.set(\color, [0.2, 0.5, 1.0])`, `x.glide(\color, [1, 0, 0], 0.5, \smooth)`, or `x.get(\color, { |value| value.postln })`. Integer and boolean controls use step changes; float scalars, vectors, and matrices also support linear and smoothstep ramps evaluated each renderer frame. See [the ABI and limits](docs/CONTROLS.md) and [the runnable example](examples/04_typed_controls.scd).

## Visual buses and audio analysis

Run the self-contained [audio-analysis example](HelpSource/Classes/ShaderAnalysis.schelp), also available in SuperCollider Help under `ShaderAnalysis`. It creates the shader, audio bus, looping `ExampleFiles.child` source, and analysis Synth, waits for buffer/SynthDef readiness, and includes cleanup. `SoundIn` is an optional commented alternative.

`ShaderAnalysis` runs its Synth at the default group tail so ordinary source Synths write the bus first. Cleanup stops its mapping and clears scheduled updates before freeing the visual target; it leaves unrelated audio and the audio server running.

## Float buffers and audiovisual streams

Run the complete [FFT example](HelpSource/Classes/ShaderFFTTexture.schelp), available in SuperCollider Help under `ShaderFFTTexture`. It includes its own source, bus, stream, cleanup, and an original spectrum-bar shader; no undeclared `audioBus` or `fft` variables are required.

`ShaderBuffer.float(128).setn(0, data)` is useful for language-generated arrays; bind it with `x.setBuffer(\spectrum, buffer)`. In custom WGSL, declare `@group(0) @binding(3) var spectrum_buffer: texture_2d<f32>;` and read exact values with `textureLoad(spectrum_buffer, vec2<i32>(bin, 0), 0).r`. The first version intentionally caps buffers at 16,384 floats. FFT magnitudes are raw linear values; normalize or compress them in the shader. These are controlled-rate streams, not audio-rate GPU updates or shared-memory transport.

## Multipass graphs

Run the self-contained [two-pass graph example](HelpSource/Classes/ShaderGraph.schelp), also in SuperCollider Help under `ShaderGraph`. It creates both shaders, waits for them to become ready, commits the chain, and includes cleanup.

Call `g.connectFeedback(b, 0.92)` after forming the chain for terminal prior-frame feedback. The graph rejects cycles, branches, missing connections, duplicate pass inputs/outputs, wrong-server nodes, and freed resources before dispatching anything to the renderer. Resize intentionally clears its frame history.

## Verification

Run the static checks and unit tests:

```sh
cargo fmt --manifest-path renderer/Cargo.toml -- --check
cargo clippy --manifest-path renderer/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path renderer/Cargo.toml --locked
python3 -m unittest discover -s tests/python -v
python3 tools/run_sc_checks.py
```

Execute every installed help/README demo and cleanup with `tools/check_help_examples.py`; see [the short documentation-check command](docs/STABILITY.md). SCDoc rendering alone does not establish that examples execute. The example-file smoke also checks copying setup blocks into an unsaved document without substituting paths.

The measured soak harness can launch and own a renderer, send 1000 float updates per second for 60 minutes, and monitor fresh replies, frame progress, and bounded process memory. Supply the exact candidate path; target-specific packages build under `renderer/target/<target-triple>/release/` (with `.exe` on Windows):

```sh
python3 tools/soak_uniforms.py --renderer /path/to/candidate/scshader-renderer --minutes 60 --rate 1000 --json build/soak/traffic-60m.json
```

Use `--seconds 10` for a short smoke, never as hour/eight-hour evidence. Current validation targets one-hour sessions plus short overload, reload, feedback/resize, and recovery checks. An eight-hour test is reserved for the final candidate's user-run release sign-off. CSV/JSON output records actual duration and failure/interruption; choose fresh output paths per run. See [STABILITY.md](docs/STABILITY.md) for commands, memory bounds, and the separate stress tests. RSS is not GPU-memory telemetry, and advancing frame counters do not prove correct displayed pixels.

## Release packages

Release archives contain an `SCShader/` extension directory with the matching renderer executable, classes, help, original shader fixtures, license, and Quark metadata. Install a downloaded archive by placing that `SCShader` directory in the SuperCollider user Extensions directory, then recompile the class library. Checksums are supplied alongside each archive; Quark metadata is source-only and never downloads an executable automatically. See [`docs/RELEASE.md`](docs/RELEASE.md) for the package layout, supported targets, and maintainer checklist.

## Protocol and current scope

The versioned subset and its temporary unversioned Phase-0 aliases are documented in [`protocol/protocol.md`](protocol/protocol.md). `ShaderServer` uses the versioned handshake, ping/pong, status, error, timing-marker, window, and quit messages; `Shader` uses the versioned create/reload/free/reflection messages. The controller keeps a lowest-round-trip estimate of the relation between `Main.elapsedTime` and the renderer's monotonic clock, without assuming the two processes began at a common epoch.

The completed macOS checks and the explicitly outstanding long/cross-platform tests are recorded in [`docs/VERIFICATION.md`](docs/VERIFICATION.md).

## Source and status

See the [Scintillator comparison](docs/SCINTILLATOR_COMPARISON.md) for architecture
and workflow tradeoffs, current limitations, and proposed future additions.

The architecture follows the local SC Shader Interface agentic implementation plan and uses the documented `wgpu`/WGSL, `winit`, and `rosc` APIs. All shipped shader code is original to this project. Exact dependency versions, sources, and licenses are recorded in [`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md), and the provisional public-name audit is recorded in [`docs/NAME_COLLISION_CHECK.md`](docs/NAME_COLLISION_CHECK.md).

Still experimental, not the final v0.1.0 release. Version 0.0.14 passed its isolated one-hour traffic gate and short reload/feedback checks with the [Metal multipass memory workaround](docs/MEMORY_INVESTIGATION.md); changed renderer candidates need their own one-hour evidence. Outstanding work includes native cross-platform verification, full dependency notices, hardware-reset qualification, broader stress testing, and the user-run eight-hour sign-off on the completed final candidate. Reflection is restricted to the documented flat control block; video/camera textures, arbitrary storage buffers, FFT phase/mel/chroma streams, shared memory, independent multiple windows, runtime monitor switching, and general graph DAGs remain unsupported. See [the implementation checkpoint](docs/STATUS.md).

Timing is frame-oriented, not sample-accurate. Native WGSL retains the built-in uniform layout and `vs_main`/`fs_main` entry points, with an optional binding-4 control block. GLSL is a strict Naga-parsed fragment subset; Shadertoy requires `mainImage` and maps all `iChannelN` names to the same source texture. Invalid edits preserve the last valid shader. `Pshader` is not supplied because the `\shader` Event type integrates directly with `Pbind`.

## License

SCShader is licensed under GPL-3.0-or-later. Direct Rust dependencies declare permissive licenses; see the dependency record for details. Development archives include an explicitly unreviewed dependency audit; unresolved macOS notice texts and the full distribution review remain release gates. See [the notice audit](docs/NOTICE_AUDIT.md).
