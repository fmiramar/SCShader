// Install the complete SCShader extension first. Run one example at a time.

(
~scShaderServer = ShaderServer.new(\exampleServer);
~scShaderServer.waitForBoot { |server|
    ["SCShader ready", server.version, server.backend, server.device].postln;
    server.status { |receiver, status|
        ["SCShader status", status].postln;
    };
};
)

// The renderer is independent of scsynth. Close only its child process:
(
~scShaderServer.free;
)
