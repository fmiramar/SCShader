(
var h=SCHydra.new, other=SCHydra.new, check, base, child, expr, held;
check={ |condition| if(condition.not) { Error("Combine AST check failed").throw } };
try {
    [\add,\sub,\blend,\mult,\layer,\diff,\mask].do { |name|
        var args;
        expr=h.osc(10).perform(name,h.shape(4,0.4).rotate(0.2));
        args=expr.snapshot[\nodes].last[\args];
        check.value(args[0][0]==\expr);
        if([\add,\sub,\mult,\blend].includes(name)) {
            check.value(args[1][1]==if(name==\blend) { 0.5 } { 1 });
        } { check.value(args.size==1) };
    };
    child=h.shape; base=h.osc.blend(child); expr=base.copy;
    held=expr.snapshot[\bytes]; child.rotate(0.8);
    check.value(expr.snapshot[\bytes]==held);
    check.value(base.snapshot[\bytes]!=held);
    check.value(try { h.osc.add(other.solid); false } { true });
    check.value(try { h.osc.mask(0.5); false } { true });
    h.free; other.free; "SCSHADER_HYDRA_COMBINE_UNIT_OK".postln; 0.exit;
} { |error| error.reportError; h.free; other.free; 1.exit };
)
