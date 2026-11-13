// Ordered visual checks for the Step 12 build. Run one numbered block at a time.
// Wait for its ACKNOWLEDGED message, then inspect the window before continuing.
// Acknowledgement proves command acceptance; it does not prove the visible result.
// If an earlier copy of this demo is running, run Step 0 and wait for STOPPED.

// Step 0 — stop the previous copy (also use this for final cleanup).
(
~hydraFourTestReady = false;
~hydraFour.tryPerform(\free);
if(~hydraFourServer.notNil) {
    ~hydraFourServer.quit {
        ~hydraFourServer = nil; ~hydraFour = nil;
        "STOPPED — startup can now be evaluated.".postln;
    };
} { "STOPPED — startup can now be evaluated.".postln };
)

// BEGIN STARTUP
(
if(~hydraFourServer.notNil) {
    "Run Step 0 and wait for STOPPED before starting another copy.".warn;
} {
    ~hydraFourTestReady = false;
    ~hydraFourTestBusy = false;
    ~hydraFourTestNext = 1;
    ~hydraFourServer = ShaderServer.new(\hydraFour, "127.0.0.1", 57192);
    ~hydraFourServer.waitForBoot({ |server|
        ~hydraFour = SCHydra.new(server);
        ~hydraFour.waitForReady({ |h|
            // Every multi-output edit is acknowledged serially.
            ~hydraFourAssign = { |builders, done|
                var assign;
                assign = { |index|
                    builders[index].value.out([\o0, \o1, \o2, \o3][index], { |result|
                        if(result[\ok]) {
                            if(index < 3) { assign.value(index + 1) } { done.value(result) };
                        } { done.value(result) };
                    });
                };
                assign.value(0);
            };
            ~hydraFourAnimated = {
                [
                    { h.osc(10, 0.1, 0.8) },
                    { h.shape(5, 0.4).rotate(0, 0.2).color(0.4, 0.7, 1) },
                    { h.noise(3, 0.3).color(0.3, 0.7, 1) },
                    { h.voronoi(5, 0.3, 0.3).color(1, 0.6, 0.3) }
                ]
            };
            ~hydraFourPutAndShow = { |expression, output, display, done|
                expression.out(output, { |result|
                    if(result[\ok]) { h.render(display, done) } { done.value(result) };
                });
            };
            ~hydraFourExpectations = [
                "Quadrants: TOP LEFT red (o0), BOTTOM LEFT green (o1), TOP RIGHT blue (o2), BOTTOM RIGHT yellow (o3).",
                "Full window BLUE: single-output o2 selection.",
                "Four animated visuals restored: oscillator top left, polygon bottom left, noise top right, voronoi bottom right.",
                "Only BOTTOM RIGHT changes to a scaled, fully inverted copy of the top-left oscillator. It should retain visible stripes.",
                "Only BOTTOM RIGHT becomes flat mid-gray: invert(0.5) deliberately removes all RGB contrast.",
                "HUSH: full-window o0, cleared to transparent black. Record the visible color; gray alone does not confirm clearing.",
                "Full window OPAQUE BLACK. If it remains gray, report a failed clear/presentation check.",
                "Full window OPAQUE WHITE. This confirms the window responds to a contrasting opaque replacement.",
                "Full window OPAQUE BLACK again. Compare this with the transparent hush result from Step 6."
            ];
            ~hydraFourTestSteps = [
                { |done|
                    ~hydraFourAssign.value([
                        { h.solid(1, 0, 0, 1) }, { h.solid(0, 1, 0, 1) },
                        { h.solid(0, 0, 1, 1) }, { h.solid(1, 1, 0, 1) }
                    ], { |result| if(result[\ok]) { h.render(nil, done) } { done.value(result) } });
                },
                { |done| h.render(\o2, done) },
                { |done|
                    ~hydraFourAssign.value(~hydraFourAnimated.value,
                        { |result| if(result[\ok]) { h.render(nil, done) } { done.value(result) } });
                },
                { |done| ~hydraFourPutAndShow.value(h.src(\o0).scale(1.3).invert(1), \o3, nil, done) },
                { |done| ~hydraFourPutAndShow.value(h.src(\o0).scale(1.3).invert(0.5), \o3, nil, done) },
                { |done| h.hush(done) },
                { |done| ~hydraFourPutAndShow.value(h.solid(0, 0, 0, 1), \o0, \o0, done) },
                { |done| ~hydraFourPutAndShow.value(h.solid(1, 1, 1, 1), \o0, \o0, done) },
                { |done| ~hydraFourPutAndShow.value(h.solid(0, 0, 0, 1), \o0, \o0, done) }
            ];
            ~hydraFourCheck = { |step, action|
                if(~hydraFourTestReady.not or: { ~hydraFourTestBusy or: { step != ~hydraFourTestNext } }) {
                    var result = (ok: false, message: "Wait for startup/previous acknowledgement and run the next numbered step.");
                    result.postln; action.value(result);
                } {
                    ~hydraFourTestBusy = true;
                    "STEP % — %".format(step, ~hydraFourExpectations[step - 1]).postln;
                    ~hydraFourTestSteps[step - 1].value({ |result|
                        ~hydraFourTestBusy = false;
                        if(result[\ok]) {
                            ~hydraFourTestNext = step + 1;
                            "STEP % ACKNOWLEDGED — inspect the window; visual PASS is your observation.".format(step).postln;
                        } { result.postln };
                        action.value(result);
                    });
                };
            };
            ~hydraFourAssign.value(~hydraFourAnimated.value, { |result|
                if(result[\ok]) {
                    h.render(nil, { |shown|
                        if(shown[\ok]) {
                            ~hydraFourTestReady = true;
                            "STARTUP ACKNOWLEDGED — run Step 1 next.".postln;
                        } { shown.postln };
                    });
                } { result.postln };
            });
        }, { |h, error| error.postln; h.free });
    }, { |server, error| error.postln });
};
)
// END STARTUP

// Step 1 — identify every quadrant using opaque primary colors.
(~hydraFourCheck.value(1);)

// Step 2 — verify single-output selection (full-window blue).
(~hydraFourCheck.value(2);)

// Step 3 — restore the original four animated visuals and quadrant mode.
(~hydraFourCheck.value(3);)

// Step 4 — sample o0 into o3 with visible inversion (bottom-right stripes).
(~hydraFourCheck.value(4);)

// Step 5 — reproduce the intentional invert(0.5) gray result in o3.
(~hydraFourCheck.value(5);)

// Step 6 — clear all four outputs and select transparent o0; record its appearance.
(~hydraFourCheck.value(6);)

// Step 7 — compare with opaque black (alpha 1).
(~hydraFourCheck.value(7);)

// Step 8 — opaque white control, confirming visible replacement.
(~hydraFourCheck.value(8);)

// Step 9 — opaque black again; compare against Step 6, then clean up with Step 0.
(~hydraFourCheck.value(9);)
