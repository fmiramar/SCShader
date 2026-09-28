ShaderWindow : Object {
    var <server;
    var width, height, title, position, fullscreen, borderless, monitor, vsync, cursorVisible, inputEnabled;
    var logicalWidth, logicalHeight, pixelWidth, pixelHeight, pixelRatio;
    var <>mouseMoveAction, <>mouseDownAction, <>mouseUpAction, <>keyDownAction, <>keyUpAction;
    var <>resizeAction, <>focusAction;

    *new { |width = 1280, height = 720, title = "SCShader", server|
        ^super.new.init(width, height, title, server ? ShaderServer.default)
    }

    init { |argWidth, argHeight, argTitle, argServer|
        server = argServer;
        if(server.isKindOf(ShaderServer).not) {
            Error("ShaderWindow server must be a ShaderServer.").throw;
        };
        width = this.positiveInteger(argWidth, "width");
        height = this.positiveInteger(argHeight, "height");
        title = this.nonEmptyString(argTitle, "title");
        position = nil;
        fullscreen = false;
        borderless = false;
        monitor = nil;
        vsync = true;
        cursorVisible = true;
        inputEnabled = false;
        logicalWidth = width;
        logicalHeight = height;
        pixelWidth = width;
        pixelHeight = height;
        pixelRatio = 1.0;
        server.registerWindow(this);
        ^this
    }

    rendererArgs {
        var args = ["--width", width.asString, "--height", height.asString, "--title", title];
        if(position.notNil) {
            args = args ++ ["--position", position.x.asString, position.y.asString];
        };
        if(fullscreen) { args = args ++ ["--fullscreen"] };
        if(borderless) { args = args ++ ["--borderless"] };
        if(monitor.notNil) { args = args ++ ["--monitor", monitor.asString] };
        if(vsync.not) { args = args ++ ["--no-vsync"] };
        if(cursorVisible.not) { args = args ++ ["--hide-cursor"] };
        ^args
    }

    width { ^width }

    height { ^height }

    size { ^width @ height }

    logicalSize { ^logicalWidth @ logicalHeight }

    pixelSize { ^pixelWidth @ pixelHeight }

    pixelRatio { ^pixelRatio }

    title { ^title }

    position { ^position }

    fullscreen { ^fullscreen }

    borderless { ^borderless }

    monitor { ^monitor }

    vsync { ^vsync }

    cursorVisible { ^cursorVisible }

    inputEnabled { ^inputEnabled }

    size_ { |value|
        var point = this.integerPoint(value, "size");
        width = this.positiveInteger(point.x, "width");
        height = this.positiveInteger(point.y, "height");
        this.sendWhenRunning('/scshader/v1/window/resize', width, height);
        ^this
    }

    width_ { |value|
        ^this.size_(this.positiveInteger(value, "width") @ height)
    }

    height_ { |value|
        ^this.size_(width @ this.positiveInteger(value, "height"))
    }

    title_ { |value|
        title = this.nonEmptyString(value, "title");
        this.sendWhenRunning('/scshader/v1/window/title', title);
        ^this
    }

    position_ { |value|
        position = this.integerPoint(value, "position");
        this.sendWhenRunning('/scshader/v1/window/position', position.x, position.y);
        ^this
    }

    fullscreen_ { |value|
        fullscreen = this.boolean(value, "fullscreen");
        this.sendWhenRunning('/scshader/v1/window/fullscreen', fullscreen.binaryValue);
        ^this
    }

    borderless_ { |value|
        borderless = this.boolean(value, "borderless");
        this.sendWhenRunning('/scshader/v1/window/borderless', borderless.binaryValue);
        ^this
    }

    monitor_ { |value|
        if(server.isBooting or: { server.isRunning }) {
            Error("Set ShaderWindow.monitor before booting the renderer.").throw;
        };
        monitor = this.nonnegativeInteger(value, "monitor");
        ^this
    }

    vsync_ { |value|
        vsync = this.boolean(value, "vsync");
        this.sendWhenRunning('/scshader/v1/window/vsync', vsync.binaryValue);
        ^this
    }

    cursorVisible_ { |value|
        cursorVisible = this.boolean(value, "cursorVisible");
        this.sendWhenRunning('/scshader/v1/window/cursor-visible', cursorVisible.binaryValue);
        ^this
    }

    inputEnabled_ { |value|
        inputEnabled = this.boolean(value, "inputEnabled");
        this.sendWhenRunning('/scshader/v1/window/input-enabled', inputEnabled.binaryValue);
        ^this
    }

    front {
        this.sendWhenRunning('/scshader/v1/window/front');
        ^this
    }

    close {
        this.sendWhenRunning('/scshader/v1/window/close');
        ^this
    }

    requestMetrics {
        this.sendWhenRunning('/scshader/v1/window/metrics');
        ^this
    }

    free {
        server.unregisterWindow(this);
        ^this
    }

    receivedReady {
        this.applyRuntimeState;
        ^this
    }

    receivedMetrics { |metrics|
        logicalWidth = metrics[\logicalWidth];
        logicalHeight = metrics[\logicalHeight];
        pixelWidth = metrics[\pixelWidth];
        pixelHeight = metrics[\pixelHeight];
        pixelRatio = metrics[\pixelRatio];
        position = metrics[\position];
        fullscreen = metrics[\fullscreen];
        borderless = metrics[\borderless];
        vsync = metrics[\vsync];
        cursorVisible = metrics[\cursorVisible];
        title = metrics[\title];
        ^this
    }

    receivedMouse { |kind, x, y, button|
        var point = x @ y;
        switch(kind,
            \move, { mouseMoveAction.tryPerform(\value, this, point) },
            \down, { mouseDownAction.tryPerform(\value, this, point, button) },
            \up, { mouseUpAction.tryPerform(\value, this, point, button) }
        );
        ^this
    }

    receivedKey { |state, key, repeat|
        switch(state,
            \down, { keyDownAction.tryPerform(\value, this, key, repeat) },
            \up, { keyUpAction.tryPerform(\value, this, key, repeat) }
        );
        ^this
    }

    receivedResize { |metrics|
        logicalWidth = metrics[\logicalWidth];
        logicalHeight = metrics[\logicalHeight];
        pixelWidth = metrics[\pixelWidth];
        pixelHeight = metrics[\pixelHeight];
        pixelRatio = metrics[\pixelRatio];
        resizeAction.tryPerform(\value, this, this.logicalSize, this.pixelSize, pixelRatio);
        ^this
    }

    receivedFocus { |focused|
        focusAction.tryPerform(\value, this, focused);
        ^this
    }

    applyRuntimeState {
        this.sendWhenRunning('/scshader/v1/window/resize', width, height);
        this.sendWhenRunning('/scshader/v1/window/title', title);
        if(position.notNil) {
            this.sendWhenRunning('/scshader/v1/window/position', position.x, position.y);
        };
        this.sendWhenRunning('/scshader/v1/window/fullscreen', fullscreen.binaryValue);
        this.sendWhenRunning('/scshader/v1/window/borderless', borderless.binaryValue);
        this.sendWhenRunning('/scshader/v1/window/vsync', vsync.binaryValue);
        this.sendWhenRunning('/scshader/v1/window/cursor-visible', cursorVisible.binaryValue);
        this.sendWhenRunning('/scshader/v1/window/input-enabled', inputEnabled.binaryValue);
        this.requestMetrics;
        ^this
    }

    sendWhenRunning { |... args|
        if(server.isRunning) { server.sendMsg(*args) };
        ^this
    }

    integerPoint { |value, name|
        if(value.isKindOf(Point).not) {
            Error("ShaderWindow % must be a Point.".format(name)).throw;
        };
        this.integer(value.x, name ++ ".x");
        this.integer(value.y, name ++ ".y");
        ^value.x.asInteger @ value.y.asInteger
    }

    positiveInteger { |value, name|
        var integer = this.integer(value, name);
        if(integer <= 0) { Error("ShaderWindow % must be positive.".format(name)).throw };
        ^integer
    }

    nonnegativeInteger { |value, name|
        var integer = this.integer(value, name);
        if(integer < 0) { Error("ShaderWindow % must not be negative.".format(name)).throw };
        ^integer
    }

    integer { |value, name|
        if(value.isNumber.not or: { value.asFloat != value.asInteger }) {
            Error("ShaderWindow % must be an integer.".format(name)).throw;
        };
        ^value.asInteger
    }

    boolean { |value, name|
        if((value == true).not and: { (value == false).not }) {
            Error("ShaderWindow % must be true or false.".format(name)).throw;
        };
        ^value
    }

    nonEmptyString { |value, name|
        var string = value.asString;
        if(string.isEmpty) { Error("ShaderWindow % must not be empty.".format(name)).throw };
        ^string
    }
}
