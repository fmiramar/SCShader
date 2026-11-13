(
var done = false, finish, text;
finish = { |ok|
    if(done.not) {
        done = true; ~hydraFour.tryPerform(\free);
        ~hydraFourServer.quit { if(ok) { "SCSHADER_HYDRA_FOUR_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
// Interpret the installed startup block as text, as in an unsaved IDE document.
text = File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/02_four_outputs.scd");
text.copyRange(0, text.find("// Evaluate each later block") - 1).interpret;
SystemClock.sched(2, {
    if(~hydraFour.notNil and: { ~hydraFour.isReady and: { ~hydraFour.lastError.isNil } }) {
        ~hydraFour.render(\o0, { |result| finish.value(result[\ok]) });
    } { finish.value(false) };
    nil
});
SystemClock.sched(10, { finish.value(false); nil });
)
