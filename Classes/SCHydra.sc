SCHydra : Object {
    var <server, <isFreed;

    *new { |server| ^super.new.init(server) }

    init { |argServer|
        if(argServer.notNil and: { argServer.isKindOf(ShaderServer).not }) {
            Error("SCHydra needs a ShaderServer or nil for offline expression work.").throw;
        };
        server = argServer;
        isFreed = false;
        ^this
    }

    osc { |frequency = 60, sync = 0.1, offset = 0|
        this.checkLive;
        ^SCHydraExpr.new(this, \osc, [frequency, sync, offset])
    }

    noise { |scale = 10, offset = 0.1|
        this.checkLive;
        ^SCHydraExpr.new(this, \noise, [scale, offset])
    }

    snapshot { |expression, output = \o0|
        this.checkLive;
        if(expression.isKindOf(SCHydraExpr).not or: { expression.hydra !== this }) {
            Error("SCHydra snapshot requires an expression from this session.").throw;
        };
        ^expression.snapshot(output)
    }

    outputIndex { |output|
        ^switch(output,
            \o0, { 0 }, \o1, { 1 }, \o2, { 2 }, \o3, { 3 },
            { Error("SCHydra output must be o0, o1, o2, or o3.").throw })
    }

    checkLive {
        if(isFreed) { Error("SCHydra session has been freed.").throw };
        ^this
    }

    free { isFreed = true; ^this }
}
