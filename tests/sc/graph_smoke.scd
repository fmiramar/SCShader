(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var shaderPath = project +/+ "renderer/shaders/fullscreen.wgsl";
var server = ShaderServer.new(\scshaderGraphSmoke, "127.0.0.1", 57159);
var window, defA, defB, shaderA, shaderB, graph, completed = false, finish, start;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        graph.tryPerform(\free);
        shaderB.tryPerform(\free);
        shaderA.tryPerform(\free);
        defB.tryPerform(\free);
        defA.tryPerform(\free);
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_GRAPH_OK".postln; 0.exit }, {
                "SCSHADER_GRAPH_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

start = {
    if(shaderA.isRunning and: { shaderB.isRunning }) {
        graph = ShaderGraph(server);
        graph.add(shaderA).connect(shaderA, \out, shaderB, \source);
        if(graph.commit.size != 2) {
            finish.value(false, "SCSHADER_GRAPH_BAD_ORDER");
        } {
            SystemClock.sched(0.5, {
                server.status { |receiver, status|
                    if(status[\shaderCount] == 2 and: { server.lastError.isNil }) {
                        finish.value(true, "SCSHADER_GRAPH_MULTIPASS_ROUNDTRIP");
                    } {
                        finish.value(false, "SCSHADER_GRAPH_BAD_STATUS: %".format(status));
                    };
                };
                nil
            });
        };
    } {
        SystemClock.sched(0.02, { start.value; nil });
    };
};

if([rendererPath, shaderPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_GRAPH_MISSING_FIXTURE");
} {
    server.rendererPath_(rendererPath);
    window = ShaderWindow.new(640, 360, "SCShader graph smoke", server);
    server.waitForBoot({ |shaderServer|
        defA = ShaderDef(\graphSmokeA, shaderPath);
        defB = ShaderDef(\graphSmokeB, shaderPath);
        shaderA = Shader(defA, [\amount, 0.25], shaderServer);
        shaderB = Shader(defB, [\amount, 0.8], shaderServer);
        start.value;
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_GRAPH_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(12.0, { finish.value(false, "SCSHADER_GRAPH_TIMEOUT"); nil });
};
)
