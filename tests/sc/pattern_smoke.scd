(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var server = ShaderServer.new(\scshaderPatternSmoke, "127.0.0.1", 57153);
var definition, shader, window, player, completed = false, gotQueuedStatus = false, startPattern, finish;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        player.tryPerform(\stop);
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_PATTERN_OK".postln; 0.exit }, {
                "SCSHADER_PATTERN_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

startPattern = {
    if(shader.isRunning) {
        player = Pbind(
            \type, \shader,
            \shader, \patternSmoke,
            \window, window,
            \amount, Pseq([0.2, 0.4, 0.6], 1),
            \dur, 0.07,
            \latency, 0.15
        ).play(TempoClock.default);
        SystemClock.sched(0.04, {
            server.status { |receiver, status|
                if(status[\scheduledEventCount] >= 1) { gotQueuedStatus = true };
            };
            nil
        });
        SystemClock.sched(0.45, {
            (type: \shader, shaderID: shader.id, amount: 0.8, latency: 0.15).play;
            nil
        });
        SystemClock.sched(0.85, {
            if(gotQueuedStatus and: { shader.value(\amount) == 0.8 }
                and: { Shader.atName(\patternSmoke) === shader }
                and: { shader.isFreed.not }) {
                finish.value(true, "SCSHADER_PATTERN_ROUNDTRIP");
            } {
                finish.value(false, "SCSHADER_PATTERN_BAD_STATE");
            };
            nil
        });
    } {
        SystemClock.sched(0.02, { startPattern.value; nil });
    };
};

if(PathName(rendererPath).isFile.not or: { PathName(shaderPath).isFile.not }) {
    finish.value(false, "SCSHADER_PATTERN_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader pattern smoke", server);
    server.waitForBoot({ |shaderServer|
        definition = ShaderDef(\patternSmoke, shaderPath);
        shader = Shader(definition, #[], shaderServer);
        startPattern.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_PATTERN_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(10.0, {
        finish.value(false, "SCSHADER_PATTERN_TIMEOUT");
        nil
    });
};
)
