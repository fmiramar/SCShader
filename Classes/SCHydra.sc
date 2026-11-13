SCHydra : Object {
    classvar all;
    var <server, <isFreed, <isReady, <lastError, epoch, session, sequence, startupTask, uploadTask;
    var readyAction, failureAction, responders, uploading, revisions, presentTask, presenting;

    *initClass { all = IdentityDictionary.new }

    *new { |server| ^super.new.init(server) }

    init { |argServer|
        if(argServer.notNil and: { argServer.isKindOf(ShaderServer).not }) {
            Error("SCHydra needs a ShaderServer or nil for offline expression work.").throw;
        };
        if(argServer.notNil and: { all[argServer].notNil }) {
            Error("Only one SCHydra controller may own a ShaderServer.").throw;
        };
        server = argServer;
        if(server.notNil) { all[server] = this };
        isFreed = false; isReady = false; sequence = 0; uploading = false; presenting = false;
        responders = IdentitySet.new; revisions = [0,0,0,0];
        if(server.notNil and: { server.isRunning }) { this.negotiate };
        ^this
    }

    osc { |frequency = 60, sync = 0.1, offset = 0|
        this.checkLive;
        ^SCHydraExpr.new(this, \osc, [frequency, sync, offset])
    }

    solid { |r = 0, g = 0, b = 0, a = 1|
        this.checkLive; ^SCHydraExpr.new(this, \solid, [r, g, b, a])
    }

    gradient { |speed = 0|
        this.checkLive; ^SCHydraExpr.new(this, \gradient, [speed])
    }

    noise { |scale = 10, offset = 0.1|
        this.checkLive;
        ^SCHydraExpr.new(this, \noise, [scale, offset])
    }

    voronoi { |scale = 5, speed = 0.3, blending = 0.3| this.checkLive; ^SCHydraExpr.new(this, \voronoi, [scale, speed, blending]) }
    shape { |sides = 3, radius = 0.3, smoothing = 0.01| this.checkLive; ^SCHydraExpr.new(this, \shape, [sides, radius, smoothing]) }

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

    nextSequence {
        if(sequence == 2147483647) { Error("SCHydra sequence exhausted; reopen session.").throw };
        sequence = sequence + 1; ^sequence
    }

    waitForReady { |action, onFailure|
        this.checkLive;
        if(server.isNil or: { server.isRunning.not }) { Error("Boot ShaderServer before Hydra negotiation.").throw };
        readyAction = action; failureAction = onFailure;
        if(isReady) { action.value(this) } {
            if(lastError.notNil) { onFailure.value(this, lastError) };
        };
        ^this
    }

    exchange { |message, suffix, matches|
        var condition = Condition.new, response, done = false, success, failure;
        success = OSCFunc({ |msg|
            if(matches.value(msg) and: { done.not }) {
                response = (ok: true, message: msg); done = true; condition.unhang;
            };
        }, ("/scshader/v1/hydra/" ++ suffix).asSymbol, server.addr);
        failure = OSCFunc({ |msg|
            if(msg[2].asString == "hydra" and: { done.not }) {
                response = (ok: false, code: msg[4].asString, message: msg[5].asString); done = true; condition.unhang;
            };
        }, '/scshader/v1/error', server.addr);
        responders.add(success).add(failure);
        server.sendMsg(*message);
        SystemClock.sched(0.5, { if(done.not) { done = true; condition.unhang }; nil });
        condition.hang;
        success.free; failure.free; responders.remove(success).remove(failure);
        ^response
    }

    request { |message, suffix, matches|
        var response;
        3.do { if(response.isNil) { response = this.exchange(message, suffix, matches) } };
        ^response ? (ok: false, code: "E_HYDRA_TIMEOUT", message: "Hydra reply timeout")
    }

    negotiate {
        startupTask = Routine({
            var id = this.nextSequence, response;
            response = this.request(['/scshader/v1/hydra/capabilities', id, 1, NetAddr.langPort],
                "capabilities.reply", { |msg| msg[1] == id });
            if(response[\ok]) {
                if(response[\message][3] != 1 or: {
                    response[\message][6].asString.find("staging-ir-v1").isNil
                }) { response = (ok: false, code: "E_HYDRA_VERSION", message: "Unsupported Hydra transport capabilities") };
            };
            if(response[\ok]) {
                epoch = response[\message][2].asString;
                id = this.nextSequence;
                response = this.request(['/scshader/v1/hydra/open', id, epoch,
                    (Main.elapsedTime.asString ++ "-" ++ this.identityHash.asString), 1, NetAddr.langPort],
                    "opened", { |msg| msg[1] == id and: { msg[2].asString == epoch } });
            };
            if(response[\ok]) {
                session = response[\message][3]; isReady = true; readyAction.value(this);
            } { lastError = response; failureAction.value(this, response) };
        }).play(SystemClock);
        ^this
    }

    submit { |expression, output, action|
        var snapshot, tx;
        this.checkLive;
        if(isReady.not or: { server.isRunning.not }) { Error("SCHydra session is not ready.").throw };
        if(uploading or: { presenting }) { Error("SCHydra already has a request in flight.").throw };
        snapshot = this.snapshot(expression, output); tx = this.nextSequence;
        uploading = true;
        uploadTask = Routine({
            var response, statusID;
            response = this.request(['/scshader/v1/hydra/begin', epoch, session, tx,
                this.outputIndex(output), revisions[this.outputIndex(output)], snapshot[\bytes].size], "ack",
                { |msg| msg[1].asString == epoch and: { msg[2] == session and: { msg[3] == tx and: { msg[4] == -1 } } } });
            snapshot[\bytes].clump(2048).do { |chunk, index|
                if(response[\ok]) {
                    response = this.request(['/scshader/v1/hydra/chunk', epoch, session, tx, index,
                        Int8Array.newFrom(chunk)], "ack", { |msg|
                        msg[1].asString == epoch and: { msg[2] == session and: { msg[3] == tx and: { msg[4] == index } } }
                    });
                };
            };
            if(response[\ok]) {
                response = this.request(['/scshader/v1/hydra/commit', epoch, session, tx], "committed",
                    { |msg| msg[1].asString == epoch and: { msg[2] == session and: { msg[3] == tx } } });
                if(response[\code] == "E_HYDRA_TIMEOUT") {
                    statusID = this.nextSequence;
                    response = this.request(['/scshader/v1/hydra/status', epoch, session, statusID, tx],
                        "status.reply", { |msg| msg[1].asString == epoch and: { msg[2] == session and: { msg[3] == statusID } } });
                    if(response[\ok]) {
                        if(response[\message][6].asString == "committed") {
                            response = (ok: true, revision: response[\message][8].asString.asInteger);
                        } {
                            response = (ok: false, code: response[\message][7].asString, message: response[\message][8].asString);
                        };
                        if(response[\code] == "") { response = (ok: false, code: "E_HYDRA_UNKNOWN", message: "Commit outcome unknown") };
                    };
                };
            };
            if(response[\ok]) {
                if(response[\revision].isNil) { response[\revision] = response[\message][5] };
                revisions[this.outputIndex(output)] = response[\revision];
            };
            if(response[\ok].not) { server.sendMsg('/scshader/v1/hydra/abort', epoch, session, tx) };
            response[\transaction] = tx; response[\output] = output;
            uploading = false; lastError = response[\ok].if({ nil }, { response });
            action.value(response);
        }).play(SystemClock);
        ^this
    }

    render { |output = \o0, action|
        var mode, id;
        this.checkLive;
        if(isReady.not) { Error("SCHydra session is not ready.").throw };
        if(uploading or: { presenting }) { Error("SCHydra already has a request in flight.").throw };
        mode = (output == \legacy).if({ -1 }, { this.outputIndex(output) });
        if(mode > 0) { Error("Only Hydra o0 is implemented.").throw };
        id = this.nextSequence;
        presenting = true;
        presentTask = Routine({
            var response = this.request(['/scshader/v1/hydra/present', epoch, session, id, mode],
                "presented", { |msg| msg[1].asString == epoch and: { msg[2] == session and: { msg[3] == id } } });
            presenting = false; lastError = response[\ok].if({ nil }, { response });
            action.value(response);
        }).play(SystemClock);
        ^this
    }

    free {
        if(isFreed.not) {
            if(presentTask !== thisThread) { presentTask.tryPerform(\stop) };
            if(startupTask !== thisThread) { startupTask.tryPerform(\stop) };
            if(uploadTask !== thisThread) { uploadTask.tryPerform(\stop) };
            responders.do(_.free); responders.clear;
            if(isReady and: { server.isRunning }) {
                server.sendMsg('/scshader/v1/hydra/close', epoch, session, this.nextSequence);
            };
            if(server.notNil and: { all[server] === this }) { all.removeAt(server) };
            isFreed = true; isReady = false;
        };
        ^this
    }
}
