(
var h=SCHydra.new, other=SCHydra.new, check, child, expr, held;
check={ |condition| if(condition.not) { Error("Modulation AST check failed").throw } };
try {
[[\modulateRepeat,[3.0,3.0,0.5,0.5]],
[\modulateRepeatX,[3.0,0.5]],
[\modulateRepeatY,[3.0,0.5]],
[\modulateKaleid,[4.0]],
[\modulateScrollX,[0.5,0.0]],
[\modulateScrollY,[0.5,0.0]],
[\modulate,[0.1]],
[\modulateScale,[1.0,1.0]],
[\modulatePixelate,[10.0,3.0]],
[\modulateRotate,[1.0,0.0]],
[\modulateHue,[1.0]]].do { |row|
    var node=h.osc.perform(row[0],h.gradient).snapshot[\nodes].last;
    check.value(node[\category]==\combineCoord);
    check.value(node[\args][0][0]==\expr);
    check.value(node[\args].drop(1).collect(_[1])==row[1]);
};
child=h.noise; expr=h.osc.modulate(child).copy; held=expr.snapshot[\bytes]; child.rotate(0.8);
check.value(expr.snapshot[\bytes]==held);
check.value(try { h.osc.modulate(other.solid); false } { true });
check.value(try { h.osc.modulate(0.5); false } { true });
h.free; other.free; "SCSHADER_HYDRA_MODULATION_UNIT_OK".postln; 0.exit;
} { |error| error.reportError; h.free; other.free; 1.exit };
)
