(
var root=ShaderDef.filenameSymbol.asString.dirname.dirname;
var text=File.readAllString(root +/+ "examples/hydra/06_video_file.scd"),a,b,done=false,finish,call,status,check;
finish={|ok,msg|if(done.not){done=true;msg.postln;~hydraVideo.tryPerform(\free);~hydraVideoServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_VIDEO_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
~hydraVideoPath="SCSHADER_TEST_VIDEO".getenv;
~hydraVideoDecoder="SCSHADER_FFMPEG".getenv;
if(~hydraVideoPath.isNil or:{~hydraVideoDecoder.isNil}){Error("Supply SCSHADER_TEST_VIDEO (one-second clip) and SCSHADER_FFMPEG").throw};
call={|operation|
 var condition=Condition.new,result;
 operation.value({|r|result=r;condition.unhang});condition.hang;
 if(result[\ok].not){Error(result.asString).throw};result
};
status={call.value({|reply|~hydraVideo.videoStatus(\s0,reply)})};
check={|predicate,message|if(predicate.not){Error(message).throw}};
a=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;b=text.find("// END STARTUP");
text.copyRange(a,b-1).replace("57212","57213").interpret;
Routine({
 while{~hydraVideoReady!=true and:{done.not}}{0.1.wait};
 if(done.not){try{
  var initial,paused,next,deadline;
  1.3.wait;initial=status.value;
  check.value(initial[\state]=="playing" and:{initial[\position]>1},"Local clip did not loop");
  call.value({|r|~hydraVideo.videoControl(\s0,\pause,0,r)});
  paused=status.value;0.15.wait;next=status.value;
  check.value(next[\state]=="paused" and:{next[\decoded]==paused[\decoded]},"Pause did not freeze decoding");
  call.value({|r|~hydraVideo.videoControl(\s0,\rate,2,r)});
  call.value({|r|~hydraVideo.videoControl(\s0,\seek,0.2,r)});
  call.value({|r|~hydraVideo.videoControl(\s0,\loop,0,r)});
  call.value({|r|~hydraVideo.videoControl(\s0,\play,0,r)});
  deadline=Main.elapsedTime+4;
  next=status.value;
  while{next[\state]!="ended" and:{Main.elapsedTime<deadline}}{0.05.wait;next=status.value};
  check.value(next[\state]=="ended" and:{next[\decoded]>paused[\decoded]},"Rate/seek/nonlooping end failed");
  ~hydraVideoServer.sendMsg('/scshader/v1/window/resize',480,270);0.1.wait;
  call.value({|r|~hydraVideo.videoControl(\s0,\loop,1,r)});
  call.value({|r|~hydraVideo.videoControl(\s0,\play,0,r)});
  0.15.wait;
  call.value({|r|~hydraVideo.image(\s0,~hydraVideoAssets +/+ "hydra_image_test.ppm",r)});
  // Replacing a running video with an image releases the decoder.
  call.value({|r|~hydraVideo.video(\s0,~hydraVideoPath,~hydraVideoDecoder,true,1,r)});
  0.15.wait;
  call.value({|r|~hydraVideo.clearSource(\s0,r)});
  // Restart once more: session free must also release an active decoder.
  call.value({|r|~hydraVideo.video(\s0,~hydraVideoPath,~hydraVideoDecoder,true,1,r)});
  0.15.wait;
  finish.value(true,"Installed video example: loop/effect/pause/rate/seek/EOS/resize/image replacement/clear/free pass");
 }{|error|finish.value(false,error.asString)}};
}).play(SystemClock);
SystemClock.sched(20,{finish.value(false,"Video source timeout");nil});
)
