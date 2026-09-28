ShaderTexture : Object {
    classvar nextID, all;
    var <path, <server, <id, <isFreed;

    *initClass {
        nextID = 2000;
        all = IdentityDictionary.new;
    }

    *new { |path, server|
        ^super.new.init(path, server ? ShaderServer.default)
    }

    *at { |id| ^all[id] }

    init { |argPath, argServer|
        path = argPath.asString.standardizePath;
        server = argServer;
        if(PathName(path).isFile.not) { Error("ShaderTexture image file does not exist: %".format(path)).throw };
        if(server.isRunning.not) { Error("Boot ShaderServer before creating a ShaderTexture.").throw };
        id = this.class.nextID;
        isFreed = false;
        all[id] = this;
        server.sendMsg('/scshader/v1/texture/create', id, path);
        ^this
    }

    *nextID {
        nextID = (nextID + 1).wrap(2000, 2147483647);
        ^nextID
    }

    free {
        if(isFreed.not) {
            isFreed = true;
            all.removeAt(id);
            server.sendMsg('/scshader/v1/texture/free', id);
        };
        ^this
    }
}
