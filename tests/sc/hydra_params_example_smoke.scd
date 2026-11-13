(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/06_parameters.scd";
var text=File.readAllString(file),start,end,done=false,finish;
finish={|ok,message|if(done.not){done=true;message.postln;~hydraParam.tryPerform(\free);~hydraParamServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_PARAMS_EXAMPLE_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
start=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;end=text.find("// END STARTUP");
text.copyRange(start,end-1).replace("57198","57199").interpret;
Routine({while{~hydraParamReady!=true and:{done.not}}{0.1.wait};if(done.not){0.3.wait;~hydraParam.set(\frequency,16,\o0,{|r|finish.value(r[\ok],r)})}}).play(SystemClock);
SystemClock.sched(20,{finish.value(false,"Numeric example timeout");nil});
)
