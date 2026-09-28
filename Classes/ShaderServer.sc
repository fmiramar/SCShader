ShaderServer : Object {
    classvar defaultServer, localServer, all;

    var <name, <addr, <rendererPath, <rendererArgs, <pid, <window;
    var <isRunning, <isBooting, <latency, <version, <backend, <device;
    var <lastStatus, <lastError, <lastTiming, <dumpOSC;
    var <showStats, <gpuRecoveryCount, <lastRecovery, <>recoveryAction;
    var <rendererClockOffset, <clockRoundTrip, <clockSampleCount;
    var bootCallbacks, quitCallbacks, statusCallbacks, syncCallbacks, markerCallbacks;
    var shaders;
    var readyResponder, pongResponder, statusResponder, errorResponder;
    var shaderCreatedResponder, shaderReloadedResponder, shaderFreedResponder, shaderReflectionResponder, timingMarkerResponder;
    var windowMetricsResponder, inputMouseResponder, inputKeyResponder, inputResizeResponder, inputFocusResponder;
    var uniformValueResponder;
    var gpuRecoveredResponder;
    var bootStartedAt, sequence, quitting;

    *initClass {
        all = IdentitySet.new;
        localServer = this.new(\local);
        defaultServer = localServer;
        CmdPeriod.add(this);
    }

    *new { |name = \local, host = "127.0.0.1", port = 57140|
        ^super.new.init(name, NetAddr(host, port))
    }

    *default { ^defaultServer }

    *local { ^localServer }

    *all { ^all.copy }

    *rendererExecutableName {
        ^(thisProcess.platform.name == \windows).if({ "scshader-renderer.exe" }, { "scshader-renderer" })
    }

    *cmdPeriod {
        // Cmd-. asks every locally launched renderer to quit. A manually
        // connected remote renderer is never terminated because it has no pid.
        all.copy.do { |server| server.quit };
    }

    init { |argName, argAddr|
        name = argName.asSymbol;
        addr = argAddr;
        rendererArgs = Array.new;
        latency = 0.02;
        isRunning = false;
        isBooting = false;
        quitting = false;
        dumpOSC = false;
        showStats = false;
        gpuRecoveryCount = 0;
        sequence = 0;
        bootCallbacks = List.new;
        quitCallbacks = List.new;
        statusCallbacks = List.new;
        syncCallbacks = IdentityDictionary.new;
        markerCallbacks = IdentityDictionary.new;
        shaders = IdentityDictionary.new;
        rendererClockOffset = nil;
        clockRoundTrip = nil;
        clockSampleCount = 0;
        all.add(this);
        ^this
    }

    rendererPath_ { |path|
        if(isBooting or: { isRunning }) {
            Error("Set ShaderServer.rendererPath before booting the renderer.").throw;
        };
        rendererPath = path.isNil.if({ nil }, { path.asString.standardizePath });
        ^this
    }

    rendererArgs_ { |args|
        if(isBooting or: { isRunning }) {
            Error("Set ShaderServer.rendererArgs before booting the renderer.").throw;
        };
        rendererArgs = args.asArray.collect(_.asString);
        ^this
    }

    dumpOSC_ { |enabled|
        dumpOSC = enabled.asBoolean;
        ^this
    }

    rendererCommand {
        var windowArgs = window.isNil.if({ #[] }, { window.rendererArgs });
        ^[this.resolvedRendererPath] ++ rendererArgs ++ windowArgs ++ ["--listen-port", addr.port.asString]
    }

    showStats_ { |enabled|
        showStats = enabled.asBoolean;
        if(isRunning) {
            this.sendMsg('/scshader/v1/diagnostics/overlay', showStats.asInteger);
        };
        ^this
    }

    resolvedRendererPath {
        var candidate = rendererPath ? this.packagedRendererPath;
        var configured = "SCSHADER_RENDERER".getenv;
        if(candidate.isNil and: { configured.notNil and: { configured.size > 0 } }) {
            candidate = configured.standardizePath;
        };
        if(candidate.isNil) {
            var separator = (thisProcess.platform.name == \windows).if({ $; }, { $: });
            candidate = ("PATH".getenv ? "").split(separator).reject(_.isEmpty).collect { |directory|
                directory +/+ this.class.rendererExecutableName
            }.detect { |path| PathName(path).isFile };
        };
        if(candidate.isNil or: { PathName(candidate).isFile.not }) {
            Error(
                "SCShader renderer executable not found. Set "
                ++ "ShaderServer.default.rendererPath_(aRendererExecutable) before booting."
            ).throw;
        };
        ^candidate
    }

    packagedRendererPath {
        var executable = this.class.rendererExecutableName;
        var classRoot = ShaderServer.filenameSymbol.asString.dirname.dirname;
        var root = Platform.userExtensionDir +/+ "SCShader";
        var candidates = [
            classRoot +/+ "renderer" +/+ executable,
            classRoot +/+ executable,
            root +/+ "renderer" +/+ executable,
            root +/+ executable
        ];
        ^candidates.detect { |candidate| PathName(candidate).isFile };
    }

    boot { |onFailure|
        var command;
        if(quitting) { quitCallbacks.add({ this.boot(onFailure) }); ^this };
        if(isRunning) { ^this };
        if(isBooting) {
            "ShaderServer '%' is already booting.".format(name).postln;
            ^this
        };
        command = this.rendererCommand;
        this.installResponders;
        isBooting = true;
        quitting = false;
        lastError = nil;
        lastStatus = nil;
        gpuRecoveryCount = 0;
        lastRecovery = nil;
        rendererClockOffset = nil;
        clockRoundTrip = nil;
        clockSampleCount = 0;
        bootStartedAt = Main.elapsedTime;
        pid = command.unixCmd({ |exitCode, processID|
            this.processExited(exitCode, processID);
        });
        this.sendHello;
        this.retryHello(onFailure);
        ^this
    }

    waitForBoot { |action, onFailure|
        if(quitting) { quitCallbacks.add({ this.waitForBoot(action, onFailure) }); ^this };
        if(isRunning) {
            action.value(this);
        } {
            bootCallbacks.add([action, onFailure]);
            if(isBooting.not) { this.boot(onFailure) };
        };
        ^this
    }

    reboot { |onComplete, onFailure|
        this.quit({
            this.waitForBoot(onComplete, onFailure);
        });
        ^this
    }

    quit { |onComplete|
        var child = pid;
        if(onComplete.notNil) { quitCallbacks.add(onComplete) };
        if(pid.isNil) {
            this.finishQuit;
            ^this
        };
        quitting = true;
        this.sendMsg('/scshader/v1/quit');
        SystemClock.sched(1.0, {
            // A previous quit's fallback must never terminate a replacement child.
            if(quitting and: { pid == child }) { this.terminateChild };
            nil
        });
        ^this
    }

    free {
        this.quit;
        all.remove(this);
        ^this
    }

    sendMsg { |... args|
        if(args.isEmpty) { Error("ShaderServer.sendMsg needs an OSC address.").throw };
        if(dumpOSC) { [\SCShader, \send, addr, args].postln };
        addr.sendMsg(*args);
        ^this
    }

    sendBundle { |time ... messages|
        if(messages.isEmpty) { Error("ShaderServer.sendBundle needs at least one OSC message.").throw };
        if(dumpOSC) { [\SCShader, \bundle, addr, time, messages].postln };
        addr.sendBundle(time, *messages);
        ^this
    }

    scheduleMarker { |time, action, messages = #[]|
        var id = this.nextSequence;
        var bundle = messages.asArray ++ [['/scshader/v1/timing/marker', id]];
        if(isRunning.not) { Error("ShaderServer '%' is not running.".format(name)).throw };
        if(action.notNil) { markerCallbacks[id] = action };
        this.sendBundle(time, *bundle);
        ^id
    }

    sync { |timeout = 1.0, action|
        var condition, id, completed = false, result;
        if(isRunning.not) { Error("ShaderServer '%' is not running.".format(name)).throw };
        id = this.nextSequence;
        condition = Condition.new;
        syncCallbacks[id] = { |reply|
            completed = true;
            result = reply;
            condition.unhang;
            action.tryPerform(\value, this, reply);
        };
        this.sendMsg('/scshader/v1/ping', id, Main.elapsedTime);
        SystemClock.sched(timeout, {
            var callback = syncCallbacks.removeAt(id);
            if(callback.notNil) { condition.unhang };
            nil
        });
        if(action.notNil) { ^this };
        condition.hang;
        if(completed.not) { Error("ShaderServer sync timed out after % seconds.".format(timeout)).throw };
        ^result
    }

    clockEstimate {
        ^(
            rendererClockOffset: rendererClockOffset,
            roundTrip: clockRoundTrip,
            sampleCount: clockSampleCount
        )
    }

    clockSync { |samples = 8, interval = 0.02, timeout = 1.0, action|
        var count = samples.asInteger.max(1);
        if(isRunning.not) { Error("ShaderServer '%' is not running.".format(name)).throw };
        Routine({
            count.do {
                this.sync(timeout);
                interval.wait;
            };
            action.tryPerform(\value, this, this.clockEstimate);
        }).play(SystemClock);
        ^this
    }

    status { |action|
        if(isRunning.not) { Error("ShaderServer '%' is not running.".format(name)).throw };
        if(action.notNil) { statusCallbacks.add(action) };
        this.sendMsg('/scshader/v1/status');
        ^lastStatus
    }

    clearScheduled {
        if(isRunning.not) { Error("Boot ShaderServer before clearing scheduled events.").throw };
        markerCallbacks.clear;
        this.sendMsg('/scshader/v1/schedule/clear');
        ^this
    }

    registerShader { |shader|
        shaders[shader.id] = shader;
        ^shader
    }

    unregisterShader { |shader|
        if(shaders[shader.id] === shader) { shaders.removeAt(shader.id) };
        ^shader
    }

    registerWindow { |aWindow|
        if(window.notNil and: { window !== aWindow }) {
            Error("SCShader currently supports one ShaderWindow per ShaderServer.").throw;
        };
        window = aWindow;
        if(isRunning) { window.receivedReady };
        ^window
    }

    unregisterWindow { |aWindow|
        if(window === aWindow) { window = nil };
        ^aWindow
    }

    installResponders {
        var source = addr;
        var receivePort = NetAddr.langPort;
        readyResponder = OSCdef(this.responderKey(\ready), { |message, time, sourceAddr|
            this.receivedReady(message, time, sourceAddr);
        }, '/scshader/v1/ready', source, receivePort);
        pongResponder = OSCdef(this.responderKey(\pong), { |message, time, sourceAddr|
            this.receivedPong(message, time, sourceAddr);
        }, '/scshader/v1/pong', source, receivePort);
        statusResponder = OSCdef(this.responderKey(\status), { |message, time, sourceAddr|
            this.receivedStatus(message, time, sourceAddr);
        }, '/scshader/v1/status.reply', source, receivePort);
        errorResponder = OSCdef(this.responderKey(\error), { |message, time, sourceAddr|
            this.receivedError(message, time, sourceAddr);
        }, '/scshader/v1/error', source, receivePort);
        shaderCreatedResponder = OSCdef(this.responderKey(\shaderCreated), { |message, time, sourceAddr|
            this.receivedShaderCreated(message, time, sourceAddr);
        }, '/scshader/v1/shader/created', source, receivePort);
        shaderReloadedResponder = OSCdef(this.responderKey(\shaderReloaded), { |message, time, sourceAddr|
            this.receivedShaderReloaded(message, time, sourceAddr);
        }, '/scshader/v1/shader/reloaded', source, receivePort);
        shaderFreedResponder = OSCdef(this.responderKey(\shaderFreed), { |message, time, sourceAddr|
            this.receivedShaderFreed(message, time, sourceAddr);
        }, '/scshader/v1/shader/freed', source, receivePort);
        shaderReflectionResponder = OSCdef(this.responderKey(\shaderReflection), { |message, time, sourceAddr|
            this.receivedShaderReflection(message, time, sourceAddr);
        }, '/scshader/v1/shader/reflection', source, receivePort);
        uniformValueResponder = OSCdef(this.responderKey(\uniformValue), { |message|
            if(message.size == 6) {
                shaders[message[1].asInteger].tryPerform(\receivedValue, message[2].asSymbol,
                    message[3].asInteger, message[4].asSymbol, message[5]);
            };
        }, '/scshader/v1/uniform/value', source, receivePort);
        timingMarkerResponder = OSCdef(this.responderKey(\timingMarker), { |message, time, sourceAddr|
            this.receivedTimingMarker(message, time, sourceAddr);
        }, '/scshader/v1/timing/marker.reply', source, receivePort);
        windowMetricsResponder = OSCdef(this.responderKey(\windowMetrics), { |message, time, sourceAddr|
            this.receivedWindowMetrics(message, time, sourceAddr);
        }, '/scshader/v1/window/metrics.reply', source, receivePort);
        inputMouseResponder = OSCdef(this.responderKey(\inputMouse), { |message, time, sourceAddr|
            this.receivedInputMouse(message, time, sourceAddr);
        }, '/scshader/v1/input/mouse', source, receivePort);
        inputKeyResponder = OSCdef(this.responderKey(\inputKey), { |message, time, sourceAddr|
            this.receivedInputKey(message, time, sourceAddr);
        }, '/scshader/v1/input/key', source, receivePort);
        inputResizeResponder = OSCdef(this.responderKey(\inputResize), { |message, time, sourceAddr|
            this.receivedInputResize(message, time, sourceAddr);
        }, '/scshader/v1/input/resize', source, receivePort);
        inputFocusResponder = OSCdef(this.responderKey(\inputFocus), { |message, time, sourceAddr|
            this.receivedInputFocus(message, time, sourceAddr);
        }, '/scshader/v1/input/focus', source, receivePort);
        gpuRecoveredResponder = OSCdef(this.responderKey(\gpuRecovered), { |message|
            this.receivedGpuRecovered(message);
        }, '/scshader/v1/gpu/recovered', source, receivePort);
        ^this
    }

    stopResponders {
        [readyResponder, pongResponder, statusResponder, errorResponder,
            shaderCreatedResponder, shaderReloadedResponder, shaderFreedResponder, shaderReflectionResponder, uniformValueResponder,
            timingMarkerResponder, windowMetricsResponder, inputMouseResponder, inputKeyResponder,
            inputResizeResponder, inputFocusResponder, gpuRecoveredResponder].do { |responder|
            responder.tryPerform(\free);
        };
        readyResponder = nil;
        pongResponder = nil;
        statusResponder = nil;
        errorResponder = nil;
        shaderCreatedResponder = nil;
        shaderReloadedResponder = nil;
        shaderFreedResponder = nil;
        shaderReflectionResponder = nil;
        uniformValueResponder = nil;
        timingMarkerResponder = nil;
        windowMetricsResponder = nil;
        inputMouseResponder = nil;
        inputKeyResponder = nil;
        inputResizeResponder = nil;
        inputFocusResponder = nil;
        gpuRecoveredResponder = nil;
        ^this
    }

    receivedReady { |message, time, source|
        // Retries can queue several hello replies during GPU initialization.
        // Only the first completes a boot; later replies must not replay window
        // state or revive a stopped/quitting controller.
        if(isBooting.not or: { quitting }) { ^this };
        if(message.size < 5 or: { message[1] != 1 }) {
            this.bootFailed("renderer sent an invalid /scshader/v1/ready reply");
            ^this
        };
        isBooting = false;
        isRunning = true;
        version = message[2].asString;
        backend = message[3].asString;
        device = message[4].asString;
        if(showStats) { this.showStats_(true) };
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        window.tryPerform(\receivedReady);
        bootCallbacks.do { |pair| pair[0].tryPerform(\value, this) };
        bootCallbacks.clear;
        ^this
    }

    receivedPong { |message, time, source|
        var id = message[1].asInteger, receivedAt = Main.elapsedTime;
        var sentAt = message[2].asFloat, rendererTime = message[3].asFloat;
        var roundTrip = (receivedAt - sentAt).max(0.0);
        var candidateOffset = rendererTime - (sentAt + (roundTrip * 0.5));
        var callback = syncCallbacks.removeAt(id);
        clockSampleCount = clockSampleCount + 1;
        if(clockRoundTrip.isNil or: { roundTrip <= clockRoundTrip }) {
            clockRoundTrip = roundTrip;
            rendererClockOffset = candidateOffset;
        };
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        callback.tryPerform(\value, message);
        ^this
    }

    receivedStatus { |message, time, source|
        if(message.size < 10) {
            this.receivedError([
                '/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed status reply"
            ], time, source);
            ^this
        };
        lastStatus = (
            fps: message[1].asFloat,
            frameIndex: message[2].asInteger,
            gpuFrameMs: message[3].asFloat,
            cpuFrameMs: message[4].asFloat,
            scheduledEventCount: message[5].asInteger,
            shaderCount: message[6].asInteger,
            textureCount: message[7].asInteger,
            bufferCount: message[8].asInteger,
            windowCount: message[9].asInteger,
            scheduledPayloadBytes: (message[10] ? 0).asInteger,
            rejectedScheduled: (message[11] ? 0).asFloat,
            droppedContinuous: (message[12] ? 0).asFloat,
            lateEventCount: (message[13] ? 0).asFloat,
            textureBytesEstimate: (message[14] ? 0).asFloat,
            showStats: (message[15] ? 0).asInteger == 1,
            gpuRecoveryCount: (message[16] ? 0).asInteger,
            lastCompileOK: (message[17] ? 1).asInteger == 1,
            suppressedDiagnostics: (message[18] ? 0).asFloat
        );
        gpuRecoveryCount = lastStatus[\gpuRecoveryCount];
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        statusCallbacks.do { |callback| callback.value(this, lastStatus) };
        statusCallbacks.clear;
        ^lastStatus
    }

    receivedGpuRecovered { |message|
        if(message.size < 3) { ^this };
        gpuRecoveryCount = message[1].asInteger;
        if(message.size >= 5) {
            backend = message[3].asString;
            device = message[4].asString;
        };
        lastRecovery = (count: gpuRecoveryCount, message: message[2].asString,
            feedbackHistoryCleared: true);
        recoveryAction.value(this, lastRecovery);
        ^this
    }

    receivedError { |message, time, source|
        lastError = (
            severity: message[1].asString,
            subsystem: message[2].asString,
            resourceID: message[3].asInteger,
            code: message[4].asString,
            message: message[5].asString
        );
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        ("ShaderServer % error %: %".format(name, lastError[\code], lastError[\message])).warn;
        shaders[lastError[\resourceID]].tryPerform(\receivedError, lastError);
        ^lastError
    }

    receivedShaderCreated { |message, time, source|
        var shader = shaders[message[1].asInteger];
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        shader.tryPerform(\receivedCreated);
        ^this
    }

    receivedShaderReloaded { |message, time, source|
        var shader = shaders[message[1].asInteger];
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        shader.tryPerform(\receivedReloaded);
        ^this
    }

    receivedShaderFreed { |message, time, source|
        var shader = shaders[message[1].asInteger];
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        shader.tryPerform(\receivedFreed);
        ^this
    }

    receivedShaderReflection { |message, time, source|
        var shader = shaders[message[1].asInteger];
        if(dumpOSC) { [\SCShader, \recv, source, message].postln };
        if(message.size < 4 or: { message.size.odd }) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", message[1],
                "E_PROTOCOL", "malformed shader reflection reply"], time, source);
        } {
            shader.tryPerform(\receivedReflection, message.copyRange(2, message.size - 1));
        };
        ^this
    }

    receivedTimingMarker { |message, time, source|
        var id = message[1].asInteger;
        var callback = markerCallbacks.removeAt(id);
        if(message.size < 4) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed timing marker reply"], time, source);
        } {
            lastTiming = (
                sequence: id,
                scheduledRendererTime: message[2].asFloat,
                appliedRendererTime: message[3].asFloat
            );
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            callback.tryPerform(\value, this, lastTiming);
        };
        ^lastTiming
    }

    receivedWindowMetrics { |message, time, source|
        if(message.size < 13) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed window metrics reply"], time, source);
        } {
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            window.tryPerform(\receivedMetrics, (
                logicalWidth: message[1].asInteger,
                logicalHeight: message[2].asInteger,
                pixelWidth: message[3].asInteger,
                pixelHeight: message[4].asInteger,
                pixelRatio: message[5].asFloat,
                position: message[6].asInteger @ message[7].asInteger,
                fullscreen: message[8].asInteger == 1,
                borderless: message[9].asInteger == 1,
                vsync: message[10].asInteger == 1,
                cursorVisible: message[11].asInteger == 1,
                title: message[12].asString
            ));
        };
        ^this
    }

    receivedInputMouse { |message, time, source|
        if(message.size < 5) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed input mouse reply"], time, source);
        } {
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            window.tryPerform(\receivedMouse, message[1].asSymbol, message[2].asFloat,
                message[3].asFloat, message[4].asInteger);
        };
        ^this
    }

    receivedInputKey { |message, time, source|
        if(message.size < 4) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed input key reply"], time, source);
        } {
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            window.tryPerform(\receivedKey, message[1].asSymbol, message[2].asString,
                message[3].asInteger == 1);
        };
        ^this
    }

    receivedInputResize { |message, time, source|
        if(message.size < 6) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed input resize reply"], time, source);
        } {
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            window.tryPerform(\receivedResize, (
                logicalWidth: message[1].asInteger,
                logicalHeight: message[2].asInteger,
                pixelWidth: message[3].asInteger,
                pixelHeight: message[4].asInteger,
                pixelRatio: message[5].asFloat
            ));
        };
        ^this
    }

    receivedInputFocus { |message, time, source|
        if(message.size < 2) {
            this.receivedError(['/scshader/v1/error', "error", "protocol", 0,
                "E_PROTOCOL", "malformed input focus reply"], time, source);
        } {
            if(dumpOSC) { [\SCShader, \recv, source, message].postln };
            window.tryPerform(\receivedFocus, message[1].asInteger == 1);
        };
        ^this
    }

    retryHello { |onFailure|
        var attempt = bootStartedAt;
        SystemClock.sched(0.1, {
            if(isRunning or: { isBooting.not or: { bootStartedAt != attempt } }) {
                nil
            } {
                if((Main.elapsedTime - bootStartedAt) > 5.0) {
                    this.bootFailed("renderer did not complete its OSC handshake within 5 seconds", onFailure);
                    this.terminateChild;
                    nil
                } {
                    this.sendHello;
                    0.1
                }
            }
        });
        ^this
    }

    sendHello {
        this.sendMsg('/scshader/v1/hello', 1, NetAddr.langPort, "sclang");
        ^this
    }

    bootFailed { |message, onFailure|
        lastError = (severity: "error", subsystem: "lifecycle", resourceID: 0,
            code: "E_BOOT", message: message.asString);
        isBooting = false;
        isRunning = false;
        ("ShaderServer % failed to boot: %".format(name, message)).warn;
        bootCallbacks.do { |pair| pair[1].tryPerform(\value, this, lastError) };
        bootCallbacks.clear;
        onFailure.tryPerform(\value, this, lastError);
        ^this
    }

    processExited { |exitCode, processID|
        if(processID != pid) { ^this };
        pid = nil;
        if(quitting) {
            this.finishQuit;
        } {
            this.stopResponders;
            this.invalidateShaders;
            this.bootFailed((exitCode == 70).if({
                "renderer exited with GPU failure code 70 (fatal error or device recovery unavailable/exhausted)"
            }, { "renderer process exited with code %".format(exitCode) }));
        };
        ^this
    }

    finishQuit {
        var callbacks = quitCallbacks.copy;
        quitCallbacks.clear;
        isRunning = false;
        isBooting = false;
        quitting = false;
        pid = nil;
        markerCallbacks.clear;
        statusCallbacks.clear;
        this.stopResponders;
        this.invalidateShaders;
        callbacks.do { |callback| callback.value(this) };
        ^this
    }

    invalidateShaders {
        shaders.values.asArray.do { |shader| shader.receivedFreed };
        ^this
    }

    terminateChild {
        var child = pid;
        if(child.notNil) {
            if(thisProcess.platform.name == \windows) {
                ["taskkill", "/pid", child.asString, "/t", "/f"].unixCmd;
            } {
                ["kill", "-TERM", child.asString].unixCmd;
            };
        };
        ^this
    }

    responderKey { |suffix|
        ^("scshader_%_%_%".format(name, this.identityHash, suffix)).asSymbol
    }

    nextSequence {
        sequence = (sequence + 1).wrap(1, 2147483647);
        ^sequence
    }
}
