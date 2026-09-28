(
var sourcePath = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/shadertoy_gradient.frag";

v = ShaderServer.new(\exampleShadertoy);
w = ShaderWindow.new(960, 540, "SCShader Shadertoy compatibility", v);
v.waitForBoot { |server|
    t = ShaderToy(\scshaderToyGradient, sourcePath);
    x = Shader(t, [\amount, 0.5], server);
};
)

// Run after the source is visible. The renderer preserves its prior image if a
// later Naga GLSL compile fails.
// x.set(\amount, 0.8);
// x.free; t.free; w.free; v.free;
