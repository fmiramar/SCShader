SCHydra : Object {
    classvar all;
    var numbers, namedNumbers, mappedBuses;
    var captureList;
    var customFunctions;
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
        mappedBuses=IdentitySet.new;
        numbers=List.new;namedNumbers=IdentityDictionary.new;
        responders = IdentitySet.new; revisions = [0,0,0,0];
        captureList=[];
        customFunctions=Dictionary.new;
        if(server.notNil and: { server.isRunning }) { this.negotiate };
        ^this
    }

    canStream { ^isFreed.not and:{isReady and:{server.isRunning and:{uploading.not and:{presenting.not}}}} }
    mapBus { |bus,parameter,channel=0,scale=1,bias=0,cutoff=0,output=\o0|
        this.checkLive;if(bus.isKindOf(ShaderBus).not){Error("Hydra mapBus requires ShaderBus").throw};
        mappedBuses=mappedBuses.select {|b|b.hydraMappingCount(this)>0};
        if(mappedBuses.sum {|b|b.hydraMappingCount(this)}>=64){Error("Hydra bus mapping limit is 64").throw};
        bus.mapHydra(this,parameter,channel,scale,bias,cutoff,output);mappedBuses.add(bus);^this
    }
    unmapBus { |bus,parameter,output| bus.unmapHydra(this,parameter,output);if(bus.hydraMappingCount(this)==0){mappedBuses.remove(bus)};^this }

    number { |kind, values|
        var n;this.checkLive;
        if(numbers.size>=256) {Error("Hydra numeric source limit: 256").throw};
        n=SCHydraNumber.new(this,numbers.size+1,kind,values);numbers.add(n);^n
    }
    param { |name,initial=0|
        if(name.isKindOf(Symbol).not) {Error("Hydra parameter names must be Symbols").throw};
        if(namedNumbers[name].notNil) {^namedNumbers[name]};
        namedNumbers[name]=this.number(0,[initial]);^namedNumbers[name]
    }
    time { |rate=1,bias=0| ^this.number(1,[rate,bias]) }
    wave { |rate=1,phase=0,low=0,high=1| ^this.number(2,[rate,phase,low,high]) }
    seq { |values|
        if(values.isSequenceableCollection.not or:{values.size<1 or:{values.size>64}}) {Error("Hydra sequence needs 1..64 values").throw};
        ^this.number(3,[1,0,0,0]++values.asArray)
    }
    beat { |rate=1,bias=0| ^this.number(4,[rate,bias]) }
    frame { |rate=1,bias=0| ^this.number(5,[rate,bias]) }
    delta { |rate=1,bias=0| ^this.number(6,[rate,bias]) }
    clockMessage { |bpm=30,speed=1,phase|
        this.checkLive;
        if(server.isNil or:{server.isRunning.not}) {Error("Hydra renderer is not running; restart the example startup.").throw};
        [bpm,speed,phase ? 0].do {|v|ShaderDef.validateValue(\float,v)};
        if(isReady.not or:{bpm<0 or:{bpm>1000 or:{speed.abs>16 or:{(phase ? 0).abs>1e9}}}}) {Error("Invalid Hydra clock or session not ready").throw};
        ^['/scshader/v1/hydra/clock',epoch,session,this.nextSequence,bpm.asFloat,speed.asFloat,(phase ? 0).asFloat,phase.notNil.if({1},{0})]
    }
    clock { |bpm=30,speed=1,phase| server.sendMsg(*this.clockMessage(bpm,speed,phase));^this }
    reset { |seconds=0,bpm=30,speed=1| ^this.clock(bpm,speed,seconds) }
    parameterMessage { |name,value,output=\o0|
        var n=name.isKindOf(SCHydraNumber).if({name},{namedNumbers[name]}),slot=this.outputIndex(output);
        this.checkLive;ShaderDef.validateValue(\float,value);
        if(isReady.not or:{n.isNil or:{n.hydra!==this or:{n.kind!=0 or:{value.abs>1e6 or:{revisions[slot]==0}}}}}) {Error("Invalid committed Hydra parameter").throw};
        ^['/scshader/v1/hydra/parameter',epoch,session,this.nextSequence,slot,revisions[slot],n.id,value.asFloat]
    }
    setAt { |name,value,delay=0,output=\o0,action|
        ^server.scheduleMarker(delay,action,[this.parameterMessage(name,value,output)])
    }

    set { |name,value,output=\o0,action|
        var n=name.isKindOf(SCHydraNumber).if({name},{namedNumbers[name]}),slot=this.outputIndex(output),tx;
        this.checkLive;ShaderDef.validateValue(\float,value);
        if(n.isNil or:{n.hydra!==this or:{n.kind!=0 or:{value.abs>1e6}}}) {Error("Invalid mutable Hydra parameter").throw};
        if(isReady.not or:{uploading or:{presenting}}) {Error("Hydra is not ready or another request is active").throw};
        tx=this.nextSequence;presenting=true;
        presentTask=Routine({
            var result=this.request(['/scshader/v1/hydra/parameter',epoch,session,tx,slot,revisions[slot],n.id,value.asFloat],"ack",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==tx}}});
            presenting=false;if(result[\ok]) {n.setValue(value)};action.value(result);
        }).play(SystemClock);^this
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

    prev { this.checkLive; ^SCHydraExpr.new(this, \prev, []) }

    exportRequest { |operation,arguments,action|
        var id,suffix=(operation=="export-status").if({"export-status.reply"},{"ack"});
        this.checkLive;if(this.canStream.not){Error("Hydra is not ready or a request is active").throw};
        id=this.nextSequence;presenting=true;
        presentTask=Routine({
            var result=this.request([("/scshader/v1/hydra/"++operation).asSymbol,epoch,session,id]++arguments,suffix,{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            if(result[\ok] and:{operation=="export-status"}){
                var msg=result[\message];result=(ok:true,state:msg[4].asString,path:msg[5].asString,width:msg[6],height:msg[7],fps:msg[8],written:msg[9],dropped:msg[10],buffers:msg[11],error:msg[12].asString);
            };
            presenting=false;lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }
    setResolution { |width,height,action|
        [width,height].do{|v|if(v.isInteger.not or:{v<0 or:{v>4096}}){Error("Resolution must contain integers 1..4096, or 0,0 to follow the window").throw}};
        if((width==0)!=(height==0)){Error("Use 0,0 to follow the window").throw};
        ^this.exportRequest("resolution",[width,height],action)
    }
    screenshot { |path,action| ^this.exportRequest("screenshot",[path.asString.standardizePath],action) }
    startRecording { |path,encoder,fps=30,action|
        if(fps.isInteger.not or:{fps<1 or:{fps>60}}){Error("Recording FPS must be an integer 1..60").throw};
        ^this.exportRequest("record-start",[path.asString.standardizePath,encoder.asString.standardizePath,fps],action)
    }
    stopRecording { |action| ^this.exportRequest("record-stop",[],action) }
    exportStatus { |action| ^this.exportRequest("export-status",[],action) }

    functionIdentifier { |value,limit|
        var text=value.asString.copy,alpha={|c|(c>=65 and:{c<=90}) or:{c>=97 and:{c<=122}}};
        if(text.isEmpty or:{text.size>limit}){Error("Custom identifier length is invalid").throw};
        if(alpha.value(text[0].ascii).not){Error("Custom identifier must start with an ASCII letter").throw};
        if(text.ascii.every{|c|alpha.value(c) or:{c>=48 and:{c<=57}} or:{c==95}}.not){Error("Custom identifier must use ASCII letters/digits/underscore").throw};
        if(text.beginsWith("u_") or:{text.beginsWith("h_")}){Error("Custom identifier prefix is reserved").throw};
        ^text
    }
    functionSpec { |wire|
        var definition=customFunctions.values.detect{|d|d[\wire]==wire.asString},types;
        if(definition.isNil){Error("Custom operation was removed/redefined or is unknown; rebuild the expression").throw};
        types=definition[\inputs].collect{|i|i[1]};
        if([\combine,\combineCoord].includes(definition[\category])){types=[\expr]++types};
        ^[definition[\category],types]
    }
    functionArguments { |name,args|
        var definition=customFunctions[name.asString],values=args.asArray,defaults;
        this.checkLive;
        if(definition.isNil){Error("Register the custom function before calling it").throw};
        defaults=definition[\inputs].collect{|i|i[2].isArray.if({i[2].copy},{i[2]})};
        if([\combine,\combineCoord].includes(definition[\category])){
            if(values.isEmpty){Error("Custom combine needs a second expression").throw};defaults=[nil]++defaults;
        };
        if(values.size>defaults.size){Error("Too many custom function arguments").throw};
        values.do{|value,index|defaults[index]=value};
        ^[definition[\wire],defaults]
    }
    call { |name,args=#[]|
        var prepared=this.functionArguments(name,args);
        ^SCHydraExpr.new(this,prepared[0],prepared[1])
    }
    setFunction { |name,category,inputs,body,action|
        var key=this.functionIdentifier(name,24),categories=[\src,\coord,\color,\combine,\combineCoord],kind=categories.indexOf(category);
        var declarations=List.new,names=Set.new,bytes=List.new,blob,id,definition;
        this.checkLive;if(this.canStream.not){Error("Hydra is not ready or a request is active").throw};
        if(kind.isNil){Error("Invalid custom function category").throw};
        if(inputs.isArray.not or:{inputs.size>8}){Error("Custom function accepts up to eight inputs").throw};
        if(body.isString.not){Error("Custom function body must be WGSL text").throw};
        if(body.isEmpty or:{body.size>2048 or:{body.ascii.any{|c|c<1 or:{c>127}}}}){Error("Custom body must contain 1..2048 ASCII WGSL bytes").throw};
        if(customFunctions[key].isNil and:{customFunctions.size>=16}){Error("Custom function registry limit is 16").throw};
        inputs.do{|input|
            var label,type,value;
            if(input.isArray.not or:{input.size!=3}){Error("Custom input must be [name,type,default]").throw};
            label=this.functionIdentifier(input[0],16);type=input[1];
            if([\float,\vec4].includes(type).not or:{["uv","time","color","other"].includes(label) or:{names.includes(label)}}){Error("Invalid/duplicate/reserved custom input").throw};
            value=ShaderDef.validateValue(type,input[2]);names.add(label);declarations.add([label,type,value.isArray.if({value.copy},{value})]);
        };
        bytes.addAll([72,67,70,78,0,1,key.size,kind,declarations.size,body.size>>8,body.size.bitAnd(255)]);bytes.addAll(key.ascii);
        declarations.do{|input|
            bytes.add(input[0].size);bytes.add((input[1]==\float).if({0},{1}));bytes.addAll(input[0].ascii);
            ShaderDef.encodeValue(input[1],input[2]).do{|b|bytes.add(b.bitAnd(255))};
        };bytes.addAll(body.ascii);
        blob=Int8Array.newFrom(bytes);id=this.nextSequence;presenting=true;
        definition=(category:category,inputs:declarations.asArray);
        presentTask=Routine({
            var result=this.request(['/scshader/v1/hydra/function-set',epoch,session,id,blob],"function-set",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            if(result[\ok]){definition[\wire]=result[\message][4].asString;customFunctions[key]=definition};
            presenting=false;lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }
    removeFunction { |name,action|
        var key=this.functionIdentifier(name,24),id;
        this.checkLive;if(this.canStream.not){Error("Hydra is not ready or a request is active").throw};
        id=this.nextSequence;presenting=true;
        presentTask=Routine({
            var result=this.request(['/scshader/v1/hydra/function-remove',epoch,session,id,key],"ack",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            if(result[\ok]){customFunctions.removeAt(key)};
            presenting=false;lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }

    sourceIndex { |slot|
        ^switch(slot,\s0,{0},\s1,{1},\s2,{2},\s3,{3},{Error("Hydra source must be s0, s1, s2 or s3").throw})
    }
    src { |slot = \o0|
        var ref;
        this.checkLive;
        ref=([\s0,\s1,\s2,\s3].includes(slot)).if({[\source,this.sourceIndex(slot)]},{[\output,this.outputIndex(slot)]});
        ^SCHydraExpr.new(this,\src,[ref])
    }
    image { |slot,path,action|
        var id,index=this.sourceIndex(slot),file=path.isNil.if({""},{path.asString.standardizePath});
        this.checkLive;if(isReady.not or:{server.isRunning.not or:{uploading or:{presenting}}}){Error("Hydra is not ready or a request is active").throw};
        id=this.nextSequence;presenting=true;
        presentTask=Routine({
            var result=this.request(['/scshader/v1/hydra/image',epoch,session,id,index,file],"ack",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            presenting=false;lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }
    videoRequest { |operation,slot,arguments,action|
        var id,index=this.sourceIndex(slot),suffix=(operation=="video-status").if({"video-status.reply"},{"ack"});
        this.checkLive;if(isReady.not or:{server.isRunning.not or:{uploading or:{presenting}}}){Error("Hydra is not ready or a request is active").throw};
        id=this.nextSequence;presenting=true;
        presentTask=Routine({
            var result=this.request([("/scshader/v1/hydra/"++operation).asSymbol,epoch,session,id,index]++arguments,suffix,{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            presenting=false;
            if(result[\ok] and:{operation=="video-status"}) {
                var msg=result[\message];result=(ok:true,state:msg[5].asString,position:msg[6],decoded:msg[7],dropped:msg[8],error:msg[9].asString,
                    kind:msg[10].asString,inputWidth:msg[11],inputHeight:msg[12],width:msg[13],height:msg[14],fps:msg[15]);
            };
            lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }
    video { |slot,path,decoder,loop=true,rate=1,action|
        ^this.videoRequest("video",slot,[path.asString.standardizePath,decoder.asString.standardizePath,loop.if({1},{0}),rate.asFloat],action)
    }
    videoControl { |slot,command,value=0,action| ^this.videoRequest("video-control",slot,[command.asString,value.asFloat],action) }
    videoStatus { |slot=\s0,action| ^this.videoRequest("video-status",slot,[],action) }

    captureDevices { |decoder,action|
        var id,result;
        this.checkLive;if(this.canStream.not){Error("Hydra is not ready or a request is active").throw};
        id=this.nextSequence;presenting=true;
        presentTask=Routine({
            var deadline,finished=false;
            result=this.request(['/scshader/v1/hydra/capture-scan',epoch,session,id,decoder.asString.standardizePath],"ack",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
            if(result[\ok]){
                captureList=[];deadline=Main.elapsedTime+7;
                while{finished.not and:{Main.elapsedTime<deadline}}{
                    id=this.nextSequence;
                    result=this.request(['/scshader/v1/hydra/capture-devices',epoch,session,id],"capture-devices.reply",{|msg|msg[1].asString==epoch and:{msg[2]==session and:{msg[3]==id}}});
                    if(result[\ok].not){finished=true}{
                        var msg=result[\message],state=msg[4].asString;
                        if(state=="ready"){
                            captureList=Array.fill(msg[6],{|i|var n=7+(i*3);(kind:msg[n].asSymbol,index:msg[n+1],name:msg[n+2].asString)});
                            result=(ok:true,state:"ready",devices:captureList);finished=true;
                        }{
                            if(state=="error"){result=(ok:false,code:"E_HYDRA_CAPTURE",message:msg[5].asString);finished=true}{0.1.wait};
                        };
                    };
                };
                if(finished.not){result=(ok:false,code:"E_HYDRA_CAPTURE",message:"Capture discovery timed out; check FFmpeg/macOS permissions")};
            };
            presenting=false;lastError=result[\ok].if({nil},{result});action.value(result);
        }).play(SystemClock);^this
    }
    captureSource { |kind,slot,index,action|
        var selected=index,row;
        if(selected.isNil){
            row=captureList.detect{|d|d[\kind]==kind.asSymbol};
            if(row.isNil){Error("Enumerate capture devices first; no matching camera/screen was found").throw};selected=row[\index];
        };
        if(selected.isInteger.not or:{selected<0 or:{selected>255}}){Error("Capture device index must be an integer in 0..255").throw};
        ^this.videoRequest("capture",slot,[kind.asString,selected],action)
    }
    initCam { |slot=\s0,index,action| ^this.captureSource(\camera,slot,index,action) }
    initScreen { |slot=\s0,index,action| ^this.captureSource(\screen,slot,index,action) }
    captureStatus { |slot=\s0,action| ^this.videoStatus(slot,action) }
    captureControl { |slot,command,action| ^this.videoControl(slot,command,0,action) }

    clearSource { |slot=\s0,action| ^this.image(slot,nil,action) }

    clear { |output = \o0, action|
        this.solid(0,0,0,0).out(output, action); ^this
    }

    hush { |action|
        var clear;
        clear = { |index|
            this.clear([\o0,\o1,\o2,\o3][index], { |result|
                if(result[\ok].not) { action.value(result) } {
                    if(index < 3) { clear.value(index+1) } { this.render(\o0, action) };
                };
            });
        };
        clear.value(0); ^this
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

    render { |output, action|
        var mode, id;
        this.checkLive;
        if(isReady.not) { Error("SCHydra session is not ready.").throw };
        if(uploading or: { presenting }) { Error("SCHydra already has a request in flight.").throw };
        mode = output.isNil.if({ 4 }, { (output == \legacy).if({ -1 }, { this.outputIndex(output) }) });
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
            customFunctions.clear;
            mappedBuses.do {|bus|bus.unmapHydra(this)};mappedBuses.clear;
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
