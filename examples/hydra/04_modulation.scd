// Recompile the SC class library after installing this development build.
(
~hydraModServer = ShaderServer.new(\hydraModulation, "127.0.0.1", 57190);
~hydraModServer.waitForBoot({ |server|
    ~hydraMod = SCHydra.new(server);
    ~hydraMod.waitForReady({ |h|
        h.osc(12, 0.1, 0.8).modulate(h.noise(3, 0.2), 0.15)
            .modulateRotate(h.gradient(0.2).rotate(0.2), 0.8, 0.1)
            .out(\o0, { |result|
                if(result[\ok]) { h.render(\o0) } { result.postln };
            });
    }, { |h, error| error.postln; h.free });
}, { |server, error| error.postln });
)

// Evaluate this replacement after startup completes.
(
~hydraMod.shape(5, 0.4, 0.02).repeat(3, 2)
    .modulateKaleid(~hydraMod.osc(5).modulate(~hydraMod.noise(2), 0.1), 5)
    .modulateScale(~hydraMod.gradient(0.1), 0.5, 1)
    .color(0.3, 0.6, 1).out(\o0, { |result| result.postln });
)

// Cleanup later:
// ~hydraMod.free; ~hydraModServer.free;
