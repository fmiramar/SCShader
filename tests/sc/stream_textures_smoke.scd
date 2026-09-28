(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var shaderServer = ShaderServer.new(\scshaderStreamsSmoke, "127.0.0.1", 57158);
var audioServer = Server(\scshaderStreamsAudio, NetAddr("127.0.0.1", 57311));
var window, definition, shader, audioBus, source, fft, waveform, completed = false, finish, beginStreams;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        fft.tryPerform(\free);
        waveform.tryPerform(\free);
        source.tryPerform(\free);
        audioBus.tryPerform(\free);
        audioServer.quit;
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        window.tryPerform(\free);
        shaderServer.quit({
            passed.if({ "SCSHADER_STREAM_TEXTURES_OK".postln; 0.exit }, {
                "SCSHADER_STREAM_TEXTURES_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

beginStreams = {
    if(shader.isRunning) {
        audioServer.waitForBoot({
            Routine({
                audioBus = Bus.audio(audioServer, 1);
                SynthDef(\scshaderStreamsSource, { |out = 0|
                    Out.ar(out, SinOsc.ar(440, 0, 0.15));
                }).add;
                audioServer.sync;
                source = Synth(\scshaderStreamsSource, [\out, audioBus], audioServer);
                fft = ShaderFFTTexture(audioServer, audioBus.index, 1024, 20, shaderServer);
                waveform = ShaderWaveformTexture(audioServer, audioBus.index, 64, 0.02, 20, shaderServer);
                fft.start;
                waveform.start;
                shader.setTexture(\spectrum, fft);
                1.5.wait;
                shader.setTexture(\spectrum, waveform);
                if(
                    (fft.buffer.asArray.maxItem > 0.001)
                    and: { waveform.buffer.asArray.collect(_.abs).maxItem > 0.01 }
                    and: { shaderServer.lastError.isNil }
                ) {
                    finish.value(true, "SCSHADER_STREAM_TEXTURES_ROUNDTRIP");
                } {
                    finish.value(false, "SCSHADER_STREAM_TEXTURES_BAD_VALUES: %".format([
                        fft.buffer.asArray.maxItem,
                        waveform.buffer.asArray.collect(_.abs).maxItem,
                        shaderServer.lastError
                    ]));
                };
            }).play(SystemClock);
        }, { |failedServer|
            finish.value(false, "SCSHADER_STREAM_TEXTURES_AUDIO_BOOT_FAILED: %".format(failedServer));
        });
        audioServer.boot;
    } {
        SystemClock.sched(0.02, { beginStreams.value; nil });
    };
};

if([rendererPath, shaderPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_STREAM_TEXTURES_MISSING_FIXTURE");
} {
    shaderServer.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader stream textures smoke", shaderServer);
    shaderServer.waitForBoot({ |runningShaderServer|
        definition = ShaderDef(\streamTexturesSmoke, shaderPath);
        shader = Shader(definition, #[], runningShaderServer);
        beginStreams.value;
    }, { |failedShaderServer, error|
        finish.value(false, "SCSHADER_STREAM_TEXTURES_RENDERER_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(15.0, { finish.value(false, "SCSHADER_STREAM_TEXTURES_TIMEOUT"); nil });
};
)
