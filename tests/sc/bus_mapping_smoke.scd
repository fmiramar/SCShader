(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var server = ShaderServer.new(\scshaderBusSmoke, "127.0.0.1", 57156);
var window, definition, shader, bus, completed = false, finish, start;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        bus.tryPerform(\free);
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_BUS_OK".postln; 0.exit }, {
                "SCSHADER_BUS_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

start = {
    if(shader.isRunning) {
        bus = ShaderBus.control(1, 0.0);
        shader.map(\amount, bus);
        bus.set(0.73);
        bus.flush;
        SystemClock.sched(0.25, {
            if((shader.value(\amount) - 0.73).abs < 0.0001 and: { server.lastError.isNil }) {
                finish.value(true, "SCSHADER_BUS_MAPPING_ROUNDTRIP");
            } {
                finish.value(false, "SCSHADER_BUS_MAPPING_BAD_VALUE: %".format(shader.value(\amount)));
            };
            nil
        });
    } {
        SystemClock.sched(0.02, { start.value; nil });
    };
};

if([rendererPath, shaderPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_BUS_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader bus smoke", server);
    server.waitForBoot({ |shaderServer|
        definition = ShaderDef(\busSmoke, shaderPath);
        shader = Shader(definition, #[], shaderServer);
        start.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_BUS_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(10.0, { finish.value(false, "SCSHADER_BUS_TIMEOUT"); nil });
};
)
