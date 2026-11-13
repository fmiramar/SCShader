(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\hydraOutputSmoke, "127.0.0.1", 57184);
var h, completed = false, finish;
finish = { |ok, message|
    if(completed.not) {
        completed = true; message.postln; h.tryPerform(\free);
        server.quit { if(ok) { "SCSHADER_HYDRA_OUTPUT_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug/scshader-renderer"));
server.waitForBoot({
    h = SCHydra.new(server);
    h.waitForReady({
        h.osc(10).rotate(0.2).out(\o0, { |first|
            if(first[\ok].not or: { first[\revision] != 1 }) { finish.value(false, first) } {
                h.render(\o0, { |shown|
                    if(shown[\ok].not) { finish.value(false, shown) } {
                        h.osc(0).out(\o0, { |bad|
                            if(bad[\ok] or: { bad[\code] != "E_HYDRA_COMPILE" }) { finish.value(false, bad) } {
                                h.solid(0.2,0.4,0.6).out(\o0, { |second|
                                    if(second[\ok].not or: { second[\revision] != 2 }) { finish.value(false, second) } {
                                        server.sendMsg('/scshader/v1/window/resize', 320, 240);
                                        SystemClock.sched(0.25, {
                                            h.render(\legacy, { |legacy|
                                                if(legacy[\ok].not) { finish.value(false, legacy) } {
                                                    h.render(\o0, { |again| finish.value(again[\ok], "Hydra o0 commit, render, failed edit, replacement, resize and mode switch") });
                                                };
                                            }); nil
                                        });
                                    };
                                });
                            };
                        });
                    };
                });
            };
        });
    }, { |receiver, error| finish.value(false, error) });
}, { |receiver, error| finish.value(false, error) });
SystemClock.sched(15, { finish.value(false, "Hydra output timeout"); nil });
)
