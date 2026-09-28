Shader : Object {
    classvar nextID, all, byName;
    var <shaderDef, <server, <id, <isRunning, <isFreed, <lastError, <uniforms, <reloadCount;
    var pendingValues, getCallbacks, getSequence, reflectionReceived, creationAcknowledged;

    *initClass {
        nextID = 1000;
        all = IdentityDictionary.new;
        byName = IdentityDictionary.new;
        Class.initClassTree(Event);
        Event.addEventType(\shader, { |ignoredServer|
            Shader.playEvent(currentEnvironment);
        });
    }

    *new { |definition, args = #[], server|
        ^super.new.init(definition, args, server)
    }

    *at { |id| ^all[id] }

    *atName { |name| ^byName[name.asSymbol] }

    *playEvent { |event|
        var requested = event[\shader];
        var shaderID = event[\shaderID];
        var window = event[\window];
        var shader, definition, targetServer, latency, pairs = List.new;

        if(window.notNil) {
            if(window.isKindOf(ShaderWindow).not) {
                Error("A shader Event's window must be a ShaderWindow.").throw;
            };
            targetServer = window.server;
        };

        if(shaderID.notNil) {
            if(shaderID.isNumber.not) {
                Error("A shader Event's shaderID must be a numeric Shader ID.").throw;
            };
            shader = this.at(shaderID.asInteger);
            if(shader.isNil) { Error("Shader Event could not find shaderID %.".format(shaderID)).throw };
        } {
            if(requested.isNil) { Error("A shader Event needs a Shader, ShaderDef, or shader name.").throw };
            shader = requested.isKindOf(Shader).if({ requested }, { nil });
            if(shader.isNil) {
                definition = requested.isKindOf(ShaderDef).if({ requested }, { ShaderDef.at(requested) });
                if(definition.isNil) {
                    Error("Shader Event could not resolve shader %.".format(requested)).throw;
                };
                shader = this.atName(definition.name);
                if(shader.isNil or: { shader.isFreed }) {
                    targetServer = targetServer ? ShaderServer.default;
                    shader = this.new(definition, #[], targetServer);
                };
            };
        };

        if(shader.isFreed) { Error("Shader Event cannot use a freed Shader.").throw };
        targetServer = targetServer ? shader.server;
        if(shader.server !== targetServer) {
            Error("Shader Event window and Shader belong to different ShaderServers.").throw;
        };
        if(targetServer.isRunning.not) {
            Error("Boot ShaderServer before playing shader Events.").throw;
        };

        latency = event[\latency] ? targetServer.latency;
        if(latency.isNumber.not or: { latency.isNaN or: { latency.abs >= inf } or: { latency < 0 } }) {
            Error("A shader Event latency must be a finite non-negative number.").throw;
        };
        shader.uniforms.keysValuesDo { |uniform, type|
            var value = event[uniform];
            if(value.notNil) { pairs.add(uniform).add(value) };
        };
        if(pairs.notEmpty) { shader.setAt(latency.asFloat, *pairs) };
        event[\shaderID] = shader.id;
        event[\shaderObject] = shader;
        ^shader
    }

    init { |definition, args, argServer|
        shaderDef = definition.isKindOf(ShaderDef).if({ definition }, { ShaderDef.at(definition) });
        if(shaderDef.isNil) { Error("Shader requires a registered ShaderDef or its name.").throw };
        server = argServer ? ShaderServer.default;
        if(server.isRunning.not) { Error("Boot ShaderServer before creating a Shader.").throw };
        id = this.class.nextID;
        isRunning = false;
        isFreed = false;
        reloadCount = 0;
        uniforms = shaderDef.uniforms.copy;
        pendingValues = IdentityDictionary.new;
        getCallbacks = IdentityDictionary.new;
        getSequence = 0;
        reflectionReceived = false;
        creationAcknowledged = false;
        this.set(*args);
        all[id] = this;
        byName[shaderDef.name] = this;
        server.registerShader(this);
        server.sendMsg('/scshader/v1/shader/create', id, shaderDef.path, shaderDef.sourceType.asString);
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(1000, 2147483647);
        ^nextID
    }

    set { |... pairs|
        if(pairs.size.odd) { Error("Shader.set requires uniform/value pairs.").throw };
        pairs.pairsDo { |uniformName, value| this.validateValue(uniformName, value) };
        pairs.pairsDo { |uniformName, value| this.setOne(uniformName, value) };
        ^this
    }

    setAt { |time ... pairs|
        var messages = List.new;
        if(time.isNumber.not or: { time.isNaN or: { time.abs >= inf } or: { time < 0 } }) {
            Error("Shader.setAt time must be a finite non-negative number of seconds.").throw;
        };
        if(pairs.size.odd) { Error("Shader.setAt requires uniform/value pairs.").throw };
        pairs.pairsDo { |uniformName, value| this.validateValue(uniformName, value) };
        pairs.pairsDo { |uniformName, value|
            var uniform = this.validateAndStore(uniformName, value);
            messages.add(this.valueMessage(uniform, pendingValues[uniform]));
        };
        if(isRunning and: { messages.notEmpty }) { server.sendBundle(time, *messages) };
        ^this
    }

    map { |uniform, bus, channel = 0|
        if(bus.isKindOf(ShaderBus).not) { Error("Shader.map requires a ShaderBus.").throw };
        bus.map(this, uniform, channel);
        ^this
    }

    setTexture { |name, texture|
        if(isFreed) { Error("Cannot set a texture on a freed Shader.").throw };
        if(name.asSymbol == \spectrum and: { texture.isKindOf(ShaderFFTTexture) or: { texture.isKindOf(ShaderWaveformTexture) } }) {
            ^this.setBuffer(\spectrum, texture.buffer);
        };
        if(texture.isKindOf(ShaderTexture).not) {
            Error("Shader.setTexture requires a ShaderTexture, ShaderFFTTexture, or ShaderWaveformTexture.").throw;
        };
        if(texture.server !== server) {
            Error("Shader and ShaderTexture must belong to the same ShaderServer.").throw;
        };
        if(name.asSymbol != \source) {
            Error("The current texture ABI exposes only the \\source binding.").throw;
        };
        server.sendMsg('/scshader/v1/shader/texture', id, name.asString, texture.id);
        ^this
    }

    setBuffer { |name, buffer|
        if(isFreed) { Error("Cannot set a buffer on a freed Shader.").throw };
        if(buffer.isKindOf(ShaderBuffer).not) {
            Error("Shader.setBuffer requires a ShaderBuffer.").throw;
        };
        if(buffer.server !== server) {
            Error("Shader and ShaderBuffer must belong to the same ShaderServer.").throw;
        };
        if(name.asSymbol != \spectrum) {
            Error("The current buffer ABI exposes only the \\spectrum binding.").throw;
        };
        server.sendMsg('/scshader/v1/shader/buffer', id, name.asString, buffer.id);
        ^this
    }

    feedback { |source = \previous, amount = 0.98|
        if(isFreed) { Error("Cannot set feedback on a freed Shader.").throw };
        if(source.asSymbol != \previous and: { source.asSymbol != \framebuffer }) {
            Error("Shader.feedback source must be \\previous or \\framebuffer.").throw;
        };
        if(amount.isNumber.not or: { amount.isNaN or: { amount < 0 or: { amount > 1 } } }) {
            Error("Shader.feedback amount must be between 0 and 1.").throw;
        };
        server.sendMsg('/scshader/v1/shader/feedback', id, source.asString, amount.asFloat);
        ^this
    }

    setOne { |uniformName, value|
        var uniform = this.validateAndStore(uniformName, value);
        if(isRunning) { server.sendMsg(*this.valueMessage(uniform, pendingValues[uniform])) };
        ^this
    }

    setn { |uniformName, values| ^this.set(uniformName, values) }

    glide { |uniformName, value, duration = 0.1, mode = \linear|
        var uniform = uniformName.asSymbol, validated;
        if(isRunning.not) { Error("Wait for Shader creation before starting a glide.").throw };
        if(duration.isNumber.not or: { duration.isNaN or: { duration.abs > 3.4028234663852886e38 or: { duration < 0 } } }) {
            Error("Shader.glide duration must be a finite non-negative Float32 number of seconds.").throw;
        };
        if([\step, \linear, \smooth].includes(mode).not) { Error("Shader.glide mode must be step, linear, or smooth.").throw };
        if(mode != \step and: { [\int, \uint, \bool].includes(uniforms[uniform]) }) {
            Error("Integer and boolean Shader uniforms support only step interpolation.").throw;
        };
        validated = this.validateValue(uniform, value);
        pendingValues[uniform] = validated;
        server.sendMsg('/scshader/v1/uniform/glide', id, uniform.asString, uniforms[uniform].asString,
            ShaderDef.encodeValue(uniforms[uniform], validated), duration.asFloat, mode.asString);
        ^this
    }

    get { |uniformName, action|
        var uniform = uniformName.asSymbol, requestID;
        if(isRunning.not or: { isFreed }) { Error("Shader.get requires a live Shader.").throw };
        if(uniforms[uniform].isNil) { Error("Shader.get requires a reflected uniform.").throw };
        if(action.isKindOf(Function).not) { Error("Shader.get requires an asynchronous callback.").throw };
        getSequence = (getSequence + 1).wrap(1, 2147483647);
        requestID = getSequence;
        getCallbacks[requestID] = [uniform, action];
        server.sendMsg('/scshader/v1/uniform/get', id, uniform.asString, requestID);
        SystemClock.sched(2, {
            var pending = getCallbacks.removeAt(requestID);
            if(pending.notNil) {
                lastError = (code: "E_TIMEOUT", message: "Shader.get timed out for %".format(uniform));
                pending[1].value(nil);
            };
            nil
        });
        ^this
    }

    getn { |uniformName, action| ^this.get(uniformName, action) }

    value { |uniformName|
        ^pendingValues[uniformName.asSymbol].copy
    }

    validateAndStore { |uniformName, value|
        var uniform = uniformName.asSymbol;
        pendingValues[uniform] = this.validateValue(uniform, value);
        ^uniform
    }

    validateValue { |uniformName, value|
        var uniform = uniformName.asSymbol;
        var type = uniforms[uniform];
        if(isFreed) { Error("Cannot set a freed Shader.").throw };
        if(type.isNil) {
            Error("Shader % uniform % is not present in its reflection.".format(shaderDef.name, uniform)).throw;
        };
        ^ShaderDef.validateValue(type, value)
    }

    reload {
        if(isFreed) { Error("Cannot reload a freed Shader.").throw };
        server.sendMsg('/scshader/v1/shader/reload', id);
        ^this
    }

    free {
        if(isFreed.not) {
            server.sendMsg('/scshader/v1/shader/free', id);
        };
        ^this
    }

    receivedCreated {
        creationAcknowledged = true;
        this.finishCreation;
        ^this
    }

    finishCreation {
        if(isRunning.not and: { creationAcknowledged and: { reflectionReceived } }) {
            isRunning = true;
            pendingValues.keysValuesDo { |uniform, value| server.sendMsg(*this.valueMessage(uniform, value)) };
        };
        ^this
    }

    receivedReloaded {
        isRunning = true;
        reloadCount = reloadCount + 1;
        ^this
    }

    receivedFreed {
        isRunning = false;
        isFreed = true;
        getCallbacks.clear;
        server.unregisterShader(this);
        all.removeAt(id);
        if(byName[shaderDef.name] === this) { byName.removeAt(shaderDef.name) };
        ^this
    }

    receivedReflection { |pairs|
        var previous = uniforms;
        uniforms = Event.new;
        pairs.pairsDo { |uniformName, type| uniforms[uniformName.asSymbol] = type.asSymbol };
        pendingValues.keys.asArray.do { |uniform|
            if(uniforms[uniform] != previous[uniform]) {
                pendingValues.removeAt(uniform);
                lastError = (code: "E_REFLECTION_CHANGED", message: "Uniform % was removed or changed type; its pending value was discarded.".format(uniform));
                lastError[\message].warn;
            };
        };
        reflectionReceived = true;
        this.finishCreation;
        ^this
    }

    receivedValue { |uniform, requestID, type, bytes|
        var pending = getCallbacks[requestID];
        if(pending.notNil and: { pending[0] == uniform }) {
            getCallbacks.removeAt(requestID);
            pending[1].value(ShaderDef.decodeValue(type, bytes));
        };
        ^this
    }

    receivedError { |error|
        lastError = error;
        if(error[\code] == "E_RESOURCE_NOT_FOUND") {
            isRunning = false;
            isFreed = true;
            server.unregisterShader(this);
            all.removeAt(id);
            if(byName[shaderDef.name] === this) { byName.removeAt(shaderDef.name) };
        };
        ^this
    }

    valueMessage { |uniform, value|
        if(uniforms[uniform] == \float) { ^['/scshader/v1/uniform/f', id, uniform.asString, value] };
        ^['/scshader/v1/uniform/set', id, uniform.asString, uniforms[uniform].asString,
            ShaderDef.encodeValue(uniforms[uniform], value)]
    }
}
