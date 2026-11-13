// Step 17. Recompile classes first. Run startup once, then cleanup below.
// A looping example recording drives amplitude -> scale and onset -> red intensity.
// BEGIN STARTUP
(
if(~hydraAudioVisualServer.notNil){Error("Run cleanup before restarting audio visuals").throw};
~hydraAudioReady=false;
~hydraAudioServer=Server.default;
~hydraAudioMonitor=0.2; // Quiet monitoring; analysis is on the private bus before this gain.
~hydraAudioVisualServer=ShaderServer.new(\hydraAudio,"127.0.0.1",57204);
~hydraAudioVisualServer.rendererArgs_(["--shader",ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders/hydra_black.wgsl"]);
~hydraAudioServer.waitForBoot({
 Routine({
  ~hydraAudioBuffer=Buffer.readChannel(~hydraAudioServer,ExampleFiles.child("a11wlk01.wav"),channels:[0]);
  ~hydraAudioServer.sync;
  ~hydraAudioBus=Bus.audio(~hydraAudioServer,1);
  SynthDef(\scshaderHydraAudio17,{|out=0,buf=0,monitor=0.2|
   var input=PlayBuf.ar(1,buf,BufRateScale.kr(buf),loop:1);
   // Alternative microphone source (replace PlayBuf): var input=SoundIn.ar(0);
   var signal=input*LFPulse.kr(2,0,0.5).lag(0.003)*Line.kr(0,0.3,0.2);
   Out.ar(out,signal);Out.ar(0,(signal*monitor)!2);
  }).send(~hydraAudioServer);
  ~hydraAudioServer.sync;
  ~hydraAudioSource=Synth.head(~hydraAudioServer.defaultGroup,\scshaderHydraAudio17,[\out,~hydraAudioBus,\buf,~hydraAudioBuffer,\monitor,~hydraAudioMonitor]);
  ~hydraAudioAnalysis=ShaderAnalysis(~hydraAudioServer,~hydraAudioBus.index,[\amplitude,\onset],30,0.3);
  ~hydraAudioAnalysis.start;
  ~hydraAudioVisualServer.waitForBoot({|server|
   ~hydraAudio=SCHydra.new(server);~hydraAudio.waitForReady({|h|
    h.osc(10,0.1,0.8).color(h.param(\red,0.2),0.4,1).scale(h.param(\size,0.8)).out(\o0,{|r|
     if(r[\ok]){h.render(\o0,{|shown|if(shown[\ok]){
      h.mapBus(~hydraAudioAnalysis[\amplitude],\size,scale:4,bias:0.8);
      h.mapBus(~hydraAudioAnalysis[\onset],\red,scale:0.8,bias:0.2);
      ~hydraAudioReady=true;"AUDIO VISUALS READY: scale follows amplitude; red reacts to onsets".postln;
     }{shown.postln}})}{r.postln};
    });
   },{|h,error|error.postln});
  });
 }).play(SystemClock);
});
)
// END STARTUP

// Cleanup: the user's audio server stays running; free only this example's resources.
(
~hydraAudio.tryPerform(\free);
~hydraAudioAnalysis.tryPerform(\free);
~hydraAudioAnalysis.tryPerform(\buses).tryPerform(\values).do(_.free);
~hydraAudioSource.tryPerform(\free);
~hydraAudioBuffer.tryPerform(\free);~hydraAudioBus.tryPerform(\free);
~hydraAudioVisualServer.quit({~hydraAudioVisualServer=nil;~hydraAudioReady=false;"AUDIO VISUALS STOPPED".postln});
)
