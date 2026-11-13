// Run after installing this development version and recompiling the class library.
// Uses the renderer discovered by ShaderServer, including from an unsaved document.
(
v = ShaderServer.new(\hydraDemo);
v.waitForBoot({ |server|
    h = SCHydra.new(server);
    h.waitForReady({
        h.osc(10).rotate(0.2).out(\o0, { |result|
            if(result[\ok]) {
                h.render(\o0, { |display| if(display[\ok].not) { display.postln } });
            } { result.postln };
        });
    }, { |controller, error| error.postln; controller.free });
}, { |server, error| error.postln });
)

// A live replacement (evaluate after the first commit):
// h.gradient(0.2).scale(1.5).color(0.5, 1, 0.8).out(\o0, { |result| result.postln });

// Return to the existing Shader / ShaderGraph presentation:
// h.render(\legacy);

// Cleanup (evaluate later):
// h.free; v.quit;
