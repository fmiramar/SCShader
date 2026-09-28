ShaderAnalysis : Object {
    classvar nextID;
    var <server, <source, <analyses, <rate, <buses, <synth;
    var responder, replyID, defName;

    *initClass { nextID = 3000 }

    *new { |server, source, analyses = #[\amplitude], rate = 30, smoothing = 0.0|
        ^super.new.init(server, source, analyses, rate, smoothing)
    }

    init { |argServer, argSource, argAnalyses, argRate, smoothing|
        if(argRate.isNumber.not or: { argRate <= 0 }) { Error("ShaderAnalysis rate must be positive.").throw };
        server = argServer;
        source = argSource;
        analyses = argAnalyses.asArray.collect(_.asSymbol);
        rate = argRate.asFloat;
        buses = IdentityDictionary.new;
        analyses.do { |analysis|
            if([\amplitude, \pitch, \confidence, \centroid, \flatness, \onset, \zeroCrossing].includes(analysis).not) {
                Error("Unsupported ShaderAnalysis type: %".format(analysis)).throw;
            };
            buses[analysis] = ShaderBus.control(1, smoothing);
        };
        ^this
    }

    at { |analysis| ^buses[analysis.asSymbol] }

    set { |analysis, value|
        var bus = this.at(analysis);
        if(bus.isNil) { Error("ShaderAnalysis does not provide %.".format(analysis)).throw };
        bus.set(value);
        ^this
    }

    flush { |latency|
        buses.values.do { |bus| bus.flush(latency) };
        ^this
    }

    start {
        var key;
        if(synth.notNil) { ^this };
        if(server.serverRunning.not) { Error("Boot the audio Server before starting ShaderAnalysis.").throw };
        if(source.isInteger.not or: { source < 0 }) { Error("ShaderAnalysis source must be a non-negative audio bus index.").throw };
        replyID = this.class.nextID;
        defName = ("scshader_analysis_" ++ replyID).asSymbol;
        key = ("scshader_analysis_" ++ this.identityHash).asSymbol;
        responder = OSCdef(key, { |message|
            if(message.size >= 10 and: { message[2].asInteger == replyID }) {
                this.received(message.copyRange(3, 9));
            };
        }, '/scshader/analysis', server.addr);
        SynthDef(defName, { |inputBus = 0, replyRate = 30|
            var input = In.ar(inputBus, 1);
            var chain = FFT(LocalBuf(2048), input);
            var pitch = Pitch.kr(input);
            var zeroCrossing = A2K.kr(ZeroCrossing.ar(input));
            SendReply.kr(Impulse.kr(replyRate), '/scshader/analysis', [
                Amplitude.kr(input), pitch[0], pitch[1], SpecCentroid.kr(chain),
                SpecFlatness.kr(chain), Onsets.kr(chain), zeroCrossing
            ], replyID);
        }).send(server);
        server.sync;
        // Reading at the default group's tail lets ordinary source Synths write
        // their audio buses first in each control block.
        synth = Synth.tail(server.defaultGroup, defName, [\inputBus, source, \replyRate, rate]);
        ^this
    }

    received { |values|
        var names = [\amplitude, \pitch, \confidence, \centroid, \flatness, \onset, \zeroCrossing];
        names.do { |name, index|
            var bus = buses[name];
            if(bus.notNil) { bus.set(values[index].asFloat) };
        };
        this.flush;
        ^this
    }

    free {
        responder.tryPerform(\free);
        responder = nil;
        synth.tryPerform(\free);
        synth = nil;
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(3000, 2147483647);
        ^nextID
    }
}
