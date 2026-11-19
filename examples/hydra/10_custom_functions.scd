// Step 22: typed, session-local WGSL functions. No audio server is needed.
// Recompile classes and reopen this file. Run blocks in order; wait for each result.
// WGSL function bodies are supported; pasted GLSL/JavaScript functions are not.
// BEGIN STARTUP
(
if(~hydraCustomServer.notNil or:{~hydraCustomClosing==true}){Error("Run cleanup and wait for CUSTOM STOPPED first").throw};
~hydraCustomReady=false;
~hydraCustomBody="let pulse=0.55+0.45*sin(time); return vec4<f32>(mix(color.rgb,color.rgb*tint.rgb,amount*pulse),color.a);";
~hydraCustomInputs=[[\amount,\float,0.65],[\tint,\vec4,[0.2,1.0,0.7,1.0]]];
~hydraCustomServer=ShaderServer.new(\hydraCustom,"127.0.0.1",57216);
~hydraCustomAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders";
if(PathName(~hydraCustomAssets).isFolder.not){~hydraCustomAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "renderer/shaders"};
~hydraCustomServer.rendererArgs_(["--shader",~hydraCustomAssets +/+ "hydra_black.wgsl"]);
~hydraCustomServer.waitForBoot({|server|
 ~hydraCustom=SCHydra.new(server);~hydraCustom.waitForReady({|h|
  h.setFunction(\tintPulse,\color,~hydraCustomInputs,~hydraCustomBody,{|registered|
   if(registered[\ok]){
    ~hydraCustomExpression=h.osc(8,0.1,0.4)
     .blend(h.shape(4,0.45,0.02).rotate(0.4).call(\tintPulse,[0.9,[1,0.3,0.6,1]]),0.5)
     .call(\tintPulse,[h.param(\strength,0.65)]);
    ~hydraCustomExpression.out(\o0,{|committed|
     if(committed[\ok]){h.render(\o0,{|shown|~hydraCustomReady=shown[\ok];if(shown[\ok]){"CUSTOM READY: moving oscillator and nested shape, with pulsing custom tint".postln}{shown.postln}})}{committed.postln};
    });
   }{registered.postln};
  });
 },{|h,error|error.postln});
});
~hydraCustomStop={
 var h=~hydraCustom,server=~hydraCustomServer;
 if(server.notNil){
  ~hydraCustomClosing=true;~hydraCustom=nil;~hydraCustomServer=nil;~hydraCustomReady=false;
  h.tryPerform(\free);server.quit({~hydraCustomClosing=false;"CUSTOM STOPPED".postln});
 }{(~hydraCustomClosing==true).if({"CUSTOM STOPPING"},{"CUSTOM ALREADY STOPPED"}).postln};
};
)
// END STARTUP

// 2. Change the custom float argument without compiling another shader.
(
~hydraCustom.set(\strength,0.15,\o0,{|result|result.postln});
)

// 3. Deliberately reject a bad replacement. Expect E_HYDRA_FUNCTION and continuing visuals.
// The accepted registration, output program and parameter mapping stay usable.
(
~hydraCustom.setFunction(\tintPulse,\color,~hydraCustomInputs,"return missing_identifier;",{|result|
 if(result[\ok]){"UNEXPECTED: bad code was accepted".postln}{("EXPECTED REJECTION: "++result[\code]).postln};
});
)

// 4. Remove the registration. The running program keeps its frozen accepted function.
(
~hydraCustom.removeFunction(\tintPulse,{|result|result.postln});
)

// 5. New calls after removal fail locally. This catches the expected error.
(
try{~hydraCustom.osc(8).call(\tintPulse);"UNEXPECTED: removed function was found".postln}{|error|"EXPECTED: removed function cannot be used by a new composition".postln};
)

// 6. Re-register and build a NEW composition. Old expression handles are stale after
// removal/redefinition; accepted outputs remain valid until explicitly replaced.
(
~hydraCustom.setFunction(\tintPulse,\color,~hydraCustomInputs,~hydraCustomBody,{|registered|
 if(registered[\ok]){~hydraCustom.osc(5,0.12,0.8)
  .blend(~hydraCustom.shape(3,0.4,0.02).rotate(0.2).call(\tintPulse),0.4)
  .call(\tintPulse,[0.9,[1,0.6,0.2,1]]).out(\o0,{|result|result.postln})
 }{registered.postln};
});
)

// 7. Cleanup is safe to run repeatedly. Re-run startup only after CUSTOM STOPPED.
(
~hydraCustomStop.value;
)
