// Recompile the SC class library, then evaluate startup once.
// BEGIN STARTUP
(
~hydraParamReady=false;
~hydraParamServer=ShaderServer.new(\hydraParameters,"127.0.0.1",57198);
~hydraParamServer.waitForBoot({|server|
    ~hydraParam=SCHydra.new(server);
    ~hydraParam.waitForReady({|h|
        h.osc(h.param(\frequency,8),0.1,0.8)
            .color(h.wave(0.7,0,0.2,1),0.7,1)
            .rotate(h.time(0.1))
            .scale(h.seq([1,1.3,1.05,1.6]).fast(0.5).smooth(1))
            .out(\o0,{|result|if(result[\ok]){h.render(\o0,{|shown|~hydraParamReady=shown[\ok];shown.postln})}{result.postln}});
    },{|h,error|error.postln});
});
)

// END STARTUP

// After startup: this changes one numeric uniform without resending the graph.
(
~hydraParam.set(\frequency,16,\o0,{|result|result.postln});
)

// Native wave/time/sequence sources run in the renderer without SC frame callbacks.
// Sequences currently use the initial 30 BPM; Step 16 adds clock controls.
// Successful graph replacement resets output history; numeric updates retain it.

// Cleanup when finished:
(
~hydraParam.free;
~hydraParamServer.quit;
)
