(
var server = ShaderServer.new(\handshakeUnit, "127.0.0.1", 57246);
var ready = ['/scshader/v1/ready', 1, "first", "Dx12", "test device"];
var calls = 0;
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
// Unsolicited replies cannot turn a fresh controller into a running one.
server.receivedReady(ready);
assert.value(server.isRunning.not, "Unsolicited ready revived a stopped server");
server.instVarPut(\isBooting, true);
server.instVarPut(\bootCallbacks, List[[{ calls = calls + 1 }, nil]]);
server.receivedReady(ready);
assert.value(server.isRunning and: { calls == 1 }, "Boot did not complete once");
server.receivedReady(['/scshader/v1/ready', 1, "duplicate", "wrong", "wrong"]);
assert.value(server.version == "first" and: { calls == 1 }, "Duplicate ready changed live state");
server.instVarPut(\quitting, true);
server.instVarPut(\isBooting, true);
server.receivedReady(['/scshader/v1/ready', 1, "late", "wrong", "wrong"]);
assert.value(server.version == "first", "Ready changed a quitting controller");
server.free;
"SCSHADER_HANDSHAKE_UNIT_OK".postln;
0.exit;
)
