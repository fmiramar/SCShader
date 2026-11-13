(
var server=ShaderServer.new(\hydraOutputsSmoke,"127.0.0.1",57191), h, done=false, finish, run, index=0, modes, select;
finish={ |ok,message|
    if(done.not) { done=true; message.postln; h.tryPerform(\free);
        server.quit { if(ok) { "SCSHADER_HYDRA_OUTPUTS_OK".postln; 0.exit } { 1.exit } };
        SystemClock.sched(2,{if(ok) {0.exit} {1.exit};nil});
    };
};
modes=[\o0,\o1,\o2,\o3,nil,\legacy,\o2];
select={ |n|
    if(n==modes.size) {
        // This creates a cycle o0 -> o3 -> o0 and must preserve every revision.
        h.src(\o3).out(\o0,{ |bad|
            if(bad[\ok] or: {bad[\code]!="E_HYDRA_COMPILE"}) {finish.value(false,bad)} {
                h.hush({ |quiet|
                    if(quiet[\ok].not) {finish.value(false,quiet)} {
                        h.solid(0.1,0.2,0.3).out(\o3,{ |good|finish.value(good[\ok] and: {good[\revision]==3},"Four outputs, sampling, switches, cycle rollback and hush revisions pass")});
                    };
                });
            };
        });
    } {h.render(modes[n],{ |result|if(result[\ok]) {SystemClock.sched(0.1,{select.value(n+1);nil})} {finish.value(false,result)}})};
};
run={
    if(index==4) { select.value(0) } {
        var expr=if(index==0) { h.osc(10) } { h.src([\o0,\o1,\o2,\o3][index-1]).rotate(0.2).color(0.8,0.5,1) };
        expr.out([\o0,\o1,\o2,\o3][index],{ |result|
            if(result[\ok].not or: {result[\revision]!=1}) {finish.value(false,result)} {index=index+1;run.value};
        });
    };
};
server.rendererPath_("SCSHADER_RENDERER".getenv);
server.waitForBoot({ h=SCHydra.new(server);h.waitForReady({h.render(nil,{ |blank|if(blank[\ok]) {run.value} {finish.value(false,blank)}})},{ |receiver,error|finish.value(false,error)})},{|receiver,error|finish.value(false,error)});
SystemClock.sched(20,{finish.value(false,"Four-output timeout");nil});
)
