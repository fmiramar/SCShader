(
var server = ShaderServer.new(\scshaderProcessExitSmoke, "127.0.0.1", 57148);
var complete = false;
var finish = { |passed, message|
    if(complete.not) {
        complete = true;
        message.postln;
        server.free;
        passed.if({ "SCSHADER_PROCESS_EXIT_OK".postln; 0.exit }, {
            "SCSHADER_PROCESS_EXIT_FAILED".postln; 1.exit
        });
    };
};

if(thisProcess.platform.name == \windows) {
    server.rendererPath_("SystemRoot".getenv +/+ "System32/cmd.exe");
    server.rendererArgs_(["/c", "exit", "1"]);
} {
    server.rendererPath_("/usr/bin/false");
};
server.waitForBoot({ |shaderServer|
    finish.value(false, "SCSHADER_PROCESS_EXIT_UNEXPECTED_READY");
}, { |shaderServer, error|
    if(shaderServer.isRunning.not and: { shaderServer.isBooting.not }
        and: { error[\code] == "E_BOOT" }) {
        finish.value(true, "SCSHADER_PROCESS_EXIT_DETECTED");
    } {
        finish.value(false, "SCSHADER_PROCESS_EXIT_BAD_STATE: %".format(error));
    };
});
SystemClock.sched(3.0, {
    finish.value(false, "SCSHADER_PROCESS_EXIT_TIMEOUT");
    nil
});
)
