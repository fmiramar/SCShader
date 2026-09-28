// Manual desktop check. Run the first block in the IDE, then use its renderer window.
// Mouse movement changes brightness. Click to print normalized coordinates.
// F: fullscreen toggle. R: reload shader. V: statistics. Escape: leave fullscreen.
// R recompiles unchanged source, so it should look the same. Check acknowledgement below.
// Also drag-resize, minimize/restore, and switch focus with Alt-Tab.
// No audio server is needed. Finish with the cleanup block below.

(
var source = ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/fullscreen.wgsl";
if(~scShaderInteractionServer.notNil) { Error("Run this example's cleanup block before restarting it.").throw };
~scShaderInteractionServer = ShaderServer.new(\interactionCheck, "127.0.0.1", 57245);
~scShaderInteractionWindow = ShaderWindow(800, 450, "SCShader interaction: F / R / V / Escape", ~scShaderInteractionServer);
~scShaderInteractionWindow.inputEnabled_(true);
~scShaderInteractionWindow.mouseMoveAction_({ |window, point|
    if(~scShaderInteractionShader.notNil and: { ~scShaderInteractionShader.isRunning }) {
        ~scShaderInteractionShader.set(\amount, point.x);
    };
});
~scShaderInteractionWindow.mouseDownAction_({ |window, point, button| [\mouse, point, button].postln });
~scShaderInteractionWindow.resizeAction_({ |window, logical, pixels, ratio| [\resize, logical, pixels, ratio].postln });
~scShaderInteractionWindow.focusAction_({ |window, focused| [\focus, focused].postln });
~scShaderInteractionWindow.keyDownAction_({ |window, key, repeat|
    [\key, key, \autoRepeat, repeat].postln;
    if(repeat.not) {
        switch(key.asString.toLower,
            "f", { window.fullscreen_(window.fullscreen.not) },
            "r", {
                ~scShaderInteractionShader.tryPerform(\reload);
                "Reload requested: unchanged source should look the same.".postln;
            },
            "v", { window.server.showStats_(window.server.showStats.not) },
            "escape", { window.fullscreen_(false) }
        );
    };
});
~scShaderInteractionServer.waitForBoot({ |server|
    ~scShaderInteractionDef = ShaderDef(\interactionPulse, source);
    ~scShaderInteractionShader = Shader(~scShaderInteractionDef, [\amount, 0.6], server);
    [\ready, server.version, server.backend, server.device].postln;
}, { |server, error| error.postln });
)

// After pressing R, reloads should be above zero and error should be nil.
(
if(~scShaderInteractionShader.notNil) {
    [\reloads, ~scShaderInteractionShader.reloadCount,
        \error, ~scShaderInteractionShader.lastError].postln;
} {
    "The interaction example has already been cleaned up.".postln;
};
)

// Cleanup only this example's resources. Run even if the renderer was closed manually.
(
~scShaderInteractionWindow.tryPerform(\free);
~scShaderInteractionServer.tryPerform(\free);
~scShaderInteractionDef.tryPerform(\free);
~scShaderInteractionShader = nil;
~scShaderInteractionDef = nil;
~scShaderInteractionWindow = nil;
~scShaderInteractionServer = nil;
)
