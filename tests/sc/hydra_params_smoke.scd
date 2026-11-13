(
var server=ShaderServer.new(\hydraParamsSmoke,"127.0.0.1",57197),h,done=false,finish;
finish={|ok,message|if(done.not){done=true;message.postln;h.tryPerform(\free);server.quit{if(ok){"SCSHADER_HYDRA_PARAMS_OK".postln;0.exit}{1.exit}};SystemClock.sched(2,{if(ok){0.exit}{1.exit};nil})}};
server.rendererPath_("SCSHADER_RENDERER".getenv);
server.waitForBoot({
 h=SCHydra.new(server);h.waitForReady({
  var frequency=h.param(\frequency,5),snapshot;
  snapshot=h.osc(frequency).color(h.wave(1),1,1).rotate(h.time(0.1)).scale(h.seq([1,1.2]).smooth).snapshot;
  if(snapshot[\bytes][7]!=1){finish.value(false,"Expected numeric IR minor 1")} {
   h.osc(frequency).color(h.wave(1),1,1).rotate(h.time(0.1)).scale(h.seq([1,1.2]).smooth).out(\o0,{|commit|
    if(commit[\ok].not){finish.value(false,commit)}{
     h.render(\o0,{|shown|if(shown[\ok].not){finish.value(false,shown)}{
      h.set(\frequency,9,\o0,{|updated|if(updated[\ok].not){finish.value(false,updated)}{
       SystemClock.sched(0.3,{h.hush({|quiet|finish.value(quiet[\ok],quiet)});nil});
      }});
     }});
    };
   });
  };
 },{|receiver,error|finish.value(false,error)});
},{|receiver,error|finish.value(false,error)});
SystemClock.sched(20,{finish.value(false,"Numeric parameter timeout");nil});
)
