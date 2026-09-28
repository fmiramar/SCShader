(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var server = ShaderServer.new(\scshaderWindowSmoke, "127.0.0.1", 57152);
var window, completed = false, inputCount = 0, gotResize = false, gotFocus = false, finish;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        window.tryPerform(\free);
        server.quit({
            passed.if({ "SCSHADER_WINDOW_OK".postln; 0.exit }, {
                "SCSHADER_WINDOW_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(3.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

if(PathName(rendererPath).isFile.not) {
    finish.value(false, "SCSHADER_WINDOW_MISSING_RENDERER");
} {
    window = ShaderWindow.new(640, 360, "SCShader window smoke", server);
    window.position_(40 @ 60);
    window.inputEnabled_(true);
    window.mouseMoveAction_({ |receiver, point| inputCount = inputCount + 1; });
    window.mouseDownAction_({ |receiver, point, button| inputCount = inputCount + 1; });
    window.keyDownAction_({ |receiver, key, repeat| inputCount = inputCount + 1; });
    window.resizeAction_({ |receiver, logical, pixels, ratio| gotResize = true; });
    window.focusAction_({ |receiver, focused| gotFocus = focused; });
    server.rendererPath_(rendererPath);
    server.waitForBoot({ |shaderServer|
        SystemClock.sched(0.4, {
            window.title_("SCShader window smoke resized");
            window.borderless_(false);
            window.fullscreen_(false);
            window.vsync_(true);
            window.cursorVisible_(true);
            window.size_(800 @ 450);
            nil
        });
        SystemClock.sched(1.8, {
            // Exercise the responder-to-callback mapping without requiring a person to use the test window.
            shaderServer.receivedInputMouse(['/scshader/v1/input/mouse', "move", 0.25, 0.75, -1], nil, nil);
            shaderServer.receivedInputMouse(['/scshader/v1/input/mouse', "down", 0.25, 0.75, 0], nil, nil);
            shaderServer.receivedInputKey(['/scshader/v1/input/key', "down", "A", 0], nil, nil);
            shaderServer.receivedInputResize(['/scshader/v1/input/resize', 800, 450, 800, 450, 1.0], nil, nil);
            shaderServer.receivedInputFocus(['/scshader/v1/input/focus', 1], nil, nil);
            if(window.logicalSize.x > 0 and: { window.logicalSize.y > 0 }
                and: { window.pixelSize.x > 0 } and: { window.pixelSize.y > 0 }
                and: { window.title == "SCShader window smoke resized" }
                and: { window.inputEnabled } and: { inputCount == 3 }
                and: { gotResize } and: { gotFocus }) {
                finish.value(true, "SCSHADER_WINDOW_ROUNDTRIP");
            } {
                finish.value(false, "SCSHADER_WINDOW_BAD_STATE");
            };
            nil
        });
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_WINDOW_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(10.0, {
        finish.value(false, "SCSHADER_WINDOW_TIMEOUT");
        nil
    });
};
)
