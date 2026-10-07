(
var server = ShaderServer.new(\replyRegressionUnit);
var callbackCount = 0;
var callbacks = IdentityDictionary.new;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
try {
    // A truncated renderer reply must be reported without changing sync state.
    server.receivedPong(['/scshader/v1/pong'], nil, nil);
    assert.value(server.lastError.notNil and: { server.lastError[\code] == "E_PROTOCOL" },
        "Malformed pong was accepted");
    assert.value(server.clockSampleCount == 0, "Malformed pong changed clock samples");

    // A late or unsolicited reply must not create a clock sample or estimate.
    server.receivedPong(['/scshader/v1/pong', 999, Main.elapsedTime - 0.01, 1.0], nil, nil);
    assert.value(server.clockSampleCount == 0, "Unsolicited pong changed clock samples");
    assert.value(server.rendererClockOffset.isNil and: { server.clockRoundTrip.isNil },
        "Unsolicited pong changed the clock estimate");

    callbacks[17] = { |reply| callbackCount = callbackCount + 1 };
    server.instVarPut(\syncCallbacks, callbacks);
    server.receivedPong(['/scshader/v1/pong', 17, Main.elapsedTime - 0.01, 3.25], nil, nil);
    assert.value(callbackCount == 1, "Pending pong callback did not run");
    assert.value(server.clockSampleCount == 1, "Valid pong did not create one clock sample");
    assert.value(server.rendererClockOffset.isNumber and: { server.clockRoundTrip.isNumber },
        "Valid pong did not update the clock estimate");

    // The same reply arriving again is stale after the pending callback is removed.
    server.receivedPong(['/scshader/v1/pong', 17, Main.elapsedTime - 0.01, 999.0], nil, nil);
    assert.value(callbackCount == 1 and: { server.clockSampleCount == 1 },
        "Duplicate pong changed completed sync state");
    server.free;
    "SCSHADER_REPLY_REGRESSION_UNIT_OK".postln;
    0.exit;
} { |error|
    error.reportError;
    server.free;
    1.exit;
};
)
