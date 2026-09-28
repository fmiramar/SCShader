(
var server = ShaderServer.new(\diagnosticsUnit, "127.0.0.1", 57169);
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var callback = false;
try {
    // Old renderers' short status replies remain readable without invented GPU measurements.
    server.receivedStatus(['/scshader/v1/status.reply', 60.0, 12, -1.0, 0.5, 0, 0, 0, 0, 1]);
    assert.value(server.lastStatus[\showStats].not, "Old overlay status default");
    assert.value(server.lastStatus[\gpuRecoveryCount] == 0, "Old recovery status default");
    assert.value(server.lastStatus[\gpuFrameMs] == -1, "Unavailable GPU timing changed");
    assert.value(server.lastStatus[\suppressedDiagnostics] == 0, "Old suppression status default");
    server.receivedStatus(['/scshader/v1/status.reply', 60, 12, -1, 0.5, 0, 0, 0, 0, 1,
        0, 0, 0, 0, 4096, 0, 0, 1, 123]);
    assert.value(server.lastStatus[\suppressedDiagnostics] == 123, "Suppression count not decoded");
    server.recoveryAction = { |receiver, recovery|
        assert.value(receiver === server, "Recovery callback receiver");
        assert.value(recovery[\feedbackHistoryCleared], "Missing history reset notice");
        callback = true;
    };
    server.receivedGpuRecovered(['/scshader/v1/gpu/recovered', 1, "history cleared", "Metal", "Test GPU"]);
    assert.value(callback and: { server.gpuRecoveryCount == 1 }, "Recovery callback not dispatched");
    assert.value(server.backend == "Metal" and: { server.device == "Test GPU" }, "GPU metadata not refreshed");
    "SCSHADER_DIAGNOSTICS_UNIT_OK".postln; 0.exit;
} { |error| error.reportError; 1.exit };
)
