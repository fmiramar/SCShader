(
var file = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/05_feedback.scd";
var text = File.readAllString(file), start, end, done = false, finish;
finish = { |ok, message|
    if(done.not) {
        done = true; message.postln; ~hydraFeedback.tryPerform(\free);
        ~hydraFeedbackServer.tryPerform(\quit, {
            if(ok) { "SCSHADER_HYDRA_FEEDBACK_OK".postln; 0.exit } { 1.exit };
        });
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
start = text.find("// BEGIN STARTUP") + "// BEGIN STARTUP".size;
end = text.find("// END STARTUP");
text.copyRange(start, end - 1).replace("57194", "57196").interpret;
Routine({
    while { ~hydraFeedbackReady != true and: { done.not } } { 0.1.wait };
    if(done.not) {
        0.5.wait;
        ~hydraFeedbackCheck.value(2, { |cross|
            if(cross[\ok].not) { finish.value(false, cross) } {
                ~hydraFeedbackServer.sendMsg('/scshader/v1/window/resize', 320, 240);
                SystemClock.sched(0.5, {
                    ~hydraFeedbackCheck.value(3, { |quiet|
                        finish.value(quiet[\ok] and: { ~hydraFeedbackNext == 4 },
                            "Installed feedback example: self, cross cycle, prev, resize and hush acknowledged");
                    }); nil
                });
            };
        });
    };
}).play(SystemClock);
SystemClock.sched(20, { finish.value(false, "Feedback example timeout"); nil });
)
