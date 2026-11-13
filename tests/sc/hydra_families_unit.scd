(
var h = SCHydra.new, expression, snapshot, weights, copy, held;
var check = { |test, message| if(test.not) { Error(message).throw } };
var rejects = { |action| var failed = false; try { action.value } { failed = true }; check.value(failed, "Expected rejection") };
try {
[
    [\noise, 0, [10, 0.1]],
    [\voronoi, 0, [5, 0.3, 0.3]],
    [\osc, 0, [60, 0.1, 0]],
    [\shape, 0, [3, 0.3, 0.01]],
    [\gradient, 0, [0]],
    [\solid, 0, [0, 0, 0, 1]],
    [\rotate, 1, [10, 0]],
    [\scale, 1, [1.5, 1, 1, 0.5, 0.5]],
    [\pixelate, 1, [20, 20]],
    [\posterize, 2, [3, 0.6]],
    [\shift, 2, [0.5, 0, 0, 0]],
    [\repeat, 1, [3, 3, 0, 0]],
    [\repeatX, 1, [3, 0]],
    [\repeatY, 1, [3, 0]],
    [\kaleid, 1, [4]],
    [\scroll, 1, [0.5, 0.5, 0, 0]],
    [\scrollX, 1, [0.5, 0]],
    [\scrollY, 1, [0.5, 0]],
    [\invert, 2, [1]],
    [\contrast, 2, [1.6]],
    [\brightness, 2, [0.4]],
    [\luma, 2, [0.5, 0.1]],
    [\thresh, 2, [0.5, 0.04]],
    [\color, 2, [1, 1, 1, 1]],
    [\saturate, 2, [2]],
    [\hue, 2, [0.4]],
    [\colorama, 2, [0.005]],
    [\sum, 2, [[1, 1, 1, 1]]],
    [\r, 2, [1, 0]],
    [\g, 2, [1, 0]],
    [\b, 2, [1, 0]],
    [\a, 2, [1, 0]]
].do { |row|
    expression = if(row[1] == 0) { h.perform(row[0]) } {
        var base = if(row[1] == 1) { h.gradient } { h.solid(0.17, 0.38, 0.63, 0.72) };
        base.perform(row[0])
    };
    snapshot = expression.snapshot;
    check.value(snapshot[\nodes].last[\name] == row[0], "Missing SC operation: %".format(row[0]));
    check.value(snapshot[\nodes].last[\args].collect(_[1]) == row[2], "Default mismatch: %".format(row[0]));
};
weights = FloatArray[0.1,0.2,0.3,0.4];
expression = h.solid.sum(weights); copy = expression.copy; held = expression.snapshot[\bytes].copy;
weights[0] = 9;
check.value(expression.snapshot[\bytes] == held and: { copy.snapshot[\bytes] == held }, "Vector literal or copy shares caller data");
rejects.value { h.solid.sum([1,2,3]) };
rejects.value { h.solid.sum([1,2,3,inf]) };
h.free;
"SCSHADER_HYDRA_FAMILIES_UNIT_OK".postln; 0.exit;
} { |error| error.reportError; h.free; 1.exit };
)
