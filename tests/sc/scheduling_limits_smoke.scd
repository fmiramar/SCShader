(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\scheduleLimitsTest, "127.0.0.1", 57167);
var complete = false, markerApplied = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(complete.not) {
        complete = true;
        server.quit;
        SystemClock.sched(0.5, {
            passed.if({ "SCSHADER_SCHEDULING_LIMITS_OK".postln; 0.exit }, { 1.exit }); nil
        });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName));
server.waitForBoot {
    Routine({
        var readStatus, reply, deadline;
        readStatus = {
            reply = nil;
            server.status { |receiver, value| reply = value };
            deadline = Main.elapsedTime + 2;
            while({ reply.isNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(reply.notNil, "Status timed out");
            reply
        };
        try {
            server.scheduleMarker(0.75, { markerApplied = true });
            reply = readStatus.value;
            assert.value(reply[\scheduledEventCount] == 2, "Expected one bundle and one marker");
            assert.value(reply[\scheduledPayloadBytes] > 0, "Missing scheduler byte accounting");
            assert.value(reply[\rejectedScheduled] == 0, "Unexpected scheduler rejection");
            assert.value(reply[\droppedContinuous] == 0, "Unexpected receiver overload");
            server.clearScheduled;
            reply = readStatus.value;
            assert.value(reply[\scheduledEventCount] == 0, "Clear left pending events");
            assert.value(reply[\scheduledPayloadBytes] == 0, "Clear left charged payload");
            0.9.wait;
            assert.value(markerApplied.not, "Cleared marker still ran");
            server.scheduleMarker(0.1, { markerApplied = true });
            deadline = Main.elapsedTime + 2;
            while({ markerApplied.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(markerApplied, "Scheduler did not accept work after clearing");
            assert.value(server.lastError.isNil, "Unexpected renderer diagnostic");
            finish.value(true);
        } { |error| error.reportError; finish.value(false) };
    }).play(SystemClock);
};
SystemClock.sched(12, { finish.value(false); nil });
)
