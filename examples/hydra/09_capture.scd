// Step 21: macOS camera and display capture through external FFmpeg/AVFoundation.
// Recompile classes, reopen this file, run blocks in order, and wait for each READY/result.
// No scsynth boot is needed. Microphones/audio are never opened.

// 1. Start and list devices. This enumerates; it does not start camera/screen capture.
// macOS Camera and Screen Recording permissions apply to the launching app/FFmpeg.
// BEGIN STARTUP
(
if(~hydraCaptureServer.notNil or:{~hydraCaptureClosing==true}){Error("Run cleanup and wait for CAPTURE STOPPED before restarting capture").throw};
~hydraCaptureDecoder=(["SCSHADER_FFMPEG".getenv,"/opt/homebrew/bin/ffmpeg","/usr/local/bin/ffmpeg"].detect{|p|p.notNil and:{File.exists(p)}});
if(~hydraCaptureDecoder.isNil){Error("Install FFmpeg separately; set SCSHADER_FFMPEG to its absolute path").throw};
~hydraCaptureDevicesReady=false;
~hydraCaptureCameraIndex=nil;~hydraCaptureScreenIndex=nil;
~hydraCaptureServer=ShaderServer.new(\hydraCapture,"127.0.0.1",57214);
~hydraCaptureAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders";
~hydraCaptureServer.rendererArgs_(["--shader",~hydraCaptureAssets +/+ "hydra_black.wgsl"]);
~hydraCaptureServer.waitForBoot({|server|
 ~hydraCapture=SCHydra.new(server);~hydraCapture.waitForReady({|h|
  h.captureDevices(~hydraCaptureDecoder,{|result|
   if(result[\ok]){
    ~hydraCaptureDevices=result[\devices];~hydraCaptureDevices.do{|device|device.postln};
    ~hydraCaptureDevicesReady=true;"CAPTURE DEVICES READY: choose indexes below, or use the first of each kind".postln;
   }{result.postln};
  });
 },{|h,error|error.postln});
});
// Wait for a real frame before presenting. Used by the following blocks and the smoke check.
~hydraCaptureStart={|kind,slot,output,index,action|
 var h=~hydraCapture,poll,accepted,permissionNotice=false,deadline=Main.elapsedTime+(kind==\camera).if({75},{12});
 poll={
  if(h.isFreed.not){h.captureStatus(slot,{|status|
   if(status[\ok].not or:{status[\state]=="error" or:{Main.elapsedTime>deadline}}){
    status.postln;("CAPTURE FAILED: "++kind++"; inspect permissions/device, then clear or cleanup").postln;action.value((ok:false,message:status));
   }{
    if(status[\state]=="permission" and:{permissionNotice.not}){permissionNotice=true;"CAMERA PERMISSION: approve SCShader Camera in the macOS prompt to receive frames".postln};
    if(status[\decoded]>0 and:{status[\inputWidth]>0}){
     h.src(slot).scale(1.05).out(output,{|committed|
      if(committed[\ok]){h.render(output,{|shown|if(shown[\ok]){(kind.asString.toUpper++" READY: "++status[\inputWidth]++"x"++status[\inputHeight]++" -> 640x360 RGBA").postln};action.value(shown)})}{action.value(committed)};
     });
    }{SystemClock.sched(0.1,{poll.value;nil})};
   };
  })};
 };
 accepted={|result|if(result[\ok]){poll.value}{result.postln;action.value(result)}};
 if(kind==\camera){h.initCam(slot,index,accepted)}{h.initScreen(slot,index,accepted)};
};
~hydraCaptureStop={
 var h=~hydraCapture,server=~hydraCaptureServer;
 if(server.notNil){
  ~hydraCaptureClosing=true;~hydraCapture=nil;~hydraCaptureServer=nil;~hydraCaptureDevicesReady=false;
  h.tryPerform(\free);server.quit({~hydraCaptureClosing=false;"CAPTURE STOPPED".postln});
 }{(~hydraCaptureClosing==true).if({"CAPTURE STOPPING"},{"CAPTURE ALREADY STOPPED"}).postln};
};
)
// END STARTUP

// Optional selection: copy indexes from the printed list, then run the relevant block.
// ~hydraCaptureCameraIndex=0;  // camera device index
// ~hydraCaptureScreenIndex=2;  // example only: use YOUR listed screen index

// 2. Explicitly start the selected camera in s0/o0. Wait for CAMERA READY.
// On first use, macOS asks for SCShader Camera access. Allow it to receive frames.
// Nil index chooses the first enumerated camera. Expect a slightly zoomed live image.
(
~hydraCaptureStart.value(\camera,\s0,\o0,~hydraCaptureCameraIndex,{|result|result.postln});
)

// 3. Explicitly start the selected display in s1/o1. Wait for SCREEN READY.
// This captures the entire chosen display. Showing this window can create visual recursion.
(
~hydraCaptureStart.value(\screen,\s1,\o1,~hydraCaptureScreenIndex,{|result|result.postln});
)

// 4. Show both: camera top-left, screen top-right; unused bottom outputs are black.
(
~hydraCapture.render(nil,{|result|result.postln});
)

// 5. Pause camera: retain its picture and release the camera process. Screen stays live.
(
~hydraCapture.captureControl(\s0,\pause,{|result|result.postln});
)

// 6. Resume camera. Live sources support play/pause; they cannot seek/loop/change rate.
(
~hydraCapture.captureControl(\s0,\play,{|result|result.postln});
)

// 7. Query native input size, uploaded size/fps, frame counts and asynchronous errors.
(
~hydraCapture.captureStatus(\s0,{|camera|
 camera.postln;~hydraCapture.captureStatus(\s1,{|screen|screen.postln});
});
)

// 8. Stop and clear both sources. Their processed outputs become black.
(
~hydraCapture.clearSource(\s0,{|result|result.postln;
 if(result[\ok]){~hydraCapture.clearSource(\s1,{|screen|screen.postln})};
});
)

// 9. Cleanup releases all captures and discovery resources; run before restarting.
(
~hydraCaptureStop.value;
)
