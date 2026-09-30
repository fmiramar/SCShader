(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var renderer = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var server = ShaderServer.new(\windowCloseRegression, "127.0.0.1", 57165);
var window = ShaderWindow(640, 360, "SCShader close regression", server);
var shader, definition, completed = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(completed.not) {
        completed = true;
        window.free;
        definition.tryPerform(\free);
        server.free;
        SystemClock.sched(0.5, {
            passed.if({ "SCSHADER_WINDOW_CLOSE_OK".postln; 0.exit }, { 1.exit }); nil
        });
    };
};
server.rendererPath_(renderer);
Routine({
    var ready = false, failed = false, deadline, firstPID;
    try {
        server.waitForBoot { ready = true };
        deadline = Main.elapsedTime + 6;
        while({ ready.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(ready, "Initial boot failed");
        definition = ShaderDef(\windowCloseFixture, project +/+ "renderer/shaders/fullscreen.wgsl");
        shader = Shader(definition, #[], server);
        deadline = Main.elapsedTime + 3;
        while({ shader.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(shader.isRunning, "Shader creation failed");
        firstPID = server.pid;
        // The renderer uses exit 0 for both this command and the native close button.
        // Neither path calls ShaderServer.quit before the child actually exits.
        window.close;
        deadline = Main.elapsedTime + 4;
        while({ server.pid.notNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(server.pid.isNil and: { server.isRunning.not and: { server.isBooting.not } },
            "Closed renderer did not stop the controller");
        assert.value(server.lastError.isNil, "Normal close was incorrectly reported as a boot error");
        assert.value(shader.isFreed and: { Shader.at(shader.id).isNil }, "Closed renderer retained a Shader");
        ready = false;
        server.waitForBoot { ready = true };
        deadline = Main.elapsedTime + 6;
        while({ ready.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(ready and: { server.pid != firstPID }, "Controller could not boot after native close");
        server.sync;
        server.quit;
        deadline = Main.elapsedTime + 4;
        while({ server.pid.notNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(server.pid.isNil, "Restarted renderer did not quit");
        // Exit 0 before a handshake must still fail boot, not become a clean close.
        if(thisProcess.platform.name == \windows) {
            server.rendererPath_("SystemRoot".getenv +/+ "System32/cmd.exe");
            server.rendererArgs_(["/c", "exit", "0"]);
        } {
            server.rendererPath_("/usr/bin/true");
        };
        ready = false;
        server.waitForBoot({ ready = true }, { |controller, error| failed = error[\code] == "E_BOOT" });
        deadline = Main.elapsedTime + 3;
        while({ failed.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        assert.value(failed and: { ready.not and: { server.isRunning.not } },
            "Exit 0 before handshake was not reported as E_BOOT");
        finish.value(true);
    } { |error| error.reportError; finish.value(false) };
}).play(SystemClock);
SystemClock.sched(25, { finish.value(false); nil });
)
