// Step 23. No audio server or screen-capture permission is needed.
// Recompile classes and reopen this file. Wait for EXPORT READY after Step 1.
// Steps 2..7 queue in order; wait for EXPORT STEP DONE messages to inspect files.
// 1. Start four outputs at 640x360 render pixels, independent of Retina window size.
// BEGIN STARTUP
(
if(~hydraExportServer.notNil or:{~hydraExportClosing==true}){Error("Run cleanup and wait for EXPORT STOPPED first").throw};
~hydraExportReady=false;~hydraExportQueue=List.new;~hydraExportRunner=nil;
~hydraExportCompleted=0;~hydraExportTaskError=nil;~hydraExportSerial=0;
~hydraExportFolder=~hydraExportFolder ? (Platform.userHomeDir +/+ "SCShader-Captures");
if(PathName(~hydraExportFolder).isFolder.not){~hydraExportFolder.mkdir};
~hydraExportName=Date.localtime.stamp ++ "-" ++ Main.elapsedTime.asString.replace(".","_");
~hydraExportFile={|suffix|
 ~hydraExportSerial=~hydraExportSerial+1;
 ~hydraExportFolder +/+ (~hydraExportName++"-"++~hydraExportSerial.asString++"-"++suffix)
};
~hydraExportRun={|label,body|
 if(~hydraExportReady!=true){Error("Wait for EXPORT READY before queuing a step").throw};
 ~hydraExportQueue.add([label,body]);("EXPORT STEP QUEUED: "++label).postln;
 if(~hydraExportRunner.isNil){~hydraExportRunner=Routine({
  var task;
  while{~hydraExportQueue.notEmpty and:{~hydraExportReady==true}}{
   task=~hydraExportQueue.removeAt(0);("EXPORT STEP START: "++task[0]).postln;
   try{task[1].value;~hydraExportCompleted=~hydraExportCompleted+1;("EXPORT STEP DONE: "++task[0]).postln}{|error|
    ~hydraExportTaskError=error.errorString;error.reportError;~hydraExportQueue.clear;
    "EXPORT QUEUE CLEARED: inspect the error, then retry or run cleanup".postln;
   };
  };
  ~hydraExportRunner=nil;
 }).play(SystemClock)};
};
~hydraExportCall={|operation|
 var result,condition=Condition.new;operation.value({|r|result=r;condition.unhang});condition.hang;
 if(result[\ok].not){Error(result.asString).throw};result
};
~hydraExportWait={
 var status,deadline=Main.elapsedTime+12;
 while{status.isNil or:{status[\state]!="finished" and:{status[\state]!="error"}}}{
  if(Main.elapsedTime>deadline){Error("Export timed out; inspect exportStatus and cleanup").throw};
  status=~hydraExportCall.value({|r|~hydraExport.exportStatus(r)});0.05.wait;
 };
 if(status[\state]=="error"){Error(status[\error]).throw};status.postln;status
};
~hydraExportServer=ShaderServer.new(\hydraExport,"127.0.0.1",57218);
~hydraExportWindow=ShaderWindow.new(640,360,"Hydra export",~hydraExportServer);
~hydraExportAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders";
if(PathName(~hydraExportAssets).isFolder.not){~hydraExportAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "renderer/shaders"};
~hydraExportServer.rendererArgs_(["--shader",~hydraExportAssets +/+ "hydra_black.wgsl"]);
~hydraExportServer.waitForBoot({|server|
 ~hydraExport=SCHydra.new(server);~hydraExport.waitForReady({|h|Routine({
  ~hydraExportCall.value({|r|h.setResolution(640,360,r)});
  ~hydraExportCall.value({|r|h.osc(8,0.1,0.4).color(1,0.3,0.6).out(\o0,r)});
  ~hydraExportCall.value({|r|h.shape(4,0.4,0.02).rotate(0,0.1).color(0.2,1,0.4).out(\o1,r)});
  ~hydraExportCall.value({|r|h.gradient(0.1).color(0.3,0.5,1).out(\o2,r)});
  ~hydraExportCall.value({|r|h.noise(4,0.2).color(1,0.8,0.2).out(\o3,r)});
  ~hydraExportCall.value({|r|h.render(nil,r)});
  ~hydraExportReady=true;("EXPORT READY; destination: "++~hydraExportFolder).postln;
 }).play(SystemClock)},{|h,error|error.postln});
});
~hydraExportStop={
 var h=~hydraExport,server=~hydraExportServer;
 ~hydraExportQueue.tryPerform(\clear);
 if(~hydraExportRunner!==thisThread){~hydraExportRunner.tryPerform(\stop);~hydraExportRunner=nil};
 if(server.notNil){~hydraExportClosing=true;~hydraExport=nil;~hydraExportServer=nil;~hydraExportReady=false;
  h.tryPerform(\free);~hydraExportWindow.tryPerform(\free);
  server.quit({~hydraExportClosing=false;"EXPORT STOPPED".postln});
 }{(~hydraExportClosing==true).if({"EXPORT STOPPING"},{"EXPORT ALREADY STOPPED"}).postln};
};
)
// END STARTUP

