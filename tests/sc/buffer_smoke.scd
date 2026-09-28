(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var server = ShaderServer.new(\scshaderBufferSmoke, "127.0.0.1", 57157);
var window, definition, shader, buffer, completed = false, finish, start;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        buffer.tryPerform(\free);
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_BUFFER_OK".postln; 0.exit }, {
                "SCSHADER_BUFFER_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

start = {
    if(shader.isRunning) {
        buffer = ShaderBuffer.float(16384, server);
        buffer.setn(0, Array.fill(16384, { |index| index / 16383 }));
        shader.setBuffer(\spectrum, buffer);
        SystemClock.sched(0.45, {
            server.status { |receiver, status|
                if(
                    status[\bufferCount] == 1
                    and: { (buffer.value(16383) - 1.0).abs < 0.0001 }
                    and: { server.lastError.isNil }
                ) {
                    buffer.free;
                    SystemClock.sched(0.2, {
                        server.status { |nextReceiver, nextStatus|
                            if(nextStatus[\bufferCount] == 0) {
                                finish.value(true, "SCSHADER_BUFFER_BLOB_ROUNDTRIP");
                            } {
                                finish.value(false, "SCSHADER_BUFFER_FREE_FAILED");
                            };
                        };
                        nil
                    });
                } {
                    finish.value(false, "SCSHADER_BUFFER_BAD_STATUS: %".format(status));
                };
            };
            nil
        });
    } {
        SystemClock.sched(0.02, { start.value; nil });
    };
};

if([rendererPath, shaderPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_BUFFER_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader buffer smoke", server);
    server.waitForBoot({ |shaderServer|
        definition = ShaderDef(\bufferSmoke, shaderPath);
        shader = Shader(definition, #[], shaderServer);
        start.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_BUFFER_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(12.0, { finish.value(false, "SCSHADER_BUFFER_TIMEOUT"); nil });
};
)
