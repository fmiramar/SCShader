(
var project = thisProcess.nowExecutingPath.dirname.dirname.dirname;
var rendererPath = "SCSHADER_RENDERER".getenv ? (project +/+ "renderer/target/debug" +/+ ShaderServer.rendererExecutableName);
var sourcePath = project +/+ "renderer/shaders/fullscreen.wgsl";
var validPath = project +/+ "build/shader-lifecycle-reload.wgsl";
var source;
var invalidPath = project +/+ "renderer/tests/fixtures/invalid.wgsl";
var server = ShaderServer.new(\scshaderShaderLifecycle, "127.0.0.1", 57147);
var definition, shader, complete = false, finish, startedAt = Main.elapsedTime;

finish = { |passed, message|
    if(complete.not) {
        complete = true;
        message.postln;
        shader.tryPerform(\free);
        definition.tryPerform(\free);
        File.delete(validPath);
        server.quit({
            passed.if({ "SCSHADER_SHADER_LIFECYCLE_OK".postln; 0.exit }, {
                "SCSHADER_SHADER_LIFECYCLE_FAILED".postln; 1.exit
            });
        });
        SystemClock.sched(2.0, { passed.if({ 0.exit }, { 1.exit }); nil });
    };
};

if([rendererPath, sourcePath, invalidPath].any { |path| PathName(path).isFile.not }) {
    finish.value(false, "SCSHADER_SHADER_LIFECYCLE_MISSING_FIXTURE");
} {
    source = File.readAllString(sourcePath);
    File.use(validPath, "w", { |file| file.write(source) });
    server.rendererPath_(rendererPath);
    server.waitForBoot({ |shaderServer|
        definition = ShaderDef(\scshaderLifecycleDef, validPath);
        shader = Shader(definition, [\amount, 0.35], shaderServer);
        SystemClock.sched(0.05, {
            if(shader.isRunning) {
                if(shader.uniforms[\amount] != \float) {
                    finish.value(false, "SCSHADER_SHADER_LIFECYCLE_REFLECTION_FAILED");
                    nil
                } {
                    // Change an owned fixture portably; no POSIX touch dependency.
                    File.use(validPath, "w", { |file| file.write(source ++ "\n// hot reload\n") });
                    SystemClock.sched(0.5, {
                        if(shader.reloadCount != 1) {
                            finish.value(false, "SCSHADER_SHADER_LIFECYCLE_HOT_RELOAD_FAILED");
                        } {
                            shader.reload;
                            SystemClock.sched(0.15, {
                                if(shader.reloadCount != 2) {
                                    finish.value(false, "SCSHADER_SHADER_LIFECYCLE_RELOAD_FAILED");
                                } {
                                    shader.set(\amount, 0.8);
                                    // A failed compile for another ID must leave this shader live.
                                    shaderServer.sendMsg('/scshader/v1/shader/create', shader.id + 1, invalidPath);
                                    SystemClock.sched(0.25, {
                                        if(shader.isRunning.not) {
                                            finish.value(false, "SCSHADER_SHADER_LIFECYCLE_FALLBACK_FAILED");
                                        } {
                                            shader.free;
                                            SystemClock.sched(0.1, {
                                                if(shader.isFreed and: { Shader.at(shader.id).isNil }) {
                                                    finish.value(true, "SCSHADER_SHADER_LIFECYCLE_ROUNDTRIP");
                                                } {
                                                    finish.value(false, "SCSHADER_SHADER_LIFECYCLE_FREE_FAILED");
                                                };
                                                nil
                                            });
                                        };
                                        nil
                                    });
                                };
                                nil
                            });
                        };
                        nil
                    });
                }
            } {
                if((Main.elapsedTime - startedAt) > 6.0) {
                    finish.value(false, "SCSHADER_SHADER_LIFECYCLE_CREATE_TIMEOUT");
                    nil
                } {
                    0.05
                }
            }
        });
    }, { |shaderServer, error|
        finish.value(false, "SCSHADER_SHADER_LIFECYCLE_BOOT_FAILED: %".format(error));
    });
    SystemClock.sched(10.0, { finish.value(false, "SCSHADER_SHADER_LIFECYCLE_TIMEOUT"); nil });
};
)
