ShaderWaveformTexture : Object {
    classvar nextID;
    var <audioServer, <source, <points, <window, <rate, <shaderServer, <buffer, <synth, <isFreed;
    var responder, replyID, defName;

    *initClass { nextID = 7000 }

    *new { |server, source, points = 512, window = 0.05, rate = 30, shaderServer|
        ^super.new.init(server, source, points, window, rate, shaderServer ? ShaderServer.default)
    }

    init { |argAudioServer, argSource, argPoints, argWindow, argRate, argShaderServer|
        if(argPoints.isInteger.not or: { argPoints <= 1 or: { argPoints > 8192 } }) {
            Error("ShaderWaveformTexture points must be an integer from 2 through 8192.").throw;
        };
        if(argWindow.isNumber.not or: { argWindow <= 0 }) { Error("ShaderWaveformTexture window must be positive.").throw };
        if(argRate.isNumber.not or: { argRate <= 0 }) { Error("ShaderWaveformTexture rate must be positive.").throw };
        if(argShaderServer.isRunning.not) { Error("Boot ShaderServer before creating a ShaderWaveformTexture.").throw };
        audioServer = argAudioServer;
        source = argSource;
        points = argPoints;
        window = argWindow.asFloat;
        rate = argRate.asFloat;
        shaderServer = argShaderServer;
        buffer = ShaderBuffer.float(points, shaderServer);
        isFreed = false;
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(7000, 2147483647);
        ^nextID
    }

    start {
        var key;
        if(isFreed) { Error("Cannot start a freed ShaderWaveformTexture.").throw };
        if(synth.notNil) { ^this };
        if(audioServer.serverRunning.not) { Error("Boot the audio Server before starting ShaderWaveformTexture.").throw };
        if(source.isInteger.not or: { source < 0 }) {
            Error("ShaderWaveformTexture source must be a non-negative audio bus index.").throw;
        };
        replyID = this.class.nextID;
        defName = ("scshader_waveform_" ++ replyID).asSymbol;
        key = ("scshader_waveform_" ++ this.identityHash).asSymbol;
        responder = OSCdef(key, { |message|
            if(message.size == (buffer.length + 3) and: { message[2].asInteger == replyID }) {
                buffer.setn(0, message.copyRange(3, message.size - 1));
            };
        }, '/scshader/waveform', audioServer.addr);
        SynthDef(defName, { |inputBus = 0, replyRate = 30, captureWindow = 0.05|
            var input = In.ar(inputBus, 1);
            var trigger = Impulse.kr(replyRate);
            var samples = Array.fill(points, { |index|
                var delay = (index / (points - 1)) * captureWindow;
                A2K.kr(Latch.ar(DelayN.ar(input, captureWindow, delay), trigger));
            });
            SendReply.kr(trigger, '/scshader/waveform', samples, replyID);
        }).send(audioServer);
        audioServer.sync;
        synth = Synth.tail(audioServer.defaultGroup, defName,
            [\inputBus, source, \replyRate, rate, \captureWindow, window]);
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
