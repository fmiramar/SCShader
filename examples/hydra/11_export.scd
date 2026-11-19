// Step 23. No audio server or screen-capture permission is needed.
// Recompile classes, reopen this file, run numbered blocks and wait for each result.
// 1. Start four outputs at 640x360 render pixels, independent of Retina window size.
// BEGIN STARTUP
(
if(~hydraExportServer.notNil or:{~hydraExportClosing==true}){Error("Run cleanup and wait for EXPORT STOPPED first").throw};
~hydraExportReady=false;
~hydraExportFolder=~hydraExportFolder ? (Platform.userHomeDir +/+ "SCShader-Captures");
if(PathName(~hydraExportFolder).isFolder.not){~hydraExportFolder.mkdir};
~hydraExportName=Date.localtime.stamp ++ "-" ++ Main.elapsedTime.asString.replace(".","_");
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
 if(server.notNil){~hydraExportClosing=true;~hydraExport=nil;~hydraExportServer=nil;~hydraExportReady=false;
  h.tryPerform(\free);~hydraExportWindow.tryPerform(\free);
  server.quit({~hydraExportClosing=false;"EXPORT STOPPED".postln});
 }{(~hydraExportClosing==true).if({"EXPORT STOPPING"},{"EXPORT ALREADY STOPPED"}).postln};
};
)
// END STARTUP

// 2. Save the quadrant presentation to a new PNG. Wait for state: finished.
// Acceptance means queued; finished means the file has actually been written.
(
Routine({
 ~hydraExportCall.value({|r|~hydraExport.screenshot(~hydraExportFolder +/+ (~hydraExportName++"-quadrants.png"),r)});
 ~hydraExportWait.value;
}).play(SystemClock);
)

// 3. Record approximately five seconds of quadrants to MP4 (no audio).
// FFmpeg is an optional, separately installed encoder; it is not bundled.
// Frames may drop; video duration is writtenFrames / FPS, not guaranteed wall time.
// BEGIN RECORD
(
~hydraExportEncoder=(["SCSHADER_FFMPEG".getenv,"/opt/homebrew/bin/ffmpeg","/usr/local/bin/ffmpeg"]
 .detect{|p|p.notNil and:{File.exists(p)}});
if(~hydraExportEncoder.isNil){Error("Install FFmpeg with libx264/MP4, then set SCSHADER_FFMPEG to its absolute executable path").throw};
Routine({
 ~hydraExportCall.value({|r|~hydraExport.startRecording(~hydraExportFolder +/+ (~hydraExportName++"-quadrants.mp4"),~hydraExportEncoder,30,r)});
 5.wait;
 ~hydraExportCall.value({|r|~hydraExport.stopRecording(r)});
 ~hydraExportWait.value;
}).play(SystemClock);
)
// END RECORD

// 4. Select only o0, then use a smaller 320x180 render/export resolution.
// Changing dimensions clears all four feedback histories but retains programs,
// source images and numeric controls. This block also saves a new single-output PNG.
(
Routine({
 ~hydraExportCall.value({|r|~hydraExport.render(\o0,r)});
 ~hydraExportCall.value({|r|~hydraExport.setResolution(320,180,r)});
 "RESOLUTION READY: 320x180 render pixels; window size is independent".postln;
}).play(SystemClock);
)

// 5. Reuse existing window controls. Fixed render resolution preserves aspect
// with black letterboxing when the window/fullscreen has another aspect ratio.
// Render/export stays 320x180 while the window is resized or made fullscreen.
(
~hydraExportWindow.size=960@540;
~hydraExportWindow.fullscreen=false; // change to true to enter fullscreen
~hydraExportWindow.requestMetrics;
)

// 6. Optional: return to following the window's physical pixel dimensions.
// Retina pixels can exceed 2048; exports then require a smaller setResolution.
(
~hydraExport.setResolution(0,0,{|result|result.postln});
)

// 7. Finish recording and wait for finished before cleanup to finalize the MP4.
// Cleanup is safe to repeat; active export is aborted if the renderer is closed.
(
~hydraExportStop.value;
)
