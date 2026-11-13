(
var done = false, finish, text;
finish = { |ok|
    if(done.not) {
        done = true; ~hydraCombine.tryPerform(\free);
        ~hydraCombineServer.quit { if(ok) { "SCSHADER_HYDRA_COMBINE_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
// Interpret the installed startup block as text, as in an unsaved IDE document.
text = File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/03_combine.scd");
text.copyRange(0, text.find("// Evaluate this replacement") - 1).interpret;
SystemClock.sched(2, {
    if(~hydraCombine.notNil and: { ~hydraCombine.isReady and: { ~hydraCombine.lastError.isNil } }) {
        ~hydraCombine.render(\o0, { |result| finish.value(result[\ok]) });
    } { finish.value(false) };
    nil
});
SystemClock.sched(10, { finish.value(false); nil });
)
