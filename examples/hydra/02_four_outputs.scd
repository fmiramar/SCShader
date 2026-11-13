// Recompile the SC class library after installing this development build.
(
~hydraFourServer = ShaderServer.new(\hydraFour, "127.0.0.1", 57192);
~hydraFourServer.waitForBoot({ |server|
    ~hydraFour = SCHydra.new(server);
    ~hydraFour.waitForReady({ |h|
        var expressions, assign;
        expressions = [
            { h.osc(10, 0.1, 0.8) },
            { h.shape(5, 0.4).rotate(0, 0.2).color(0.4, 0.7, 1) },
            { h.noise(3, 0.3).color(0.3, 0.7, 1) },
            { h.voronoi(5, 0.3, 0.3).color(1, 0.6, 0.3) }
        ];
        assign = { |index|
            expressions[index].value.out([\o0, \o1, \o2, \o3][index], { |result|
                if(result[\ok]) {
                    if(index < 3) { assign.value(index + 1) } { h.render };
                } { result.postln };
            });
        };
        assign.value(0);
    }, { |h, error| error.postln; h.free });
}, { |server, error| error.postln });
)

// Evaluate each later block separately after startup/previous request completes.
(~hydraFour.render(\o2);)
(~hydraFour.render;)

// Acyclic same-frame sampling: o3 reads o0. Self-feedback awaits Step 13.
(
~hydraFour.src(\o0).scale(1.3).invert(0.5)
    .out(\o3, { |result| result.postln });
)

// Clear all four outputs with acknowledged sequential replacements, then select o0.
// ~hydraFour.hush({ |result| result.postln });
// Cleanup later:
// ~hydraFour.free; ~hydraFourServer.free;
