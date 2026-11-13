(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\hydraModulationSmoke, "127.0.0.1", 57189);
var h, finished = false, finish, cases, run, index = 0;
finish = { |ok, message|
    if(finished.not) {
        finished = true; message.postln; h.tryPerform(\free);
        server.quit { if(ok) { "SCSHADER_HYDRA_MODULATION_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
cases = [\modulateRepeat,\modulateRepeatX,\modulateRepeatY,\modulateKaleid,\modulateScrollX,\modulateScrollY,\modulate,\modulateScale,\modulatePixelate,\modulateRotate,\modulateHue].collect { |name|
    { h.osc(10).modulate(h.noise(3),0.1).perform(name,h.gradient(0.2).rotate(0.15)).rotate(0.2) }
};
run = {
    if(index == cases.size) { h.osc(10).modulateKaleid(h.noise(3),0).out(\o0, { |bad|
            if(bad[\ok] or: { bad[\code]!="E_HYDRA_COMPILE" }) { finish.value(false,bad) } {
                h.osc(10).modulate(h.noise(3),0.1).out(\o0, { |good|
                    finish.value(good[\ok] and: { good[\revision]==12 }, "Eleven spatial modulations, bad edit rejection and unchanged revision passed");
                });
            };
        }) } {
        cases[index].value.out(\o0, { |result|
            if(result[\ok].not or: { result[\revision] != (index+1) }) { finish.value(false, result) } {
                h.render(\o0, { |display|
                    if(display[\ok].not) { finish.value(false, display) } {
                        index = index+1; SystemClock.sched(0.15, { run.value; nil });
                    };
                });
            };
        });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug/scshader-renderer"));
server.waitForBoot({ h=SCHydra.new(server); h.waitForReady({ run.value }, { |receiver,error| finish.value(false,error) }) },
    { |receiver,error| finish.value(false,error) });
SystemClock.sched(15, { finish.value(false,"Modulation timeout"); nil });
)
