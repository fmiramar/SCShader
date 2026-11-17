// Step 20: local video through an explicitly installed FFmpeg executable.
// Recompile classes, reopen this file, then run the numbered blocks in order.
// No scsynth boot is needed. Video audio is ignored.

// 1. Choose a short local clip. Wait for VIDEO FILE SELECTED before Step 2.
// Standard macOS FFmpeg locations are checked; SCSHADER_FFMPEG overrides them.
(
~hydraVideoDecoder=(["SCSHADER_FFMPEG".getenv,"/opt/homebrew/bin/ffmpeg","/usr/local/bin/ffmpeg"]
    .detect{|p|p.notNil and:{File.exists(p)}});
if(~hydraVideoDecoder.isNil){Error("Install FFmpeg separately and set SCSHADER_FFMPEG to its absolute executable path").throw};
Dialog.openPanel({|path|~hydraVideoPath=path;("VIDEO FILE SELECTED: "++path).postln});
)

// 2. Start. Expect a moving clip, slightly zoomed and rotated. Wait for VIDEO READY.
// The decoder letterboxes into 640x360 RGBA at 30 fps; use a 16:9 window for matching aspect.
// BEGIN STARTUP
(
if(~hydraVideoServer.notNil){Error("Run cleanup before restarting video").throw};
if(~hydraVideoPath.isNil or:{~hydraVideoDecoder.isNil}){Error("Run Step 1 and choose a local clip first").throw};
~hydraVideoReady=false;
~hydraVideoServer=ShaderServer.new(\hydraVideo,"127.0.0.1",57212);
~hydraVideoAssets=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "shaders";
~hydraVideoServer.rendererArgs_(["--shader",~hydraVideoAssets +/+ "hydra_black.wgsl"]);
~hydraVideoServer.waitForBoot({|server|
 server.sendMsg('/scshader/v1/window/resize',640,360);
 ~hydraVideo=SCHydra.new(server);~hydraVideo.waitForReady({|h|
  h.video(\s0,~hydraVideoPath,~hydraVideoDecoder,true,1,{|accepted|
   if(accepted[\ok]){
    ~hydraVideoDeadline=Main.elapsedTime+10;
    ~hydraVideoPoll={h.videoStatus(\s0,{|status|
     if(status[\ok].not or:{status[\state]=="error" or:{Main.elapsedTime>~hydraVideoDeadline}}){
      status.postln;"VIDEO STARTUP FAILED: inspect the decoder error/status; cleanup before retrying".postln;
     }{
      if(status[\decoded]>0){
       h.src(\s0).scale(1.1).rotate(0.08).out(\o0,{|r|
        if(r[\ok]){h.render(\o0,{|shown|~hydraVideoReady=shown[\ok];if(shown[\ok]){"VIDEO READY: looping local clip with zoom/rotation".postln}{shown.postln}})}{r.postln};
       });
      }{SystemClock.sched(0.1,{if(h.isFreed.not){~hydraVideoPoll.value};nil})};
     };
    })};~hydraVideoPoll.value;
   }{accepted.postln};
  });
 },{|h,error|error.postln});
});
)
// END STARTUP

// 3. Pause: picture freezes; the decoder process is stopped and reaped.
(
~hydraVideo.videoControl(\s0,\pause,0,{|r|r.postln});
)

// 4. Resume from the paused position. Wait for the result between blocks.
(
~hydraVideo.videoControl(\s0,\play,0,{|r|r.postln});
)

// 5. Play at double speed (positive rates 0.1..4).
(
~hydraVideo.videoControl(\s0,\rate,2,{|r|r.postln});
)

// 6. Seek to zero; the next decoded frame replaces the retained picture.
(
~hydraVideo.videoControl(\s0,\seek,0,{|r|r.postln});
)

// 7. Disable looping. At the end, retain the last picture; play restarts from zero.
(
~hydraVideo.videoControl(\s0,\loop,0,{|r|r.postln});
)

// 8. Query state, source position (seconds), published/dropped frame counts and errors.
(
~hydraVideo.videoStatus(\s0,{|r|r.postln});
)

// 9. Clear source: black picture; decoder resources are released.
(
~hydraVideo.clearSource(\s0,{|r|r.postln});
)

// 10. Cleanup. This also stops/reaps any decoder left running.
(
~hydraVideo.free;~hydraVideoServer.quit({~hydraVideoServer=nil;"VIDEO STOPPED".postln});
)
