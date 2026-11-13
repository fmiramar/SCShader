(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\hydraTransportSmoke, "127.0.0.1", 57183);
var hydra, definition, shader, completed = false, finish;
finish = { |passed, message|
    if(completed.not) {
        completed = true; message.postln;
        hydra.tryPerform(\free); shader.tryPerform(\free); definition.tryPerform(\free);
        server.quit { passed.if({ "SCSHADER_HYDRA_TRANSPORT_OK".postln; 0.exit }, { 1.exit }) };
        SystemClock.sched(2, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug/scshader-renderer"));
server.waitForBoot({
    definition = ShaderDef(\hydraTransportLegacy, project +/+ "renderer/shaders/fullscreen.wgsl");
    shader = Shader(definition, [\amount, 0.4], server);
    hydra = SCHydra.new(server);
    hydra.waitForReady({
        hydra.osc(10).rotate(0.2).out(\o0, { |result|
            if(result[\ok].not and: { result[\code] == "E_HYDRA_CAPABILITY" }) {
                server.status { |receiver, status|
                    finish.value(status[\shaderCount] == 1, "Hydra IR roundtrip rejected activation; legacy shader retained");
                };
            } { finish.value(false, "Unexpected Hydra upload result: %".format(result)) };
        });
    }, { |receiver, error| finish.value(false, "Hydra negotiation failed: %".format(error)) });
}, { |receiver, error| finish.value(false, "Renderer boot failed: %".format(error)) });
SystemClock.sched(15, { finish.value(false, "Hydra transport smoke timed out"); nil });
)
