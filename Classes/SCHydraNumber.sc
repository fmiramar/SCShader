SCHydraNumber : Object {
    var <hydra, <id, <kind, <values;
    *new { |hydra,id,kind,values| ^super.new.init(hydra,id,kind,values) }
    init { |h,i,k,v| hydra=h;id=i;kind=k;values=v.copy;this.validate;^this }
    validate {
        values.do { |v| ShaderDef.validateValue(\float,v); if(v.abs>1e6) {Error("Hydra number exceeds 1e6 bound").throw} };
        ^this
    }
    setValue { |value| values=[value];this.validate;^this }
    sequenceOnly { if(kind!=3) {Error("Sequence modifier requires h.seq").throw};^this }
    fast { |speed=1| this.sequenceOnly;values[0]=speed;this.validate;^this }
    offset { |phase=0.5| this.sequenceOnly;values[1]=phase%1;this.validate;^this }
    smooth { |amount=1| this.sequenceOnly;if(amount<0 or:{amount>1}) {Error("Smooth must be 0..1").throw};values[2]=amount;^this }
    ease { |name=\linear| this.sequenceOnly;
        if([\linear,\easeInOutQuad].includes(name).not) {Error("Supported native easing: linear, easeInOutQuad").throw};
        values[2]=1;values[3]=[\linear,\easeInOutQuad].indexOf(name);^this
    }
    fit { |low=0,high=1|
        var data,lo,hi;
        this.sequenceOnly;data=values.copyRange(4,values.size-1);lo=data.minItem;hi=data.maxItem;
        if(lo==hi) {Error("Cannot fit a constant sequence").throw};
        values=values.copyRange(0,3)++data.collect { |v| (v-lo)/(hi-lo)*(high-low)+low };
        values[1]=0;this.validate;^this
    }
}
