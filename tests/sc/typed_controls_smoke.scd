(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var server = ShaderServer.new(\typedControlsTest, "127.0.0.1", 57163);
var fixture = project +/+ "renderer/shaders/typed_controls.wgsl";
var reloadPath = project +/+ "build/typed-controls-reload.wgsl";
var source = File.readAllString(fixture);
var shader, second, definition, completed = false;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var finish = { |passed|
    if(completed.not) {
        completed = true;
        server.quit;
        SystemClock.sched(0.5, { File.delete(reloadPath); passed.if({ "SCSHADER_TYPED_CONTROLS_OK".postln; 0.exit }, { 1.exit }); nil });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName));
server.waitForBoot {
    Routine({
        var read, value, received, deadline, reloads, rejected;
        read = { |target, name|
            received = false;
            target.get(name, { |reply| value = reply; received = true });
            deadline = Main.elapsedTime + 2;
            while({ received.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(received, "Uniform get timed out");
            value
        };
        try {
            File.use(reloadPath, "w", { |file| file.write(source) });
            definition = ShaderDef(\typedControlsFixture, reloadPath);
            shader = Shader(definition, [\amount, 0.75], server);
            deadline = Main.elapsedTime + 5;
            while({ shader.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(shader.isRunning, "Typed shader creation timed out");
            assert.value(shader.uniforms.size == 11, "Incomplete renderer reflection");
            [
                [\gain, 0.25], [\count, -2147483648], [\seed, 4294967295.0], [\enabled, true],
                [\offset, [0.25, -0.5]], [\color, [1, 2, 3]], [\tint, [1, 2, 3, 4]],
                [\rotate, [1, 2, 3, 4]], [\transform, (1..9)], [\projection, (1..16)]
            ].do { |pair|
                shader.set(*pair);
                assert.value(read.value(shader, pair[0]) == pair[1], "Renderer value mismatch: %".format(pair));
            };
            rejected = false;
            try { shader.set(\gain, 0.9, \offset, [1]) } { rejected = true };
            assert.value(rejected and: { (read.value(shader, \gain) - 0.25).abs < 0.00001 }, "Invalid pair list partially changed an earlier control");
            rejected = false;
            try { shader.glide(\count, 1, 0.1) } { rejected = true };
            assert.value(rejected, "Integer interpolation was not rejected before dispatch");
            shader.set(\count, 2147483647, \seed, 0, \enabled, false);
            assert.value(read.value(shader, \count) == 2147483647, "Signed integer maximum failed");
            assert.value(read.value(shader, \seed) == 0, "Unsigned zero failed");
            assert.value(read.value(shader, \enabled) == false, "Bool false failed");
            second = Shader(ShaderDef(\typedSecondFixture, fixture), [\amount, 0.2], server);
            deadline = Main.elapsedTime + 5;
            while({ second.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(second.isRunning, "Second shader creation failed");
            second.set(\gain, 0.8);
            assert.value((read.value(shader, \gain) - 0.25).abs < 0.00001, "Shader controls leaked across instances");
            shader.set(\gain, 0);
            shader.glide(\gain, 1, 1.0, \linear);
            0.4.wait;
            value = read.value(shader, \gain);
            assert.value(value > 0.2 and: { value < 0.85 }, "Linear glide did not interpolate");
            shader.glide(\gain, 0, 0.2, \smooth);
            0.35.wait;
            assert.value(read.value(shader, \gain) == 0, "Smooth glide did not finish");
            shader.glide(\color, [0.2, 0.4, 0.6], 0.1);
            0.2.wait;
            assert.value((read.value(shader, \color) - [0.2, 0.4, 0.6]).abs.maxItem < 0.00001, "Vector glide failed");
            shader.setAt(0.15, \count, 17, \offset, [0.5, 0.75]);
            0.3.wait;
            assert.value(read.value(shader, \count) == 17, "Timestamped typed set failed");
            reloads = shader.reloadCount;
            shader.reload;
            deadline = Main.elapsedTime + 5;
            while({ shader.reloadCount == reloads and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(shader.reloadCount > reloads, "Typed shader reload failed");
            assert.value(read.value(shader, \count) == 17, "Reload discarded matching control value");
            assert.value(server.lastError.isNil, "Unexpected renderer diagnostic: %".format(server.lastError));
            // The first Shader is not the selected pass. Its watch must still run,
            // and an invalid edit must preserve its pipeline and typed controls.
            File.use(reloadPath, "w", { |file| file.write("this is not WGSL") });
            deadline = Main.elapsedTime + 5;
            while({ shader.lastError.isNil and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(shader.lastError.notNil and: { shader.lastError[\code] == "E_SHADER_COMPILE" }, "Inactive shader hot-reload failure was not reported");
            assert.value(read.value(shader, \count) == 17, "Invalid reload discarded last-valid controls");
            reloads = shader.reloadCount;
            File.use(reloadPath, "w", { |file|
                file.write(source.replace("gain: f32", "gain: u32").replace("* controls.gain", "* f32(controls.gain)"))
            });
            deadline = Main.elapsedTime + 5;
            while({ shader.reloadCount == reloads and: { Main.elapsedTime < deadline } }, { 0.01.wait });
            assert.value(shader.reloadCount > reloads, "Inactive shader did not recover after valid edit");
            assert.value(shader.uniforms[\gain] == \uint, "Reload reflection did not replace the old type");
            assert.value(read.value(shader, \gain) == 0, "Type-changed control did not reset");
            assert.value(read.value(shader, \count) == 17, "Reload changed an unrelated control");
            shader.free; second.free;
            finish.value(true);
        } { |error| error.reportError; finish.value(false) };
    }).play(SystemClock);
};
SystemClock.sched(20, { finish.value(false); nil });
)
