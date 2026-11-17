// Step 18: recompile classes and reopen this installed file.
// Run startup once, then the finite Pattern performance after PATTERNS READY.
// BEGIN STARTUP
(
if(~hydraPatternServer.notNil){Error("Run cleanup before restarting Patterns").throw};
~hydraPatternReady=false;~hydraPatternBusy=false;
~hydraPatternServer=ShaderServer.new(\hydraPatterns,"127.0.0.1",57207);
~hydraPatternServer.rendererArgs_(["--shader",ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/hydra_black.wgsl"]);
~hydraPatternServer.waitForBoot({|server|
 ~hydraPattern=SCHydra.new(server);~hydraPattern.waitForReady({|h|
  h.osc(h.param(\frequency,8),0.1,0.8).color(h.param(\red,0.6),0.5,1).out(\o0,{|r|
   if(r[\ok]){h.render(\o0,{|shown|if(shown[\ok]){~hydraPatternReady=true;"PATTERNS READY".postln}{shown.postln}})}{r.postln};
  });
 },{|h,error|error.postln});
});
)
// END STARTUP

// BEGIN PERFORMANCE
(
if(~hydraPatternReady!=true or:{~hydraPatternBusy}) {Error("Wait for PATTERNS READY and the prior performance to finish").throw};
~hydraPatternBusy=true;~hydraPatternApplied=0;~hydraPatternSent=0;
~hydraPatternAmplitude=0.03;
~hydraPatternAudio=~hydraPatternAudio ? Server.default;
~hydraPatternAudio.waitForBoot({
 Routine({
  SynthDef(\scshaderHydraPattern18,{|freq=440,amp=0.03|
   Out.ar(0,(SinOsc.ar(freq)*EnvGen.kr(Env.perc(0.005,0.08),doneAction:2)*amp)!2);
  }).add;
  ~hydraPatternAudio.sync;
  ~hydraPatternClock.tryPerform(\stop);~hydraPatternClock=TempoClock.new(2);
  ~hydraPatternDelay={ (thisThread.seconds+0.3).max(Main.elapsedTime+0.1)-thisThread.seconds };
  ~hydraPatternSend={|name,value|
   ~hydraPatternSent=~hydraPatternSent+1;
   ~hydraPattern.setAt(name,value,~hydraPatternDelay.value,\o0,{|server,timing|
    ~hydraPatternApplied=~hydraPatternApplied+1;
    if(~hydraPatternApplied==16){~hydraPatternBusy=false;"PATTERNS: 16 scalar updates applied; no graph uploads per note".postln};
   });0
  };
  ~hydraPatternClock.schedAbs(~hydraPatternClock.beats.ceil+1,{
   // These are control Patterns: Pfunc sends values, while rest events make no Synth.
   ~hydraPatternFrequency=Pbind(\type,\rest,\dur,1,\value,Pseq([5,8,12,16],2),
    \send,Pfunc{|event|~hydraPatternSend.value(\frequency,event[\value])}).play(~hydraPatternClock,quant:0);
   ~hydraPatternColor=Pbind(\type,\rest,\dur,1,\value,Pseq([0.2,0.9,0.4,1],2),
    \send,Pfunc{|event|~hydraPatternSend.value(\red,event[\value])}).play(~hydraPatternClock,quant:0);
   ~hydraPatternNotes=Pbind(\instrument,\scshaderHydraPattern18,\server,~hydraPatternAudio,
    \freq,Pseq([220,330,440,550],2),\dur,1,\amp,~hydraPatternAmplitude,
    \latency,Pfunc{~hydraPatternDelay.value},\hasGate,false).play(~hydraPatternClock,quant:0);
   nil
  });
 }).play(SystemClock);
});
)
// END PERFORMANCE

// Cleanup: stop all three players before disposing the Hydra session.
(
~hydraPatternFrequency.tryPerform(\stop);~hydraPatternColor.tryPerform(\stop);~hydraPatternNotes.tryPerform(\stop);
~hydraPatternClock.tryPerform(\stop);~hydraPattern.tryPerform(\free);
~hydraPatternServer.quit({~hydraPatternServer=nil;~hydraPatternReady=false;"PATTERNS STOPPED".postln});
)
