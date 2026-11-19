// Opt-in: running this test authorizes live camera and entire-display capture.
(
var root=ShaderDef.filenameSymbol.asString.dirname.dirname,text=File.readAllString(root +/+ "examples/hydra/09_capture.scd"),a,b,done=false,finish,request,call,check,status;
finish={|ok,msg|if(done.not){done=true;msg.postln;~hydraCapture.tryPerform(\free);~hydraCaptureServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_CAPTURE_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
request={|operation|var condition=Condition.new,result;operation.value({|r|result=r;condition.unhang});condition.hang;result};
call={|operation|var result=request.value(operation);if(result[\ok].not){Error(result.asString).throw};result};
check={|condition,message|if(condition.not){Error(message).throw}};
status={|slot|call.value({|reply|~hydraCapture.captureStatus(slot,reply)})};
a=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;b=text.find("// END STARTUP");
text.copyRange(a,b-1).replace("57214","57215").interpret;
Routine({
 while{~hydraCaptureDevicesReady!=true and:{done.not}}{0.1.wait};
 if(done.not){try{
  var camera,screen,paused,next,rejected;
  check.value(~hydraCaptureDevices.any{|d|d[\kind]==\camera},"No camera enumerated");
  check.value(~hydraCaptureDevices.any{|d|d[\kind]==\screen},"No display enumerated");
  rejected=request.value({|reply|~hydraCapture.initScreen(\s2,~hydraCaptureDevices.detect{|d|d[\kind]==\camera}[\index],reply)});
  check.value(rejected[\ok].not and:{rejected[\code]=="E_HYDRA_CAPTURE"},"Wrong capture kind was accepted");
  call.value({|reply|~hydraCaptureStart.value(\camera,\s0,\o0,nil,reply)});
  call.value({|reply|~hydraCaptureStart.value(\screen,\s1,\o1,nil,reply)});
  call.value({|reply|~hydraCapture.render(nil,reply)});
  0.3.wait;camera=status.value(\s0);screen=status.value(\s1);
  [camera,screen].do{|s|check.value(s[\decoded]>1 and:{s[\width]==640 and:{s[\height]==360 and:{s[\inputWidth]>0 and:{s[\inputHeight]>0}}}},"Capture frame/size metadata failed")};
  check.value(camera[\kind]=="camera" and:{screen[\kind]=="screen"},"Capture kind metadata failed");
  rejected=request.value({|reply|~hydraCapture.videoControl(\s0,\seek,1,reply)});
  check.value(rejected[\ok].not and:{rejected[\code]=="E_HYDRA_VIDEO"},"Live camera incorrectly accepted seek");
  call.value({|reply|~hydraCapture.captureControl(\s0,\pause,reply)});
  paused=status.value(\s0);0.15.wait;next=status.value(\s0);
  check.value(next[\state]=="paused" and:{next[\decoded]==paused[\decoded]},"Camera pause failed");
  check.value(status.value(\s1)[\decoded]>screen[\decoded],"Camera pause stopped screen capture");
  call.value({|reply|~hydraCapture.captureControl(\s0,\play,reply)});0.2.wait;
  ~hydraCaptureServer.sendMsg('/scshader/v1/window/resize',480,270);0.1.wait;
  call.value({|reply|~hydraCapture.clearSource(\s1,reply)});
  call.value({|reply|~hydraCapture.image(\s0,~hydraCaptureAssets +/+ "hydra_image_test.ppm",reply)});
  // Reopen a running source; session disposal must release it.
  call.value({|reply|~hydraCapture.initCam(\s0,nil,reply)});0.1.wait;
  finish.value(true,"Installed camera/screen example: selection, two live effects, metadata, independent pause/resume, resize, clear, image replacement and free pass");
 }{|error|finish.value(false,error.asString)}};
}).play(SystemClock);
SystemClock.sched(25,{finish.value(false,"Camera/screen capture timeout; inspect macOS permissions");nil});
)
