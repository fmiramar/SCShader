(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\restartRegression, "127.0.0.1", 57164);
var shader, definition, completed = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(completed.not) {
        completed = true;
        server.quit;
        SystemClock.sched(0.5, { passed.if({ "SCSHADER_REBOOT_OK".postln; 0.exit }, { 1.exit }); nil });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName));
Routine({
    var ready, deadline, oldPID;
    try {
        ready = false;
        server.waitForBoot { ready = true };
        deadline = Main.elapsedTime + 6;
        while({ ready.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(ready, "Initial boot failed");
        definition = ShaderDef(\restartFixture, project +/+ "renderer/shaders/fullscreen.wgsl");
        shader = Shader(definition, #[], server);
        deadline = Main.elapsedTime + 3;
        while({ shader.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(shader.isRunning, "Initial Shader creation failed");
        server.sync;
        assert.value(server.clockSampleCount > 0, "Initial clock sample absent");
        oldPID = server.pid;
        ready = false;
        server.reboot { ready = true };
        deadline = Main.elapsedTime + 6;
        while({ ready.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(ready and: { server.pid != oldPID }, "Reboot did not replace child");
        assert.value(shader.isFreed and: { Shader.at(shader.id).isNil }, "Old Shader survived renderer loss");
        assert.value(server.clockSampleCount == 0 and: { server.rendererClockOffset.isNil }, "Old clock estimate survived reboot");
        1.25.wait; // exceeds the previous child's one-second termination fallback
        assert.value(server.isRunning and: { server.lastError.isNil }, "Old quit timer killed replacement child");
        server.sync;
        ready = false;
        server.quit;
        server.waitForBoot { ready = true }; // boot requested while quit is in flight
        deadline = Main.elapsedTime + 6;
        while({ ready.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(ready, "Deferred boot during quit failed");
        1.25.wait;
        server.sync;
        assert.value(server.isRunning and: { server.lastError.isNil }, "Deferred restart was terminated");
        finish.value(true);
    } { |error| error.reportError; finish.value(false) };
}).play(SystemClock);
SystemClock.sched(25, { finish.value(false); nil });
)
