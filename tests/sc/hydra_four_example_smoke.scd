(
var done = false, finish, text, begin, end, waitForStartup, run;
finish = { |ok|
    if(done.not) {
        done = true; ~hydraFour.tryPerform(\free);
        ~hydraFourServer.quit { if(ok) { "SCSHADER_HYDRA_FOUR_EXAMPLE_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2, { if(ok) { 0.exit } { 1.exit }; nil });
    };
};
// Execute the installed startup and every actual diagnostic step as unsaved text.
// Use an owned port so the user's current demo on 57192 can keep running.
text = File.readAllString(Platform.userExtensionDir +/+ "SCShader/examples/hydra/02_four_outputs.scd");
begin = text.find("// BEGIN STARTUP"); end = text.find("// END STARTUP");
text.copyRange(begin, end - 1).replace("57192", "57193").interpret;
run = { |step|
    if(step > 9) { finish.value(true) } {
        ~hydraFourCheck.value(step, { |result|
            if(result[\ok].not or: { ~hydraFourTestNext != (step + 1) }) { finish.value(false) } {
                // Give the renderer time to display each accepted change.
                SystemClock.sched(0.15, { run.value(step + 1); nil });
            };
        });
    };
};
waitForStartup = {
    if(done.not) {
        if(~hydraFourTestReady == true) {
            // A skipped step must fail without advancing the sequence.
            ~hydraFourCheck.value(2, { |result|
                if(result[\ok] or: { ~hydraFourTestNext != 1 }) { finish.value(false) } { run.value(1) };
            });
        } { SystemClock.sched(0.1, { waitForStartup.value; nil }) };
    };
};
waitForStartup.value;
SystemClock.sched(20, { finish.value(false); nil });
)
