(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/07_clock.scd";
var text=File.readAllString(file),extract,done=false,finish;
extract={|begin,end|text.copyRange(text.find(begin)+begin.size,text.find(end)-1)};
finish={|ok,message|if(done.not){done=true;message.postln;~hydraTimingRoutine.tryPerform(\stop);~hydraTimingClock.tryPerform(\stop);~hydraTiming.tryPerform(\free);~hydraTimingAudio.tryPerform(\quit);~hydraTimingServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_CLOCK_EXAMPLE_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
extract.value("// BEGIN VISUAL","// END VISUAL").replace("57200","57203").interpret;
Routine({
 while{~hydraTimingReady!=true and:{done.not}}{0.1.wait};
 if(done.not){
  extract.value("// BEGIN PERFORMANCE","// END PERFORMANCE")
   .replace("~hydraTimingAmplitude=0.03","~hydraTimingAmplitude=0")
   .replace("Server.default","Server(\\hydraTimingOwnedAudio,NetAddr(\"127.0.0.1\",57202),ServerOptions.new.numInputBusChannels_(0))").interpret;
  while{~hydraTimingFinished!=true and:{done.not}}{0.1.wait};
  if(done.not){finish.value(~hydraTimingApplied==8,"Actual example: eight audio bundles and timed Hydra controls applied; test audio muted")};
 };
}).play(SystemClock);
SystemClock.sched(30,{finish.value(false,"Clock example timeout");nil});
)
