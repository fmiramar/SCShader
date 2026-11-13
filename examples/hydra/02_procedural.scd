// Uses its own server/controller names and port so 01_osc can keep running.
(
~hydraFamiliesServer = ShaderServer.new(\hydraFamilies, "127.0.0.1", 57185);
~hydraFamiliesServer.waitForBoot({ |server|
    ~hydraFamilies = SCHydra.new(server);
    ~hydraFamilies.waitForReady({ |h|
        h.noise(4, 0.15).kaleid(5)
            .color(0.25, 0.25, 0.25).brightness(0.5)
            .posterize(5, 0.7).color(0.3, 0.65, 1)
            .out(\o0, { |result|
                if(result[\ok]) { h.render(\o0) } { result.postln };
            });
    }, { |h, error| error.postln; h.free });
}, { |server, error| error.postln });
)

// Evaluate each replacement separately after startup/previous commit completes.
(
~hydraFamilies.shape(5, 0.35, 0.015)
    .repeat(3, 2, 0.3, -0.2).rotate(0.2, 0.05)
    .color(0.7, 0.3, 1).invert(0.2)
    .out(\o0, { |result| result.postln });
)

(
~hydraFamilies.voronoi(6, 0.3, 0.3).scrollX(0.1, 0.03)
    .color(0.25, 0.7, 1).colorama(0.03)
    .out(\o0, { |result| result.postln });
)

// Cleanup (evaluate later):
// ~hydraFamilies.free; ~hydraFamiliesServer.free;
