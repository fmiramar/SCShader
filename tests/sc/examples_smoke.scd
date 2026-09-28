(
var project = "SCSHADER_EXAMPLE_ROOT".getenv ? thisProcess.nowExecutingPath.dirname.dirname.dirname;
var completed = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(completed.not) {
        completed = true;
        ShaderServer.all.do { |server| server.free };
        SystemClock.sched(0.5, { passed.if({ "SCSHADER_EXAMPLES_OK".postln; 0.exit }, { 1.exit }); nil });
    };
};
Routine({
    try {
        [
            ["examples/01_shader_server.scd", nil, \environment],
            ["examples/02_shader_lifecycle.scd", \scShaderPulse, \environment],
            ["examples/03_shadertoy_compatibility.scd", \scshaderToyGradient, \interpreter],
            ["examples/04_typed_controls.scd", \typedControls, \interpreter]
        ].do { |entry|
            var source = File.readAllString(project +/+ entry[0]);
            var start = source.find("(\n"), end = source.find("\n)");
            var code, server, shader, deadline;
            assert.value(start.notNil and: { end.notNil }, "Missing runnable example block");
            // Execute verbatim, like pasting into an unsaved IDE document. All
            // guide/class-help blocks are covered by check_help_examples.py.
            thisProcess.nowExecutingPath = nil;
            ShaderServer.default.rendererPath_("/path/to/stale-renderer");
            code = source.copyRange(start, end + 1);
            code.interpret;
            server = (entry[2] == \environment).if({ ~scShaderServer }, { v });
            deadline = Main.elapsedTime + 6;
            while({ server.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(server.isRunning, "Example boot failed: %".format(entry[0]));
            if(entry[1].notNil) {
                shader = Shader.atName(entry[1]);
                while({ shader.isNil or: { shader.isRunning.not } and: { Main.elapsedTime < deadline } }, {
                    0.01.wait; shader = Shader.atName(entry[1]);
                });
                assert.value(shader.notNil and: { shader.isRunning }, "Example shader failed: %".format(entry[0]));
            };
            1.0.wait;
            assert.value(server.lastError.isNil, "Example renderer diagnostic: %".format(server.lastError));
            assert.value(shader.isNil or: { shader.lastError.isNil }, "Example shader diagnostic");
            shader.tryPerform(\free);
            server.quit;
            deadline = Main.elapsedTime + 3;
            while({ server.isRunning and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(server.isRunning.not, "Example renderer failed to quit");
            server.window.tryPerform(\free);
            ["EXAMPLE_OK", entry[0]].postln;
        };
        finish.value(true);
    } { |error| error.reportError; finish.value(false) };
}).play(SystemClock);
SystemClock.sched(40, { finish.value(false); nil });
)
