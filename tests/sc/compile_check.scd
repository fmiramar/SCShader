(
var server, window, args, bus;
server = ShaderServer.new(\scshaderCompileCheck, "127.0.0.1", 57149);
// Platform.name is the class name, not the operating system instance's name.
if(thisProcess.platform.name == \windows and: {
    ShaderServer.rendererExecutableName != "scshader-renderer.exe"
}) { Error("Windows renderer discovery must use the .exe suffix").throw };
if(server.addr.port != 57149) { Error("ShaderServer did not retain its OSC address").throw };
if(server.isRunning or: { server.isBooting }) { Error("new ShaderServer must be stopped").throw };
if(server.latency != 0.02) { Error("ShaderServer default latency changed").throw };
if(Event.eventTypes[\shader].isNil) { Error("SCShader did not register the shader Event type").throw };
bus = ShaderBus.control(2, 0.5);
bus.setn(0, [0.4, 0.8]);
bus.flush;
if((bus.value(0) - 0.2).abs > 0.0001 or: { (bus.value(1) - 0.4).abs > 0.0001 }) {
    Error("ShaderBus smoothing did not apply one control-rate step").throw;
};
bus.free;
server.rendererArgs_(["--unused-test-argument"]);
window = ShaderWindow.new(640, 360, "SCShader compile check", server);
window.position_(20 @ 30);
window.monitor_(0);
window.borderless_(true);
window.vsync_(false);
window.cursorVisible_(false);
args = window.rendererArgs;
if(args[0].asString != "--width" or: { args[1].asInteger != 640 }
    or: { args[4].asString != "--title" } or: { args[6].asString != "--position" }
    or: { args[10].asString != "--monitor" } or: { args[12].asString != "--no-vsync" }
    or: { args[13].asString != "--hide-cursor" }) {
    Error("ShaderWindow did not construct the required renderer arguments").throw;
};
window.free;
server.free;
"SCSHADER_CLASSES_COMPILE_OK".postln;
0.exit;
)
