(
var assert = { |condition, message| if(condition.not) { Error(message).throw } };
var rejects = { |function| var rejected = false; try { function.value } { rejected = true }; assert.value(rejected, "Expected validation failure") };
try {
[
    [\float, 0.25], [\int, -2147483648], [\int, 2147483647],
    [\uint, 4294967295.0], [\bool, true], [\bool, false],
    [\vec2, [0.25, -0.5]], [\vec3, [1, 2, 3]], [\vec4, [1, 2, 3, 4]],
    [\mat2, [1, 2, 3, 4]], [\mat3, (1..9)], [\mat4, (1..16)]
].do { |pair|
    var value = ShaderDef.validateValue(*pair);
    var decoded = ShaderDef.decodeValue(pair[0], ShaderDef.encodeValue(pair[0], value));
    assert.value(decoded == value, "Typed value roundtrip failed: %".format(pair));
};
[
    [\int, 0.5], [\int, 2147483648.0], [\int, -2147483649.0], [\uint, -1],
    [\uint, 4294967296.0], [\bool, 1], [\float, inf], [\float, 1e100],
    [\vec3, [1, 2]], [\vec3, [1, 2, inf]], [\mat2, [[1, 2], [3, 4]]]
].do { |pair| rejects.value { ShaderDef.validateValue(*pair) } };
"SCSHADER_TYPED_CONTROLS_UNIT_OK".postln;
0.exit;
} { |error| error.reportError; 1.exit };
)
