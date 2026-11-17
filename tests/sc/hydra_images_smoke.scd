(
var file=ShaderDef.filenameSymbol.asString.dirname.dirname +/+ "examples/hydra/05_image_source.scd",text=File.readAllString(file),a,b,done=false,finish;
finish={|ok,msg|if(done.not){done=true;msg.postln;~hydraImage.tryPerform(\free);~hydraImageServer.tryPerform(\quit,{if(ok){"SCSHADER_HYDRA_IMAGES_OK".postln;0.exit}{1.exit}});SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
a=text.find("// BEGIN STARTUP")+"// BEGIN STARTUP".size;b=text.find("// END STARTUP");
text.copyRange(a,b-1).replace("57210","57211").interpret;
Routine({while{~hydraImageReady!=true and:{done.not}}{0.1.wait};if(done.not){
 if(try{~hydraImage.image(\o0,"bad");false}{true}.not){finish.value(false,"Output accepted as image source destination")};
 ~hydraImage.image(\s0,~hydraImageAssets +/+ "hydra_image_second.ppm",{|replaced|
  if(replaced[\ok].not){finish.value(false,replaced)}{
   ~hydraImage.image(\s0,~hydraImageAssets +/+ "missing_image.png",{|bad|
    if(bad[\ok] or:{bad[\code]!="E_HYDRA_IMAGE"}){finish.value(false,bad)}{
     ~hydraImageServer.sendMsg('/scshader/v1/window/resize',320,240);
     SystemClock.sched(0.2,{~hydraImage.clearSource(\s0,{|cleared|finish.value(cleared[\ok],"Source load/replace/failure/resize/clear and source/output kind check")});nil});
    };
   });
  };
 });
}}).play(SystemClock);
SystemClock.sched(20,{finish.value(false,"Image source timeout");nil});
)
