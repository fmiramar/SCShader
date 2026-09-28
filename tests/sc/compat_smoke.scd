(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var glslPath = project +/+ "renderer/shaders/compat_gradient.glsl";
var shaderToyPath = project +/+ "renderer/shaders/shadertoy_gradient.frag";
var mousePath = project +/+ "renderer/shaders/shadertoy_mouse_date.frag";
var channelPath = project +/+ "renderer/shaders/shadertoy_channel_loop.frag";
var server = ShaderServer.new(\scshaderCompatibilitySmoke, "127.0.0.1", 57160);
var window, glslDef, toyDef, mouseDef, channelDef, glslShader, toyShader, mouseShader, channelShader;
var completed = false, finish, waitForShaders;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        toyShader.tryPerform(\free);
        glslShader.tryPerform(\free);
        mouseShader.tryPerform(\free);
        channelShader.tryPerform(\free);
        toyDef.tryPerform(\free);
        glslDef.tryPerform(\free);
        mouseDef.tryPerform(\free);
        channelDef.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_COMPAT_OK".postln; 0.exit }, {
                "SCSHADER_COMPAT_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

waitForShaders = {
    if(glslShader.isRunning and: { toyShader.isRunning and: {
        mouseShader.isRunning and: { channelShader.isRunning } } }) {
        SystemClock.sched(0.25, {
            server.status { |receiver, status|
                if(status[\shaderCount] == 4 and: { server.lastError.isNil }) {
                    finish.value(true, "SCSHADER_COMPAT_NAGA_GLSL_AND_SHADERTOY_ROUNDTRIP");
                } {
                    finish.value(false, "SCSHADER_COMPAT_BAD_STATUS: %".format(status));
                };
            };
            nil
        });
    } {
        SystemClock.sched(0.02, { waitForShaders.value; nil });
    }
};

if([rendererPath, glslPath, shaderToyPath, mousePath, channelPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_COMPAT_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader compatibility smoke", server);
    server.waitForBoot({ |shaderServer|
        glslDef = ShaderDef(\compatGLSL, glslPath, nil, \glsl);
        toyDef = ShaderToy(\compatShaderToy, shaderToyPath);
        mouseDef = ShaderToy(\compatShaderToyMouse, mousePath);
        channelDef = ShaderToy(\compatShaderToyChannel, channelPath);
        glslShader = Shader(glslDef, [\amount, 0.25], shaderServer);
        toyShader = Shader(toyDef, [\amount, 0.8], shaderServer);
        mouseShader = Shader(mouseDef, #[], shaderServer);
        channelShader = Shader(channelDef, #[], shaderServer);
        waitForShaders.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_COMPAT_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(12.0, { finish.value(false, "SCSHADER_COMPAT_TIMEOUT"); nil });
};
)