// 2. Queue a quadrant PNG. A new name is generated each time this block is run.
// BEGIN SCREENSHOT
(
~hydraExportRun.value("quadrant PNG",{
 ~hydraExportCall.value({|r|~hydraExport.screenshot(~hydraExportFile.value("quadrants.png"),r)});
 ~hydraExportWait.value;
});
)
// END SCREENSHOT

// 3. Queue approximately five seconds of MP4. It starts after Step 2 finishes.
// FFmpeg is optional and separately installed; no audio is recorded.
// Files continue rendering while the window is covered or minimized.
// Frames may drop; movie duration is writtenFrames / FPS.
// BEGIN RECORD
(
~hydraExportRun.value("quadrant MP4",{
 ~hydraExportEncoder=(["SCSHADER_FFMPEG".getenv,"/opt/homebrew/bin/ffmpeg","/usr/local/bin/ffmpeg"]
  .detect{|p|p.notNil and:{File.exists(p)}});
 if(~hydraExportEncoder.isNil){Error("Install FFmpeg with libx264/MP4, then set SCSHADER_FFMPEG to its absolute executable path").throw};
 ~hydraExportCall.value({|r|~hydraExport.startRecording(~hydraExportFile.value("quadrants.mp4"),~hydraExportEncoder,30,r)});
 5.wait;
 ~hydraExportCall.value({|r|~hydraExport.stopRecording(r)});
 ~hydraExportWait.value;
});
)
// END RECORD

// 4. Queue single o0 at 320x180 render pixels and save a new single-output PNG.
// Resizing clears feedback histories and retains programs, sources and controls.
// BEGIN RESOLUTION
(
~hydraExportRun.value("320x180 single PNG",{
 ~hydraExportCall.value({|r|~hydraExport.render(\o0,r)});
 ~hydraExportCall.value({|r|~hydraExport.setResolution(320,180,r)});
 ~hydraExportCall.value({|r|~hydraExport.screenshot(~hydraExportFile.value("single.png"),r)});
 ~hydraExportWait.value;
 "RESOLUTION READY: 320x180 render pixels; window size is independent".postln;
});
)
// END RESOLUTION

// 5. Queue existing window controls. Fixed render resolution preserves aspect
// with black bars when the window/fullscreen has another aspect ratio.
// BEGIN WINDOW
(
~hydraExportRun.value("window controls",{
 ~hydraExportWindow.size=960@540;
 ~hydraExportWindow.fullscreen=false; // change to true for fullscreen
 ~hydraExportWindow.requestMetrics;
});
)
// END WINDOW

// 6. Optional: queue returning to the window's physical pixel dimensions.
// Retina dimensions may exceed 2048; use a smaller setResolution before exporting.
// BEGIN FOLLOW
(
~hydraExportRun.value("follow window resolution",{
 ~hydraExportCall.value({|r|~hydraExport.setResolution(0,0,r)});
});
)
// END FOLLOW

// 7. Queue cleanup after earlier steps, including MP4 finalization.
// Repeating cleanup after EXPORT STOPPED is safe.
// BEGIN CLEANUP
(
if(~hydraExportReady==true){~hydraExportRun.value("cleanup",{~hydraExportStop.value})}{~hydraExportStop.value};
)
// END CLEANUP
