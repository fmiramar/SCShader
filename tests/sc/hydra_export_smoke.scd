// Explicit selection: opt-in FFmpeg recording and export files, no audio server.
(
var root=ShaderDef.filenameSymbol.asString.dirname.dirname,text=File.readAllString(root +/+ "examples/hydra/11_export.scd"),a,b,done=false,finish,call,check,poll;
finish={|ok,msg|var server=~hydraExportServer,exit={if(ok){"SCSHADER_HYDRA_EXPORT_OK".postln;0.exit}{1.exit}};
 if(done.not){done=true;msg.postln;~hydraExport.tryPerform(\free);if(server.notNil){server.quit(exit)}{exit.value};SystemClock.sched(2,{exit.value;nil})};
};
call={|operation|~hydraExportCall.value(operation)};
check={|condition,message|if(condition.not){Error(message).throw}};
poll={~hydraExportWait.value};
~hydraExportFolder="SCSHADER_EXPORT_TEST_DIR".getenv;
if(~hydraExportFolder.isNil){Error("Set SCSHADER_EXPORT_TEST_DIR to a fresh absolute directory").throw};
a=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;b=text.find("// END STARTUP");
text.copyRange(a,b-1).replace("57218","57219")
 .replace("~hydraExportServer.waitForBoot","~hydraExportServer.rendererPath_(\"SCSHADER_RENDERER\".getenv);~hydraExportServer.waitForBoot").interpret;
Routine({while{~hydraExportReady!=true and:{done.not}}{0.1.wait};if(done.not){try{
 var h=~hydraExport,status,bad,request,baseline;
 request={|operation|var condition=Condition.new,result;operation.value({|r|result=r;condition.unhang});condition.hang;result};
 baseline=call.value({|r|h.exportStatus(r)});check.value(baseline[\buffers]==0,"readback allocated without opt-in");
 call.value({|r|h.setResolution(319,181,r)});
 [[1,0,0],[0,1,0],[0,0,1],[1,1,0]].do{|color,index|call.value({|r|h.solid(*color).out([\o0,\o1,\o2,\o3][index],r)})};
 call.value({|r|h.screenshot(~hydraExportFolder +/+ "quadrants.png",r)});status=poll.value;
 check.value(status[\width]==319 and:{status[\height]==181 and:{status[\written]==1 and:{status[\buffers]==0}}},"screenshot size/count/disposal failed");
 bad=request.value({|r|h.screenshot(~hydraExportFolder +/+ "quadrants.png",r)});check.value(bad[\ok].not,"existing PNG overwritten");
 bad=request.value({|r|h.startRecording(~hydraExportFolder +/+ "odd.mp4","SCSHADER_FFMPEG".getenv,30,r)});check.value(bad[\ok].not,"odd MP4 dimensions accepted");
 call.value({|r|h.setResolution(320,180,r)});~hydraExportWindow.size=800@450;
 call.value({|r|h.render(\o2,r)});call.value({|r|h.screenshot(~hydraExportFolder +/+ "single.png",r)});status=poll.value;
 check.value(status[\width]==320 and:{status[\height]==180},"window DPI/resize changed explicit resolution");
 call.value({|r|h.render(nil,r)});
 call.value({|r|h.startRecording(~hydraExportFolder +/+ "short.mp4","SCSHADER_FFMPEG".getenv,30,r)});
 bad=request.value({|r|h.setResolution(640,360,r)});check.value(bad[\ok].not,"resolution changed during recording");
 bad=request.value({|r|h.screenshot(~hydraExportFolder +/+ "busy.png",r)});check.value(bad[\ok].not,"concurrent screenshot allocated another readback");
 1.2.wait;status=call.value({|r|h.exportStatus(r)});check.value(status[\buffers]==1,"recording staging allocation count changed");
 call.value({|r|h.stopRecording(r)});status=poll.value;
 check.value(status[\written]>5 and:{status[\buffers]==0},"recording frames/disposal failed");
 ("EXPORT_RECORDED_FRAMES "++status[\written]).postln;
 if("SCSHADER_EXPORT_STALLED_ENCODER".getenv.notNil){
  var deadline;
  call.value({|r|h.startRecording(~hydraExportFolder +/+ "stalled.mp4","SCSHADER_EXPORT_STALLED_ENCODER".getenv,30,r)});
  0.5.wait;status=call.value({|r|h.exportStatus(r)});
  check.value(status[\buffers]==1 and:{status[\dropped]>0},"blocked encoder did not retain one staging buffer/drop frames");
  call.value({|r|h.stopRecording(r)});deadline=Main.elapsedTime+8;
  while{status[\state]!="error" and:{Main.elapsedTime<deadline}}{
   status=call.value({|r|h.exportStatus(r)});0.1.wait;
  };
  check.value(status[\state]=="error" and:{status[\buffers]==0},"blocked encoder was not killed/reaped and released");
  "EXPORT STALLED ENCODER: bounded drops and timed shutdown pass".postln;
  call.value({|r|h.screenshot(~hydraExportFolder +/+ "after-error.png",r)});poll.value;
 };
 call.value({|r|h.setResolution(0,0,r)});
 ~hydraExportStop.value;~hydraExportStop.value;while{~hydraExportClosing==true}{0.05.wait};~hydraExportStop.value;
 finish.value(true,"PNG/MP4, quadrants/single, physical resolution, overwrite/odd/busy guards and cleanup pass");
}{|error|error.reportError;finish.value(false,error.errorString)}}}).play(SystemClock);
SystemClock.sched(30,{finish.value(false,"Export timeout");nil});
)
