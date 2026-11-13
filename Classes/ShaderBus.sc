ShaderBus : Object {
    var hydraTargets;
    var <channels, <smoothing, values, desiredValues, targets, pending, scheduled, isFreed;

    *control { |channels = 1, smoothing = 0.0|
        ^this.new(channels, smoothing)
    }

    *new { |channels = 1, smoothing = 0.0|
        ^super.new.init(channels, smoothing)
    }

    init { |argChannels, argSmoothing|
        if(argChannels.isInteger.not or: { argChannels <= 0 }) {
            Error("ShaderBus channels must be a positive integer.").throw;
        };
        if(argSmoothing.isNumber.not or: { argSmoothing < 0 or: { argSmoothing >= 1 } }) {
            Error("ShaderBus smoothing must be at least 0 and less than 1.").throw;
        };
        channels = argChannels;
        smoothing = argSmoothing.asFloat;
        values = Array.fill(channels, 0.0);
        desiredValues = Array.fill(channels, 0.0);
        targets = List.new;hydraTargets=List.new;
        pending = false;
        scheduled = false;
        isFreed = false;
        ^this
    }

    value { |channel = 0| ^values[this.channelIndex(channel)] }

    set { |value, channel = 0|
        var index = this.channelIndex(channel);
        if(value.isNumber.not or: { value.isNaN or: { value.abs >= inf } }) {
            Error("ShaderBus values must be finite numbers.").throw;
        };
        desiredValues[index] = value.asFloat;
        pending = true;
        this.scheduleFlush(0.0);
        ^this
    }

    setn { |start = 0, array|
        array.asArray.do { |value, offset| this.set(value, start + offset) };
        ^this
    }

    map { |shader, uniform, channel = 0|
        var index;
        if(shader.isKindOf(Shader).not) { Error("ShaderBus.map requires a Shader.").throw };
        if(isFreed) { Error("Cannot map a freed ShaderBus.").throw };
        index = this.channelIndex(channel);
        shader.validateAndStore(uniform, this.value(index));
        targets = targets.reject { |target| target[0] === shader and: { target[1] == uniform.asSymbol } };
        targets.add([shader, uniform.asSymbol, index]);
        pending = true;
        this.scheduleFlush(0.0);
        ^this
    }

    hydraMappingCount { |hydra| ^hydraTargets.count {|t|t[0]===hydra} }

    mapHydra { |hydra,parameter,channel=0,scale=1,bias=0,cutoff=0,output=\o0|
        if(isFreed) {Error("Cannot map a freed ShaderBus").throw};
        this.channelIndex(channel);
        [scale,bias,cutoff].do {|v|ShaderDef.validateValue(\float,v);if(v.abs>1e6){Error("Hydra mapping exceeds bound").throw}};
        hydra.parameterMessage(parameter,bias.clip(-1e6,1e6),output); // Validate committed handle.
        this.unmapHydra(hydra,parameter,output);
        if(hydraTargets.size>=64){Error("ShaderBus Hydra target limit is 64").throw};
        hydraTargets.add([hydra,parameter,channel,scale,bias,cutoff,output,nil,0.0]);
        pending=true;this.scheduleFlush(0);^this
    }
    unmapHydra { |hydra,parameter,output|
        hydraTargets=hydraTargets.reject {|t|t[0]===hydra and:{parameter.isNil or:{t[1]==parameter and:{output.isNil or:{t[6]==output}}}}};^this
    }

    unmap { |shader, uniform|
        targets = targets.reject { |target| target[0] === shader and: { target[1] == uniform.asSymbol } };
        ^this
    }

    flush { |latency|
        var groups = IdentityDictionary.new;
        if(pending.not) { ^this };
        channels.do { |index|
            values[index] = values[index] + ((desiredValues[index] - values[index]) * (1.0 - smoothing));
        };
        targets.do { |target|
            var shader = target[0], uniform = target[1], value = values[target[2]];
            var pairs = groups[shader] ? List.new;
            groups[shader] = pairs.add(uniform).add(value);
        };
        groups.keysValuesDo { |shader, pairs|
            if(shader.isFreed.not) { shader.setAt(latency ? shader.server.latency, *pairs) };
        };
        hydraTargets=hydraTargets.reject {|t|t[0].isFreed or:{t[0].server.isRunning.not}};
        hydraTargets.do {|t|
            var value=((values[t[2]]-t[5]).max(0)*t[3]+t[4]).clip(-1e6,1e6),now=Main.elapsedTime;
            if(t[7].isNil or:{(value-t[7]).abs>0.000001}) {
                if(t[0].canStream and:{(now-t[8]) >= (1/30)}) {
                    t[0].server.sendMsg(*t[0].parameterMessage(t[1],value,t[6]));
                    t[7]=value;t[8]=now;
                };
            };
        };
        pending = values.any { |value, index| (desiredValues[index] - value).abs > 0.000001 };
        pending=pending or:{hydraTargets.any {|t|
            var value=((values[t[2]]-t[5]).max(0)*t[3]+t[4]).clip(-1e6,1e6);
            t[7].isNil or:{(value-t[7]).abs>0.000001}
        }};
        if(pending) { this.scheduleFlush(1.0 / 60.0) };
        ^this
    }

    scheduleFlush { |delay|
        if(scheduled.not and: { isFreed.not }) {
            scheduled = true;
            SystemClock.sched(delay, {
                scheduled = false;
                this.flush;
                nil
            });
        };
        ^this
    }

    free {
        isFreed = true;
        targets.clear;hydraTargets.clear;
        pending = false;
        ^this
    }

    channelIndex { |channel|
        if(channel.isInteger.not or: { channel < 0 or: { channel >= channels } }) {
            Error("ShaderBus channel % is outside 0 through %.".format(channel, channels - 1)).throw;
        };
        ^channel
    }
}
