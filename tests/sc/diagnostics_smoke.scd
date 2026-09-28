(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\diagnosticsTest, "127.0.0.1", 57169);
var complete = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(complete.not) {
        complete = true;
        server.quit;
        SystemClock.sched(0.5, {
            passed.if({ "SCSHADER_DIAGNOSTICS_OK".postln; 0.exit }, { 1.exit }); nil
        });
    };
};
assert.value(server.showStats.not, "Overlay should default off");
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName));
server.showStats = true;
server.waitForBoot {
    Routine({
        var readStatus, reply, deadline, firstFrame;
        readStatus = {
            reply = nil;
            server.status { |receiver, value| reply = value };
            deadline = Main.elapsedTime + 2;
            while({ reply.isNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(reply.notNil, "Status timed out");
            reply
        };
        try {
            0.3.wait;
            reply = readStatus.value;
            firstFrame = reply[\frameIndex];
            assert.value(reply[\showStats], "Boot did not restore the overlay preference");
            assert.value(reply[\textureBytesEstimate] > 0, "Missing texture memory estimate");
            assert.value(reply[\lastCompileOK], "Built-in shader did not compile");
            assert.value(reply[\gpuRecoveryCount] == 0, "Unexpected recovery");
            assert.value(reply[\lateEventCount] == 0, "Unexpected late event");
            server.showStats = false;
            0.3.wait;
            reply = readStatus.value;
            assert.value(reply[\showStats].not, "Overlay did not turn off");
            assert.value(reply[\frameIndex] > firstFrame, "Rendering stopped");
            assert.value(server.lastError.isNil, "Unexpected renderer diagnostic");
            finish.value(true);
        } { |error| error.reportError; finish.value(false) };
    }).play(SystemClock);
};
SystemClock.sched(12, { finish.value(false); nil });
)
