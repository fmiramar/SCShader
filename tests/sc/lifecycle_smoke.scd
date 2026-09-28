(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var expectedVersion = File.readAllString(project +/+ "VERSION").stripWhiteSpace;
var server = ShaderServer.new(\scshaderLifecycleSmoke, "127.0.0.1", 57149);
var completed = false, gotPong = false, gotStatus = false, finish;

finish = { |passed, message|
    if(completed.not) {
        completed = true;
        message.postln;
        server.quit({
            passed.if({ "SCSHADER_LIFECYCLE_OK".postln; 0.exit }, {
                "SCSHADER_LIFECYCLE_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(2.0, {
            passed.if({ 0.exit }, { 1.exit });
            nil
        });
    };
};

if(PathName(rendererPath).isFile.not) {
    finish.value(false, "SCSHADER_LIFECYCLE_MISSING_RENDERER");
} {
    server.rendererPath_(rendererPath);
    server.waitForBoot({ |shaderServer|
        if(shaderServer.version != expectedVersion or: { shaderServer.backend.isNil }
            or: { shaderServer.device.isNil }) {
            finish.value(false, "SCSHADER_LIFECYCLE_BAD_READY");
        } {
            shaderServer.sync(1.0, { |receiver, reply|
                if(reply[1].asInteger == 1) { gotPong = true };
                if(gotStatus and: { gotPong }) { finish.value(true, "SCSHADER_LIFECYCLE_ROUNDTRIP") };
            });
            shaderServer.status({ |receiver, status|
                // shaderCount reports language-created resources. The renderer's
                // built-in fallback is deliberately not counted.
                if(status[\frameIndex] >= 0 and: { status[\shaderCount] == 0 }
                    and: { status[\windowCount] == 1 }) {
                    gotStatus = true;
                };
                if(gotStatus and: { gotPong }) { finish.value(true, "SCSHADER_LIFECYCLE_ROUNDTRIP") };
            });
        };
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_LIFECYCLE_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(8.0, {
        finish.value(false, "SCSHADER_LIFECYCLE_TIMEOUT");
        nil
    });
};
)
