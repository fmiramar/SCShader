// Install the complete SCShader extension first. Close the previous demo first.

(
var shaderPath = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/fullscreen.wgsl";
~scShaderServer = ShaderServer.new(\exampleLifecycle);
~scShaderServer.waitForBoot { |server|
    ~scShaderDef = ShaderDef(\scShaderPulse, shaderPath);
    ~scShader = Shader(~scShaderDef, [\amount, 0.45], server);
    AppClock.sched(0.5, {
        ~scShader.set(\amount, 0.8);
        nil
    });
};
)

// Evaluate after editing and saving the WGSL source, or request a reload now:
(
~scShader.reload;
)

// Restore the built-in fullscreen shader and close the renderer child:
(
~scShader.free;
~scShaderDef.free;
~scShaderServer.free;
)
