// Start the Rust renderer first. This example needs neither scsynth nor audio input.

(
~scShader = NetAddr("127.0.0.1", 57140);

OSCdef(\scShaderReady, { |message|
    "SCShader ready: %".format(message).postln;
}, '/scshader/v1/ready');

OSCdef(\scShaderPong, { |message|
    "SCShader pong: %".format(message).postln;
}, '/scshader/v1/pong');

OSCdef(\scShaderError, { |message|
    "SCShader diagnostic: %".format(message).warn;
}, '/scshader/v1/error');

~scShader.sendMsg(
    '/scshader/v1/hello',
    1,
    NetAddr.langPort,
    "sclang-feasibility-example"
);
)

// Drag the control: resource 1 exposes one float named "amount".
(
~scShaderControl = Window("SCShader amount", Rect(200, 200, 420, 90)).front;
~scShaderSlider = Slider(~scShaderControl, Rect(20, 20, 380, 40))
    .value_(0.5)
    .action_({ |slider|
        ~scShader.sendMsg('/scshader/v1/uniform/f', 1, "amount", slider.value);
    });
)

// Optional ping.
(
~scShader.sendMsg('/scshader/v1/ping', 1, Main.elapsedTime);
)

// Clean up the example and terminate only the renderer process.
(
~scShader.sendMsg('/scshader/v1/quit');
~scShaderControl.tryPerform(\close);
OSCdef(\scShaderReady).free;
OSCdef(\scShaderPong).free;
OSCdef(\scShaderError).free;
)

