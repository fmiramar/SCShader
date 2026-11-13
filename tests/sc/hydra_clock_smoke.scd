(
var server=ShaderServer.new(\hydraClockSmoke,"127.0.0.1",57201),h,done=false,finish,seen=0,clock;
finish={|ok,message|if(done.not){done=true;message.postln;clock.tryPerform(\stop);h.tryPerform(\free);server.quit{if(ok){"SCSHADER_HYDRA_CLOCK_OK".postln;0.exit}{1.exit}};SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
server.rendererPath_("SCSHADER_RENDERER".getenv);
server.waitForBoot({h=SCHydra.new(server);h.waitForReady({
 h.solid(h.param(\pulse,0.1),h.beat(0.1),h.delta(1),1).rotate(h.frame(0.01)).out(\o0,{|committed|
  if(committed[\ok].not){finish.value(false,committed)}{
   h.render(\o0,{|shown|if(shown[\ok].not){finish.value(false,shown)}{
    h.clock(120,1,0);
    clock=TempoClock.new(4);
    Routine({
     var stream=Pseq([0.2,0.7,0.3,0.9],1).asStream;
     4.do {|i|h.setAt(\pulse,stream.next,0.1,\o0,{|receiver,timing|
      var error=timing[\appliedRendererTime]-timing[\scheduledRendererTime];
      error.postln;if(error<0 or:{error>0.25}){finish.value(false,"Timing error outside short smoke bound")}{seen=seen+1;if(seen==4){h.clock(90,0);h.reset(0,90,1);finish.value(true,"TempoClock/Routine/Pseq scheduled Hydra controls and clock changes pass")}};
     });1.wait};
    }).play(clock);
   }});
  };
 });
},{|receiver,error|finish.value(false,error)})},{|receiver,error|finish.value(false,error)});
SystemClock.sched(20,{finish.value(false,"Hydra clock timeout");nil});
)
