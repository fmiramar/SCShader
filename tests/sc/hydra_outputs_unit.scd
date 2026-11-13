(
var h=SCHydra.new, check={ |condition| if(condition.not) { Error("Output AST check failed").throw } };
try {
    [\o0,\o1,\o2,\o3].do { |slot,index|
        var snapshot=h.src(slot).scale(1.2).snapshot(slot), node;
        node=snapshot[\nodes][0];
        check.value(node[\name]==\src and: { node[\args][0]==[\output,index] });
        check.value(snapshot[\bytes].size>20);
    };
    check.value(try { h.src(\o4); false } { true });
    check.value(h.prev.snapshot(\o3)[\nodes][0][\name] == \prev);
    h.free; "SCSHADER_HYDRA_OUTPUTS_UNIT_OK".postln; 0.exit;
} { |error| error.reportError; h.free; 1.exit };
)
