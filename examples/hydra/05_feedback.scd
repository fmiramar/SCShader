// Step 13: recompile the SC class library after installing this build.
// Evaluate each numbered block separately and wait for ACKNOWLEDGED.
// Successful replacements start that output with black history.

// Step 0: stop an earlier copy before restarting. Wait for STOPPED.
(
~hydraFeedback.tryPerform(\free);
if(~hydraFeedbackServer.notNil) {
    ~hydraFeedbackServer.quit({
        ~hydraFeedback = nil; ~hydraFeedbackServer = nil;
        "FEEDBACK STOPPED".postln;
    });
} { "FEEDBACK STOPPED".postln };
)

// Step 1: startup. Expect animated stripes building up from black, with trails.
// BEGIN STARTUP
(
if(~hydraFeedbackServer.notNil) {
    "Run Step 0 and wait for STOPPED before restarting.".warn;
} {
    ~hydraFeedbackReady = false; ~hydraFeedbackBusy = false; ~hydraFeedbackNext = 2;
    ~hydraFeedbackServer = ShaderServer.new(\hydraFeedback, "127.0.0.1", 57194);
    ~hydraFeedbackServer.waitForBoot({ |server|
        ~hydraFeedback = SCHydra.new(server);
        ~hydraFeedback.waitForReady({ |h|
            ~hydraFeedbackCheck = { |step, action|
                var finish;
                if(~hydraFeedbackReady.not or: { ~hydraFeedbackBusy or: { step != ~hydraFeedbackNext } }) {
                    "Wait for startup/acknowledgement and run steps in order.".warn;
                    action.value((ok: false, code: "E_EXAMPLE_ORDER"));
                } {
                    ~hydraFeedbackBusy = true;
                    finish = { |result|
                        ~hydraFeedbackBusy = false;
                        if(result[\ok]) {
                            ~hydraFeedbackNext = step + 1;
                            ("FEEDBACK STEP % ACKNOWLEDGED".format(step)).postln;
                        } { result.postln };
                        action.value(result);
                    };
                    if(step == 2) {
                        // o1 and o2 form a temporal cycle; o3 uses native prev().
                        h.src(\o2).scale(1.01).blend(h.osc(7, 0.08, 0.8), 0.08).out(\o1, { |a|
                            if(a[\ok].not) { finish.value(a) } {
                                h.src(\o1).rotate(0.03).out(\o2, { |b|
                                    if(b[\ok].not) { finish.value(b) } {
                                        h.prev.scale(1.02).rotate(0.015)
                                            .blend(h.shape(5, 0.35, 0.02).rotate(0, 0.2), 0.08)
                                            .out(\o3, { |c|
                                                if(c[\ok]) { h.render(nil, finish) } { finish.value(c) };
                                            });
                                    };
                                });
                            };
                        });
                    } { h.hush(finish) };
                };
            };
            h.src(\o0).scale(1.01).blend(h.osc(5), 0.06).out(\o0, { |result|
                if(result[\ok]) {
                    h.render(\o0, { |shown|
                        if(shown[\ok]) {
                            ~hydraFeedbackReady = true;
                            "FEEDBACK STEP 1 ACKNOWLEDGED: watch the evolving stripes.".postln;
                        } { shown.postln };
                    });
                } { result.postln };
            });
        }, { |h, error| error.postln });
    }, { |server, error| error.postln });
};
)
// END STARTUP

// Step 2: four outputs. o0 top left; o1 bottom left; o2 top right; o3 bottom right.
// The two cross-feedback quadrants evolve with different delayed images.
// The bottom-right polygon leaves expanding/rotating trails using prev().
(
~hydraFeedbackCheck.value(2);
)

// Optional: resize the window while Step 2 runs. Histories restart from black
// and build up again; all four programs should keep running.

// Step 3: expect a full black window. Both history images are reset per output.
(
~hydraFeedbackCheck.value(3);
)

// Finish with Step 0. Restart at Step 1 to repeat the complete test.
