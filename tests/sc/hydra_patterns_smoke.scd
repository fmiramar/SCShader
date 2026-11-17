(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/08_patterns.scd",text=File.readAllString(file),extract,done=false,finish,shaderEvent=Event.eventTypes[\shader];
extract={|a,b|text.copyRange(text.find(a)+a.size,text.find(b)-1)};
finish={|ok,msg|if(done.not){done=true;msg.postln;~hydraPatternFrequency.tryPerform(\stop);~hydraPatternColor.tryPerform(\stop);~hydraPatternNotes.tryPerform(\stop);~hydraPatternClock.tryPerform(\stop);~hydraPattern.tryPerform(\free);~hydraPatternAudio.tryPerform(\quit);~hydraPatternServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_PATTERNS_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
~hydraPatternOwnedAudio=Server(\hydraPatternOwned,NetAddr("127.0.0.1",57209),ServerOptions.new.numInputBusChannels_(0));
extract.value("// BEGIN STARTUP","// END STARTUP").replace("57207","57208").interpret;
Routine({while{~hydraPatternReady!=true and:{done.not}}{0.1.wait};if(done.not){
 if(try{~hydraPattern.parameterMessage(\missing,1);false}{true}.not){finish.value(false,"Invalid parameter accepted")};
 extract.value("// BEGIN PERFORMANCE","// END PERFORMANCE").replace("Server.default","~hydraPatternOwnedAudio").replace("~hydraPatternAmplitude=0.03","~hydraPatternAmplitude=0").interpret;
 while{~hydraPatternBusy and:{done.not}}{0.1.wait};0.3.wait;
 if(done.not){finish.value(~hydraPatternApplied==16 and:{~hydraPatternSent==16 and:{~hydraPatternServer.lastError.isNil and:{Event.eventTypes[\shader]===shaderEvent}}},"Two concurrent control Pbinds plus audio Pbind; legacy shader Event unchanged")};
}}).play(SystemClock);
SystemClock.sched(25,{finish.value(false,"Hydra Pattern timeout");nil});
)
