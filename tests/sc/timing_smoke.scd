(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var server = ShaderServer.new(\scshaderTimingSmoke, "127.0.0.1", 57146);
var complete = false, gotQueuedStatus = false, finish;

finish = { |passed, message|
    if(complete.not) {
        complete = true;
        message.postln;
        server.quit({
            passed.if({ "SCSHADER_TIMING_OK".postln; 0.exit }, {
                "SCSHADER_TIMING_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(2.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

if(PathName(rendererPath).isFile.not) {
    finish.value(false, "SCSHADER_TIMING_MISSING_RENDERER");
} {
    server.rendererPath_(rendererPath);
    server.waitForBoot({ |shaderServer|
        shaderServer.clockSync(4, 0.02, 1.0, { |receiver, estimate|
            if(estimate[\sampleCount] < 4 or: { estimate[\rendererClockOffset].isNumber.not }
                or: { estimate[\roundTrip].isNumber.not }) {
                finish.value(false, "SCSHADER_TIMING_CLOCK_SYNC_FAILED");
            } {
                shaderServer.scheduleMarker(0.25, { |timingServer, timing|
                    var error = timing[\appliedRendererTime] - timing[\scheduledRendererTime];
                    if(gotQueuedStatus and: { error >= 0.0 and: { error < 0.20 } }) {
                        timingServer.scheduleMarker(-0.10, { |lateServer, lateTiming|
                            SystemClock.sched(0.05, {
                                if(lateServer.lastError.notNil
                                    and: { lateServer.lastError[\code] == "E_LATE_EVENT" }) {
                                    finish.value(true, "SCSHADER_TIMING_ROUNDTRIP");
                                } {
                                    finish.value(false, "SCSHADER_TIMING_LATE_DIAGNOSTIC_FAILED");
                                };
                                nil
                            });
                        });
                    } {
                        finish.value(false, "SCSHADER_TIMING_MARKER_FAILED: %".format(timing));
                    };
                }, [[
                    '/scshader/v1/uniform/f', 1, "amount", 0.9
                ]]);
                SystemClock.sched(0.08, {
                    shaderServer.status { |statusServer, status|
                        if(status[\scheduledEventCount] >= 1) { gotQueuedStatus = true };
                    };
                    nil
                });
            };
        });
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_TIMING_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(8.0, {
        finish.value(false, "SCSHADER_TIMING_TIMEOUT");
        nil
    });
};
)
