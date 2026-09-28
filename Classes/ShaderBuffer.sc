ShaderBuffer : Object {
    classvar nextID, all;
    var <length, <server, <id, <isFreed, values;

    *initClass {
        nextID = 4000;
        all = IdentityDictionary.new;
    }

    *float { |length, server|
        ^this.new(length, server ? ShaderServer.default)
    }

    *new { |length, server|
        ^super.new.init(length, server ? ShaderServer.default)
    }

    *at { |id| ^all[id] }

    init { |argLength, argServer|
        if(argLength.isInteger.not or: { argLength <= 0 or: { argLength > 16384 } }) {
            Error("ShaderBuffer length must be an integer from 1 through 16384.").throw;
        };
        if(argServer.isRunning.not) { Error("Boot ShaderServer before creating a ShaderBuffer.").throw };
        length = argLength;
        server = argServer;
        id = this.class.nextID;
        values = FloatArray.newClear(length);
        isFreed = false;
        all[id] = this;
        server.sendMsg('/scshader/v1/buffer/create', id, length);
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(4000, 2147483647);
        ^nextID
    }

    value { |index|
        ^values[this.index(index)]
    }

    asArray { ^values.copy }

    setn { |start = 0, array|
        var data = array.asArray;
        if(isFreed) { Error("Cannot write a freed ShaderBuffer.").throw };
        if(data.isEmpty) { ^this };
        if(start.isInteger.not or: { start < 0 or: { (start + data.size) > length } }) {
            Error("ShaderBuffer write is outside 0 through %.".format(length - 1)).throw;
        };
        data.do { |value, offset|
            if(value.isNumber.not or: { value.isNaN or: { value.abs >= inf } }) {
                Error("ShaderBuffer values must be finite numbers.").throw;
            };
            values[start + offset] = value.asFloat;
        };
        this.sendChunks(start, data);
        ^this
    }

    set { |index, value|
        ^this.setn(index, [value])
    }

    sendChunks { |start, data|
        // Keep each datagram below the usual macOS 9216-byte UDP ceiling.
        var chunkSize = 2048;
        data.clump(chunkSize).do { |chunk, chunkIndex|
            server.sendMsg('/scshader/v1/buffer/write', id, start + (chunkIndex * chunkSize),
                this.floatBlob(chunk));
        };
        ^this
    }

    floatBlob { |array|
        var bytes = Int8Array.newClear(array.size * 4);
        array.do { |value, index|
            var bits = value.asFloat.as32Bits;
            var offset = index * 4;
            bytes[offset] = (bits >> 24).bitAnd(255);
            bytes[offset + 1] = (bits >> 16).bitAnd(255);
            bytes[offset + 2] = (bits >> 8).bitAnd(255);
            bytes[offset + 3] = bits.bitAnd(255);
        };
        ^bytes
    }

    free {
        if(isFreed.not) {
            isFreed = true;
            all.removeAt(id);
            server.sendMsg('/scshader/v1/buffer/free', id);
        };
        ^this
    }

    index { |index|
        if(index.isInteger.not or: { index < 0 or: { index >= length } }) {
            Error("ShaderBuffer index % is outside 0 through %.".format(index, length - 1)).throw;
        };
        ^index
    }
}
