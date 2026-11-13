// Recompile the SC class library after installation. Run Step 1 once.
// Step 0 at the end stops only this example's visual server and musical clock.
// BEGIN VISUAL
(
~hydraTimingReady=false;
~hydraTimingServer=ShaderServer.new(\hydraTiming,"127.0.0.1",57200);
~hydraTimingServer.waitForBoot({|server|
    ~hydraTiming=SCHydra.new(server);
    ~hydraTiming.waitForReady({|h|
        var pulse=h.param(\pulse,0.15);
        h.osc(8).color(pulse,pulse,pulse).rotate(h.beat(0.1)).out(\o0,{|result|
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
// Audio and visuals receive the same future logical timestamp. Display refresh
// and audio hardware latency still apply; this is not sample-accurate video.
// BEGIN PERFORMANCE
(
if(~hydraTimingReady!=true) {"Run Step 1 and wait for VISUAL READY.".warn} {
    ~hydraTimingAmplitude=0.03;
    ~hydraTimingAudio=Server.default;
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
                    var value=pulses.next, messages=List.new;
                    if(i==0) {messages.add(~hydraTiming.clockMessage(120,1,0))};
                    messages.add(~hydraTiming.parameterMessage(\pulse,value,\o0));
                    ~hydraTimingAudio.makeBundle(0.1,{
                        Synth(\scshaderHydraClock16Tick,[\freq,if(i.even){880}{660},\amp,~hydraTimingAmplitude],~hydraTimingAudio);
                    });
                    ~hydraTimingServer.scheduleMarker(0.1,{|server,timing|
                        ~hydraTimingApplied=~hydraTimingApplied+1;
                        ((timing[\appliedRendererTime]-timing[\scheduledRendererTime])*1000).postln;
                        if(~hydraTimingApplied==8) {~hydraTimingFinished=true;"CLOCK EIGHT BEATS APPLIED (error above is ms)".postln};
                    },messages.asArray);
                    1.wait;
                };
            }).play(~hydraTimingClock);
        }).play(SystemClock);
    });
};
)
// END PERFORMANCE

// Step 3, after the eight-beat block: slow visual time continuously and change BPM.
// The audio block has finished. Re-run Step 2 to realign both at 120 BPM.
(
~hydraTiming.clock(90,0.5);
)

// Step 4: reset seconds to zero and restore 120 BPM / speed one.
// This jumps time/beat phase and retains image history and numeric handles.
(
~hydraTiming.reset(0,120,1);
)

// Step 0: cleanup. The user's audio server stays available; tick Synths free themselves.
(
~hydraTimingRoutine.tryPerform(\stop);
~hydraTimingClock.tryPerform(\stop);
~hydraTiming.tryPerform(\free);
~hydraTimingServer.tryPerform(\quit);
)
