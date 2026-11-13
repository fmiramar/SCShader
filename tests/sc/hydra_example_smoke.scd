(
var done = false, finish;
finish = { |ok|
    if(done.not) {
        done = true; h.tryPerform(\free);
        v.quit { if(ok) { "SCSHADER_HYDRA_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/01_osc.scd").interpret;
SystemClock.sched(2, {
    if(h.notNil and: { h.isReady and: { h.lastError.isNil } }) {
        h.render(\o0, { |result| finish.value(result[\ok]) });
    } { finish.value(false) };
    nil
});
SystemClock.sched(10, { finish.value(false); nil });
)
