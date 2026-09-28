ShaderFFTTexture : Object {
    classvar nextID;
    var <audioServer, <source, <fftSize, <rate, <shaderServer, <buffer, <synth, <isFreed;
    var responder, replyID, defName;

    *initClass { nextID = 5000 }

    *new { |server, source, fftSize = 2048, rate = 30, shaderServer|
        ^super.new.init(server, source, fftSize, rate, shaderServer ? ShaderServer.default)
    }

    init { |argAudioServer, argSource, argFFTSize, argRate, argShaderServer|
        if([64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384].includes(argFFTSize).not) {
            Error("ShaderFFTTexture fftSize must be a power of two from 64 through 16384.").throw;
        };
        if(argRate.isNumber.not or: { argRate <= 0 }) { Error("ShaderFFTTexture rate must be positive.").throw };
        if(argShaderServer.isRunning.not) { Error("Boot ShaderServer before creating a ShaderFFTTexture.").throw };
        audioServer = argAudioServer;
        source = argSource;
        fftSize = argFFTSize;
        rate = argRate.asFloat;
        shaderServer = argShaderServer;
        buffer = ShaderBuffer.float(fftSize div: 2, shaderServer);
        isFreed = false;
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(5000, 2147483647);
        ^nextID
    }

    start {
        var key;
        if(isFreed) { Error("Cannot start a freed ShaderFFTTexture.").throw };
        if(synth.notNil) { ^this };
        if(audioServer.serverRunning.not) { Error("Boot the audio Server before starting ShaderFFTTexture.").throw };
        if(source.isInteger.not or: { source < 0 }) {
            Error("ShaderFFTTexture source must be a non-negative audio bus index.").throw;
        };
        replyID = this.class.nextID;
        defName = ("scshader_fft_" ++ replyID).asSymbol;
        key = ("scshader_fft_" ++ this.identityHash).asSymbol;
        responder = OSCdef(key, { |message|
            if(message.size == (buffer.length + 3) and: { message[2].asInteger == replyID }) {
                buffer.setn(0, message.copyRange(3, message.size - 1));
            };
        }, '/scshader/fft', audioServer.addr);
        SynthDef(defName, { |inputBus = 0, replyRate = 30|
            var input = In.ar(inputBus, 1);
            var chain = FFT(LocalBuf(fftSize), input);
            var windowStarts = chain > -1;
            var divider = (SampleRate.ir / ((fftSize * 0.5) * replyRate)).round.max(1);
            var trigger = PulseDivider.kr(windowStarts, divider);
            var streams = Array.fill(fftSize div: 2, { |index|
                Unpack1FFT(chain, fftSize, index, 0);
            });
            // Demand.kr currently accepts at most 32 inputs, so unpack all bins
            // in small groups before combining them into one SendReply packet.
            var magnitudes = streams.clump(32).collect { |group|
                Demand.kr(trigger, 0, group);
            }.flat;
            SendReply.kr(trigger, '/scshader/fft', magnitudes, replyID);
        }).send(audioServer);
        audioServer.sync;
        synth = Synth.tail(audioServer.defaultGroup, defName, [\inputBus, source, \replyRate, rate]);
        ^this
    }

    free {
        if(isFreed.not) {
            isFreed = true;
            responder.tryPerform(\free);
            responder = nil;
            synth.tryPerform(\free);
            synth = nil;
            buffer.free;
        };
        ^this
    }
}
