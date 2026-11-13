// Recompile the SC class library after installation. Run Step 1 once.
// Step 0 at the end stops only this example's visual server and musical clock.
// Do not press Cmd-. between steps: ShaderServer uses it to quit renderers.
// The finite musical Routine ending does not stop the native visual clock.
// BEGIN VISUAL
(
if(~hydraTimingServer.notNil) {Error("Run Step 0 and wait for CLOCK STOPPED before restarting Step 1.").throw};
~hydraTimingReady=false;~hydraTimingBusy=false;
~hydraTimingServer=ShaderServer.new(\hydraTiming,"127.0.0.1",57200);
~hydraTimingServer.rendererArgs_(["--shader",ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/hydra_black.wgsl"]);
~hydraTimingServer.waitForBoot({|server|
    ~hydraTiming=SCHydra.new(server);
    ~hydraTiming.waitForReady({|h|
        var pulse=h.param(\pulse,1);
        h.osc(8,0,0.8).color(pulse,pulse,pulse).rotate(h.beat(0.8)).scale(h.wave(1,0,0.8,1.2)).out(\o0,{|result|
            if(result[\ok]) {h.render(\o0,{|shown|
                if(shown[\ok]) {h.clock(120,1,0);~hydraTimingReady=true;"CLOCK VISUAL READY".postln} {shown.postln};
            })} {result.postln};
        });
    },{|h,error|error.postln});
});
)
// END VISUAL

// Step 2: after VISUAL READY, run this short eight-beat performance.
// Expect alternating brightness and matching high/low quiet ticks at 120 BPM.
// Audio and visuals receive the same captured future timestamp.
// Normally this is 0.3 s ahead of the beat. A delayed SC wake-up moves BOTH
// targets to at least 0.1 s after now; that beat shifts instead of being sent late. Display refresh
// and audio hardware latency still apply; this is not sample-accurate video.
// BEGIN PERFORMANCE
(
if(~hydraTimingBusy==true) {Error("Wait for CLOCK EIGHT BEATS APPLIED before running Step 2 again.").throw};
if(~hydraTimingReady!=true) {"Run Step 1 and wait for VISUAL READY.".warn} {
    ~hydraTimingBusy=true;
    ~hydraTimingRoutine.tryPerform(\stop);~hydraTimingClock.tryPerform(\stop);
    ~hydraTimingAmplitude=0.03;
    ~hydraTimingAudio=~hydraTimingAudio ? Server.default;
    ~hydraTimingFinished=false;~hydraTimingApplied=0;
    ~hydraTimingAudio.waitForBoot({
        Routine({
            SynthDef(\scshaderHydraClock16Tick,{|freq=880,amp=0.03|
                var signal=SinOsc.ar(freq)*EnvGen.kr(Env.perc(0.002,0.06),doneAction:2)*amp;
                Out.ar(0,signal!2);
            }).send(~hydraTimingAudio);
            ~hydraTimingAudio.sync;
            ~hydraTimingClock=TempoClock.new(2);
            ~hydraTimingRoutine=Routine({
                var pulses=Pseq([1,0.15],4).asStream;
                8.do {|i|
                    var value=pulses.next, messages=List.new, logical=thisThread.seconds, target, delay;
                    // Keep the musical grid when on time; recover to a real future
                    // deadline when SC wakes after its logical deadline. Both
                    // audio and video use this SAME captured deadline.
                    target=(logical+0.3).max(Main.elapsedTime+0.1);
                    delay=target-logical;
                    if(i==0) {messages.add(~hydraTiming.clockMessage(120,1,0))};
                    messages.add(~hydraTiming.parameterMessage(\pulse,value,\o0));
                    ~hydraTimingAudio.makeBundle(delay,{
                        Synth(\scshaderHydraClock16Tick,[\freq,if(i.even){880}{660},\amp,~hydraTimingAmplitude],~hydraTimingAudio);
                    });
                    ~hydraTimingServer.scheduleMarker(delay,{|server,timing|
                        ~hydraTimingApplied=~hydraTimingApplied+1;
                        ((timing[\appliedRendererTime]-timing[\scheduledRendererTime])*1000).postln;
                        if(~hydraTimingApplied==8) {
                            ~hydraTimingServer.sendMsg(*~hydraTiming.parameterMessage(\pulse,1,\o0));
                            ~hydraTimingBusy=false;~hydraTimingFinished=true;"CLOCK EIGHT BEATS APPLIED (error above is ms)".postln};
                    },messages.asArray);
                    1.wait;
                };
            }).play(~hydraTimingClock);
        }).play(SystemClock);
    });
};
)
// END PERFORMANCE

// Step 3: only the musical Routine ends; the native visual clock keeps running.
// After eight beats, slow the clearly visible rotation/scaling and change BPM.
// The audio block has finished. Re-run Step 2 to realign both at 120 BPM.
(
if(~hydraTimingBusy) {Error("Wait for the eight-beat block to finish.").throw};
~hydraTiming.clock(90,0.25);
"VISUAL CLOCK: 90 BPM, quarter speed; rotation/scaling continue slowly.".postln;
)

// Step 4: reset seconds to zero and restore 120 BPM / speed one.
// This jumps time/beat phase and retains image history and numeric handles.
(
if(~hydraTimingBusy) {Error("Wait for the eight-beat block to finish.").throw};
~hydraTiming.reset(0,120,1);
"VISUAL CLOCK RESET: rotation/scaling restart at full speed.".postln;
)

// Step 0: cleanup. The user's audio server stays available; tick Synths free themselves.
(
~hydraTimingRoutine.tryPerform(\stop);
~hydraTimingClock.tryPerform(\stop);
~hydraTiming.tryPerform(\free);
if(~hydraTimingServer.notNil) {
    ~hydraTimingServer.quit({~hydraTimingServer=nil;~hydraTimingReady=false;~hydraTimingBusy=false;"CLOCK STOPPED".postln});
} {"CLOCK STOPPED".postln};
)
