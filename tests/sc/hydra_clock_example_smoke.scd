(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/07_clock.scd";
var text=File.readAllString(file),extract,done=false,finish,performance;
extract={|begin,end|text.copyRange(text.find(begin)+begin.size,text.find(end)-1)};
finish={|ok,message|if(done.not){done=true;message.postln;~hydraTimingRoutine.tryPerform(\stop);~hydraTimingClock.tryPerform(\stop);~hydraTiming.tryPerform(\free);~hydraTimingAudio.tryPerform(\quit);~hydraTimingServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_CLOCK_EXAMPLE_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
~hydraTimingOwnedAudio=Server(\hydraTimingOwnedAudio,NetAddr("127.0.0.1",57202),ServerOptions.new.numInputBusChannels_(0));
performance=extract.value("// BEGIN PERFORMANCE","// END PERFORMANCE")
 .replace("~hydraTimingAmplitude=0.03","~hydraTimingAmplitude=0")
 .replace("Server.default","~hydraTimingOwnedAudio")
 .replace("delay=target-logical;","delay=target-logical; if(i==2 and:{delay<0.5}) {Error(\"Stall deadline was not rebased\").throw};")
 .replace("target=(logical+0.3)","if(i==2) {var until=Main.elapsedTime+0.45;while {Main.elapsedTime<until} {}}; target=(logical+0.3)");
extract.value("// BEGIN VISUAL","// END VISUAL").replace("57200","57203").interpret;
Routine({
 while{~hydraTimingReady!=true and:{done.not}}{0.1.wait};
 if(done.not){
  2.do {|run|
   if(done.not){
    performance.interpret;
    if(try {performance.interpret;false} {true}.not) {finish.value(false,"Overlapping run was accepted")};
    while{~hydraTimingFinished!=true and:{done.not}}{0.1.wait};
    if(done.not){
     if(~hydraTimingApplied!=8 or:{~hydraTimingBusy or:{~hydraTimingServer.lastError.notNil}}){finish.value(false,"Incomplete/late/error performance run")} {
      ("Run %: eight applied beats, including a 450ms language stall; no renderer errors".format(run+1)).postln;
      0.2.wait;
     };
    };
   };
  };
  if(done.not){~hydraTiming.clock(90,0.25);0.3.wait;~hydraTiming.reset(0,120,1);0.3.wait;finish.value(~hydraTimingServer.lastError.isNil,"Two actual example performances and independent visual clock controls pass; audio muted")};
 };
}).play(SystemClock);
SystemClock.sched(35,{finish.value(false,"Clock example timeout");nil});
)
