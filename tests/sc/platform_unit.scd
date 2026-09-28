(
var server = ShaderServer.new(\platformUnit);
var expected = (thisProcess.platform.name == \windows).if({ "scshader-renderer.exe" }, { "scshader-renderer" });
var packaged = server.packagedRendererPath;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
assert.value(ShaderServer.rendererExecutableName == expected, "Wrong platform executable suffix");
assert.value(packaged.notNil and: { PathName(packaged).isFile }, "Installed renderer was not discovered");
server.rendererArgs_(["--title", "SCShader path with spaces", "--backend", "auto"]);
assert.value(server.rendererCommand[2] == "SCShader path with spaces", "Arguments were shell-joined");
assert.value(server.rendererCommand.first == packaged, "Wrong discovery order");
server.rendererPath_(packaged ++ ".does-not-exist");
assert.value(try { server.resolvedRendererPath; false } { true }, "Explicit missing path fell back silently");
server.free;
"SCSHADER_PLATFORM_UNIT_OK".postln;
0.exit;
)
