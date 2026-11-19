(
var root=ShaderDef.filenameSymbol.asString.dirname.dirname;
var text=File.readAllString(root +/+ "examples/hydra/10_custom_functions.scd"),a,b,done=false,finish,request,call,check;
finish={|ok,msg|
 var exit={if(ok){"SCSHADER_HYDRA_CUSTOM_OK".postln;0.exit}{1.exit}},server=~hydraCustomServer;
 if(done.not){done=true;msg.postln;~hydraCustom.tryPerform(\free);if(server.notNil){server.quit(exit)}{exit.value};SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})};
};
request={|operation|var condition=Condition.new,result;operation.value({|r|result=r;condition.unhang});condition.hang;result};
call={|operation|var result=request.value(operation);if(result[\ok].not){Error(result.asString).throw};result};
check={|condition,message|if(condition.not){Error(message).throw}};
a=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;b=text.find("// END STARTUP");
// An explicit test executable takes precedence over the user's installed package.
text.copyRange(a,b-1).replace("57216","57217")
 .replace("~hydraCustomServer.waitForBoot", "~hydraCustomServer.rendererPath_(\"SCSHADER_RENDERER\".getenv); ~hydraCustomServer.waitForBoot").interpret;
Routine({while{~hydraCustomReady!=true and:{done.not}}{0.1.wait};if(done.not){try{
 var bad,old=~hydraCustomExpression,other=SCHydra.new;
 check.value(try{other.call(\tintPulse);false}{true},"custom registration leaked into another session");other.free;
 check.value(~hydraCustom.respondsTo(\tintPulse).not,"custom registration added a global selector");
 call.value({|r|~hydraCustom.set(\strength,0.25,\o0,r)});
 bad=request.value({|r|~hydraCustom.setFunction(\tintPulse,\color,~hydraCustomInputs,"return missing_identifier;",r)});
 check.value(bad[\ok].not and:{bad[\code]=="E_HYDRA_FUNCTION"},"bad function replacement was accepted");
 call.value({|r|old.out(\o0,r)}); // failed replacement retained its accepted generation
 bad=request.value({|r|~hydraCustom.setFunction(\osc,\src,[],"return vec4<f32>(1.0);",r)});
 check.value(bad[\ok].not,"builtin override was accepted");
 call.value({|r|~hydraCustom.removeFunction(\tintPulse,r)});
 check.value(try{~hydraCustom.osc(8).call(\tintPulse);false}{true},"removed function remained callable");
 check.value(try{old.snapshot;false}{true},"removed generation remained available for new snapshots");
 ~hydraCustomServer.sendMsg('/scshader/v1/window/resize',480,270);0.1.wait;
 call.value({|r|~hydraCustom.setFunction(\tintPulse,\color,~hydraCustomInputs,~hydraCustomBody,r)});
 check.value(try{old.snapshot;false}{true},"re-register silently changed an old expression's meaning");
 call.value({|r|~hydraCustom.osc(5).blend(~hydraCustom.shape(3).call(\tintPulse),0.4).call(\tintPulse).out(\o0,r)});
 [
  [\customSource,\src,"return vec4<f32>(uv,0.5,1.0);"],
  [\customCoord,\coord,"return uv*0.8;"],
  [\customCombine,\combine,"return mix(color,other,0.25);"],
  [\customModulate,\combineCoord,"return uv+other.xy*0.05;"]
 ].do{|definition|call.value({|r|~hydraCustom.setFunction(definition[0],definition[1],[],definition[2],r)})};
 call.value({|r|~hydraCustom.call(\customSource).call(\customCoord)
  .call(\customCombine,[~hydraCustom.shape(3).call(\tintPulse)])
  .call(\customModulate,[~hydraCustom.osc(3)]).out(\o0,r)});
 ~hydraCustomStop.value;~hydraCustomStop.value;while{~hydraCustomClosing==true}{0.05.wait};~hydraCustomStop.value;
 finish.value(true,"Custom WGSL: nested example, defaults/vec4/numbers, invalid replacement, reserved builtin, removal/generation/isolation, resize and cleanup pass");
}{|error|finish.value(false,error.asString)}}}).play(SystemClock);
SystemClock.sched(25,{finish.value(false,"Custom function timeout");nil});
)
