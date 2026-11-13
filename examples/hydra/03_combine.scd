// Recompile the SC class library after installing this development build.
(
~hydraCombineServer = ShaderServer.new(\hydraCombine, "127.0.0.1", 57188);
~hydraCombineServer.waitForBoot({ |server|
    ~hydraCombine = SCHydra.new(server);
    ~hydraCombine.waitForReady({ |h|
        h.osc(10).blend(h.shape(4, 0.4).rotate(0.2), 0.5)
            .out(\o0, { |result|
                if(result[\ok]) { h.render(\o0) } { result.postln };
            });
    }, { |h, error| error.postln; h.free });
}, { |server, error| error.postln });
)

// Evaluate this replacement after startup completes.
(
~hydraCombine.osc(12, 0.1, 0.8)
    .layer(~hydraCombine.shape(5, 0.4).color(0.2, 0.6, 1, 0.35)
        .blend(~hydraCombine.gradient(0.1), 0.2).rotate(0.2, 0.1))
    .out(\o0, { |result| result.postln });
)

// Cleanup later:
// ~hydraCombine.free; ~hydraCombineServer.free;
