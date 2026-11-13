(
var done = false, finish, text;
finish = { |ok|
    if(done.not) {
        done = true; ~hydraMod.tryPerform(\free);
        ~hydraModServer.quit { if(ok) { "SCSHADER_HYDRA_MODULATION_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
// Interpret the installed startup block as text, as in an unsaved IDE document.
text = File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/04_modulation.scd");
text.copyRange(0, text.find("// Evaluate this replacement") - 1).interpret;
SystemClock.sched(2, {
    if(~hydraMod.notNil and: { ~hydraMod.isReady and: { ~hydraMod.lastError.isNil } }) {
        ~hydraMod.render(\o0, { |result| finish.value(result[\ok]) });
    } { finish.value(false) };
    nil
});
SystemClock.sched(10, { finish.value(false); nil });
)
