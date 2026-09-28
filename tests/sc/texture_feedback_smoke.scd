(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var imagePath = project +/+ "tests/fixtures/two_color.ppm";
var server = ShaderServer.new(\scshaderTextureSmoke, "127.0.0.1", 57154);
var window, definition, shader, texture, completed = false, finish, start;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        texture.tryPerform(\free);
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_TEXTURE_OK".postln; 0.exit }, {
                "SCSHADER_TEXTURE_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

start = {
    if(shader.isRunning) {
        texture = ShaderTexture(imagePath, server);
        shader.setTexture(\source, texture);
        shader.feedback(\previous, 0.92);
        window.size_(800 @ 450);
        SystemClock.sched(0.45, {
            server.status { |receiver, status|
                if(status[\textureCount] == 1 and: { server.lastError.isNil }) {
                    texture.free;
                    SystemClock.sched(0.2, {
                        server.status { |nextReceiver, nextStatus|
                            if(nextStatus[\textureCount] == 0) {
                                finish.value(true, "SCSHADER_TEXTURE_FEEDBACK_ROUNDTRIP");
                            } {
                                finish.value(false, "SCSHADER_TEXTURE_FREE_FAILED");
                            };
                        };
                        nil
                    });
                } {
                    finish.value(false, "SCSHADER_TEXTURE_BAD_STATUS: %".format(status));
                };
            };
            nil
        });
    } {
        SystemClock.sched(0.02, { start.value; nil });
    };
};

if([rendererPath, shaderPath, imagePath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_TEXTURE_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader texture smoke", server);
    server.waitForBoot({ |shaderServer|
        definition = ShaderDef(\textureSmoke, shaderPath);
        shader = Shader(definition, #[], shaderServer);
        start.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_TEXTURE_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(10.0, { finish.value(false, "SCSHADER_TEXTURE_TIMEOUT"); nil });
};
)
