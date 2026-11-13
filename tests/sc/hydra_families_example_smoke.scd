(
var done = false, finish, text;
finish = { |ok|
    if(done.not) {
        done = true; ~hydraFamilies.tryPerform(\free);
        ~hydraFamiliesServer.quit { if(ok) { "SCSHADER_HYDRA_FAMILIES_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
// Interpret the installed startup block as text, as in an unsaved IDE document.
text = File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/02_procedural.scd");
text.copyRange(0, text.find("// Evaluate each replacement") - 1).interpret;
SystemClock.sched(2, {
    if(~hydraFamilies.notNil and: { ~hydraFamilies.isReady and: { ~hydraFamilies.lastError.isNil } }) {
        ~hydraFamilies.render(\o0, { |result| finish.value(result[\ok]) });
    } { finish.value(false) };
    nil
});
SystemClock.sched(10, { finish.value(false); nil });
)
