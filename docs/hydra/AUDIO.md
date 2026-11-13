# Audio control bridge — Step 17

Create and commit named Hydra parameters first. Map existing ShaderBus controls:

```supercollider
h.mapBus(analysis[\amplitude], \size, scale: 4, bias: 0.8);
h.mapBus(analysis[\onset], \red, scale: 0.8, bias: 0.2);
```

The full call is `h.mapBus(bus, parameter, channel=0, scale=1, bias=0,
cutoff=0, output=\o0)`. Its scalar mapping is
`max(busValue - cutoff, 0) * scale + bias`, clipped to ±1e6 for the numeric API.
ShaderBus applies its existing smoothing coefficient before this mapping.
Only changed values are sent, at most 30 times/second per mapping, with at most
64 mappings per Hydra controller and 64 targets per bus. Updates coalesce to the
latest value while uploads/presentation requests are active. They carry the
current output revision and use the existing scalar uniform endpoint; they do
not upload graphs or recompile shaders. Acknowledgements are not awaited by the
stream; normal renderer diagnostics report rejected values/handles.

`h.unmapBus(bus, parameter, output)` removes selected mappings; omitted parameter
removes all this controller's mappings from that bus. `h.free` unmaps its buses
without freeing buses owned by other code. ShaderBus.free clears its targets;
a stopped renderer is removed from streaming targets. Inactive bus references
are pruned before adding mappings. Manual sets may be overwritten by a subsequent mapped update; unmap first to
return a parameter to manual control. Unmap before replacing a graph with one
that omits its mapped handles; native rejection diagnostics remain active. Existing ShaderBus.map(Shader, uniform)
behavior is retained.

ShaderAnalysis already provides amplitude, pitch/confidence, centroid, flatness,
onset and zero-crossing buses. Select the existing analysis names and map them
as above; choose sensible scale/bias/cutoff for each unit. These scsynth UGen
features are not signal-processing equivalents of Hydra's WebAudio analysis.
No loudness standard or meter qualification is claimed.

Existing ShaderFFTTexture and ShaderWaveformTexture streams also expose
`fft.binBus(index, smoothing=0)` and `wave.pointBus(index, smoothing=0)`. They
return a ShaderBus fed from the same already-received buffer values, with at most
64 selected indices per stream. Repeated access returns the same bus; smoothing
is chosen on creation. Index validation comes from ShaderBuffer. Freeing the
stream frees these owned scalar buses. For example, after starting an existing
FFT stream:

```supercollider
h.mapBus(fft.binBus(3, 0.3), \size, scale: 0.02, bias: 0.8, cutoff: 0.01);
```

The bin frequency spacing is audio sample rate / FFT size. Values are the
existing raw Unpack1FFT magnitudes, not normalized WebAudio a.fft bands. Scaling,
threshold subtraction and smoothing provide useful native controls; they do not
reproduce WebAudio's FFT/equalization/aggregation exactly. Waveform points may
be signed; the cutoff mapping rectifies below its threshold. These accessors add
no second array stream or GPU readback. Starting the existing texture producers
still incurs their normal full-buffer traffic at their configured rate. Direct
Hydra shader sampling of those buffer textures remains unsupported.

`examples/hydra/04_audio_reactive.scd` uses a looping ExampleFiles PlayBuf, a
private audio bus, a synchronized Buffer/SynthDef setup, quiet monitoring and
fade-in. A 2 Hz gate gives reproducible amplitude changes and attacks. Amplitude
drives scale and onsets drive red intensity; a commented SoundIn alternative
is provided. Start once and use its cleanup block before restarting. Cleanup
frees only its source/analysis/buffer/bus/visual resources; the audio server stays
available. Existing onset SendReply sampling is reused and may miss short events;
this is not an event-loss-free onset transport.

The installed check mutes monitoring while retaining the analysis signal. It
checks amplitude range, onset response, live FFT-bin/waveform-point values,
accepted two-parameter mappings and unmapping on free. Separate legacy analysis,
ShaderBus mapping and Hydra smokes remain enabled. Manual listening/visual
observation and other backends remain separate; see VERIFICATION.md.
