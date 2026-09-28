(
var source = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/typed_controls.wgsl";
v = ShaderServer.new(\exampleTyped);
w = ShaderWindow(960, 540, "SCShader typed controls", v);
v.waitForBoot { |server|
    d = ShaderDef(\typedControls, source);
    x = Shader(d, [\amount, 0.8], server);
    Routine({
        var deadline = Main.elapsedTime + 5;
        while({ x.isRunning.not and: { Main.elapsedTime < deadline } }, { 0.01.wait });
        if(x.isRunning) {
            x.set(\gain, 1, \count, 12, \enabled, true, \seed, 4294967295.0,
                \color, [0.1, 0.3, 0.9], \tint, [1, 0.3, 0.1, 1],
                \rotate, [1, 0, 0, 1], \transform, [1, 0, 0, 0, 1, 0, 0, 0, 1],
                \projection, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
            x.glide(\color, [0.9, 0.2, 0.5], 2, \smooth);
            x.get(\seed, { |value| [\exactUnsignedValue, value].postln });
        } { "Typed shader did not become ready; inspect its lastError.".warn };
    }).play(SystemClock);
};
)

// Subsequent live changes:
// x.glide(\offset, [0.25, 0], 1, \linear);
// x.set(\enabled, false);
// x.free; d.free; w.free; v.free;
