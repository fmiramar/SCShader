(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\hydraFamiliesSmoke, "127.0.0.1", 57186);
var h, finished = false, finish, cases, run, index = 0;
finish = { |ok, message|
    if(finished.not) {
        finished = true; message.postln; h.tryPerform(\free);
        server.quit { if(ok) { "SCSHADER_HYDRA_FAMILIES_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
cases = [
    { h.noise(4,0.15).kaleid(5).color(0.25,0.25,0.25).brightness(0.5).posterize(5,0.7).color(0.3,0.65,1) },
    { h.shape(5,0.35,0.015).repeat(3,2,0.3,-0.2).rotate(0.2,0.05).color(0.7,0.3,1).invert(0.2) },
    { h.voronoi(6,0.3,0.3).scrollX(0.1,0.03).color(0.25,0.7,1).colorama(0.03) },
    { h.solid(0.2,0.4,0.6,0.8).sum([0.1,0.2,0.3,0.4]).g(0.5,0.1) }
];
run = {
    if(index == cases.size) { finish.value(true, "Procedural families and vector sum compiled, committed and displayed") } {
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
SystemClock.sched(15, { finish.value(false,"Procedural family timeout"); nil });
)
