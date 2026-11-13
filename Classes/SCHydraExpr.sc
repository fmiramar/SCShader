SCHydraExpr : Object {
    var <hydra, operations;

    *new { |hydra, name, args| ^super.new.init(hydra, name, args) }

    init { |argHydra, name, args|
        hydra = argHydra;
        operations = List.new;
        this.append(name, args);
        ^this
    }

    rotate { |angle = 10, speed = 0| ^this.append(\rotate, [angle, speed]) }

    scale { |amount = 1.5, xMult = 1, yMult = 1, offsetX = 0.5, offsetY = 0.5|
        ^this.append(\scale, [amount, xMult, yMult, offsetX, offsetY])
    }

    color { |r = 1, g = 1, b = 1, a = 1| ^this.append(\color, [r, g, b, a]) }

    blend { |other, amount = 0.5| ^this.append(\blend, [other, amount]) }

    add { |other, amount = 1| ^this.append(\add, [other, amount]) }
    sub { |other, amount = 1| ^this.append(\sub, [other, amount]) }
    mult { |other, amount = 1| ^this.append(\mult, [other, amount]) }
    layer { |other| ^this.append(\layer, [other]) }
    diff { |other| ^this.append(\diff, [other]) }
    mask { |other| ^this.append(\mask, [other]) }

    modulateRepeat { |other, repeatX = 3.0, repeatY = 3.0, offsetX = 0.5, offsetY = 0.5| ^this.append(\modulateRepeat, [other, repeatX, repeatY, offsetX, offsetY]) }
    modulateRepeatX { |other, reps = 3.0, offset = 0.5| ^this.append(\modulateRepeatX, [other, reps, offset]) }
    modulateRepeatY { |other, reps = 3.0, offset = 0.5| ^this.append(\modulateRepeatY, [other, reps, offset]) }
    modulateKaleid { |other, nSides = 4.0| ^this.append(\modulateKaleid, [other, nSides]) }
    modulateScrollX { |other, scrollX = 0.5, speed = 0.0| ^this.append(\modulateScrollX, [other, scrollX, speed]) }
    modulateScrollY { |other, scrollY = 0.5, speed = 0.0| ^this.append(\modulateScrollY, [other, scrollY, speed]) }
    modulate { |other, amount = 0.1| ^this.append(\modulate, [other, amount]) }
    modulateScale { |other, multiple = 1.0, offset = 1.0| ^this.append(\modulateScale, [other, multiple, offset]) }
    modulatePixelate { |other, multiple = 10.0, offset = 3.0| ^this.append(\modulatePixelate, [other, multiple, offset]) }
    modulateRotate { |other, multiple = 1.0, offset = 0.0| ^this.append(\modulateRotate, [other, multiple, offset]) }
    modulateHue { |other, amount = 1.0| ^this.append(\modulateHue, [other, amount]) }

    pixelate { |pixelX = 20, pixelY = 20| ^this.append(\pixelate, [pixelX, pixelY]) }
    repeat { |repeatX = 3, repeatY = 3, offsetX = 0, offsetY = 0| ^this.append(\repeat, [repeatX, repeatY, offsetX, offsetY]) }
    repeatX { |reps = 3, offset = 0| ^this.append(\repeatX, [reps, offset]) }
    repeatY { |reps = 3, offset = 0| ^this.append(\repeatY, [reps, offset]) }
    kaleid { |nSides = 4| ^this.append(\kaleid, [nSides]) }
    scroll { |scrollX = 0.5, scrollY = 0.5, speedX = 0, speedY = 0| ^this.append(\scroll, [scrollX, scrollY, speedX, speedY]) }
    scrollX { |scrollX = 0.5, speed = 0| ^this.append(\scrollX, [scrollX, speed]) }
    scrollY { |scrollY = 0.5, speed = 0| ^this.append(\scrollY, [scrollY, speed]) }

    posterize { |bins = 3, gamma = 0.6| ^this.append(\posterize, [bins, gamma]) }
    shift { |r = 0.5, g = 0, b = 0, a = 0| ^this.append(\shift, [r, g, b, a]) }
    invert { |amount = 1| ^this.append(\invert, [amount]) }
    contrast { |amount = 1.6| ^this.append(\contrast, [amount]) }
    brightness { |amount = 0.4| ^this.append(\brightness, [amount]) }
    luma { |threshold = 0.5, tolerance = 0.1| ^this.append(\luma, [threshold, tolerance]) }
    thresh { |threshold = 0.5, tolerance = 0.04| ^this.append(\thresh, [threshold, tolerance]) }
    saturate { |amount = 2| ^this.append(\saturate, [amount]) }
    hue { |hue = 0.4| ^this.append(\hue, [hue]) }
    colorama { |amount = 0.005| ^this.append(\colorama, [amount]) }
    sum { |scale = #[1, 1, 1, 1]| ^this.append(\sum, [scale]) }
    r { |scale = 1, offset = 0| ^this.append(\r, [scale, offset]) }
    g { |scale = 1, offset = 0| ^this.append(\g, [scale, offset]) }
    b { |scale = 1, offset = 0| ^this.append(\b, [scale, offset]) }
    a { |scale = 1, offset = 0| ^this.append(\a, [scale, offset]) }

    append { |name, args|
        var spec = this.spec(name);
        hydra.checkLive;
        if(args.size != spec[1].size) { Error("Wrong SCHydra argument count for %.".format(name)).throw };
        args.do { |value, index|
            if(spec[1][index] == \expr) {
                if(value.isKindOf(SCHydraExpr).not or: { value.hydra !== hydra }) {
                    Error("SCHydra nested expression belongs to another session or is invalid.").throw;
                };
            } {
                if(spec[1][index] == \output) {
                    if(value.isInteger.not or: { value < 0 or: { value > 3 } }) { Error("Invalid output slot").throw };
                } {
                    if(value.isKindOf(SCHydraNumber)) {
                        if(value.hydra!==hydra or:{spec[1][index]!=\float}) {Error("Invalid Hydra numeric argument").throw};
                    } { ShaderDef.validateValue(spec[1][index], value) };
                };
            };
        };
        if(operations.isEmpty and: { spec[0] != \src }) {
            Error("SCHydra expression must start with a source.").throw;
        };
        if(operations.isEmpty.not and: { spec[0] == \src }) {
            Error("SCHydra source operations start new expressions.").throw;
        };
        if(operations.size >= 256) { Error("SCHydra expression exceeds 256 operations.").throw };
        operations.add([name, args.collect { |value| value.isSequenceableCollection.if({ value.copy }, { value }) }]);
        ^this
    }

    copy {
        ^this.copyWithMemo(IdentityDictionary.new, IdentitySet.new)
    }

    copyWithMemo { |memo, visiting|
        var clone, first = operations.first;
        if(visiting.includes(this)) { Error("SCHydra expression cycle.").throw };
        if(memo[this].notNil) { ^memo[this] };
        visiting.add(this);
        clone = SCHydraExpr.new(hydra, first[0], first[1].copy);
        memo[this] = clone;
        operations.do { |operation, index|
            if(index > 0) {
                clone.append(operation[0], operation[1].collect { |value|
                    value.isKindOf(SCHydraExpr).if({ value.copyWithMemo(memo, visiting) }, { value.isSequenceableCollection.if({ value.copy }, { value }) })
                });
            };
        };
        visiting.remove(this);
        ^clone
    }

    spec { |name|
        ^switch(name,
            \src, { [\src, [\output]] },
            \prev, { [\src, []] },
            \noise, { [\src, [\float, \float]] },
            \voronoi, { [\src, [\float, \float, \float]] },
            \osc, { [\src, [\float, \float, \float]] },
            \shape, { [\src, [\float, \float, \float]] },
            \gradient, { [\src, [\float]] },
            \solid, { [\src, [\float, \float, \float, \float]] },
            \rotate, { [\coord, [\float, \float]] },
            \scale, { [\coord, [\float, \float, \float, \float, \float]] },
            \pixelate, { [\coord, [\float, \float]] },
            \posterize, { [\color, [\float, \float]] },
            \shift, { [\color, [\float, \float, \float, \float]] },
            \repeat, { [\coord, [\float, \float, \float, \float]] },
            \repeatX, { [\coord, [\float, \float]] },
            \repeatY, { [\coord, [\float, \float]] },
            \kaleid, { [\coord, [\float]] },
            \scroll, { [\coord, [\float, \float, \float, \float]] },
            \scrollX, { [\coord, [\float, \float]] },
            \scrollY, { [\coord, [\float, \float]] },
            \invert, { [\color, [\float]] },
            \contrast, { [\color, [\float]] },
            \brightness, { [\color, [\float]] },
            \luma, { [\color, [\float, \float]] },
            \thresh, { [\color, [\float, \float]] },
            \color, { [\color, [\float, \float, \float, \float]] },
            \saturate, { [\color, [\float]] },
            \hue, { [\color, [\float]] },
            \colorama, { [\color, [\float]] },
            \sum, { [\color, [\vec4]] },
            \r, { [\color, [\float, \float]] },
            \g, { [\color, [\float, \float]] },
            \b, { [\color, [\float, \float]] },
            \a, { [\color, [\float, \float]] },
            \add, { [\combine, [\expr, \float]] },
            \sub, { [\combine, [\expr, \float]] },
            \mult, { [\combine, [\expr, \float]] },
            \layer, { [\combine, [\expr]] },
            \diff, { [\combine, [\expr]] },
            \mask, { [\combine, [\expr]] },
            \modulateRepeat, { [\combineCoord, [\expr,\float,\float,\float,\float]] },
            \modulateRepeatX, { [\combineCoord, [\expr,\float,\float]] },
            \modulateRepeatY, { [\combineCoord, [\expr,\float,\float]] },
            \modulateKaleid, { [\combineCoord, [\expr,\float]] },
            \modulateScrollX, { [\combineCoord, [\expr,\float,\float]] },
            \modulateScrollY, { [\combineCoord, [\expr,\float,\float]] },
            \modulate, { [\combineCoord, [\expr,\float]] },
            \modulateScale, { [\combineCoord, [\expr,\float,\float]] },
            \modulatePixelate, { [\combineCoord, [\expr,\float,\float]] },
            \modulateRotate, { [\combineCoord, [\expr,\float,\float]] },
            \modulateHue, { [\combineCoord, [\expr,\float]] },
            \blend, { [\combine, [\expr, \float]] },
            { Error("Unsupported SCHydra operation: %".format(name)).throw })
    }

    snapshot { |output = \o0|
        var nodes = List.new, visiting = IdentitySet.new, memo = IdentityDictionary.new;
        var root, bytes, result;
        hydra.checkLive;
        hydra.outputIndex(output);
        root = this.collectNodes(nodes, visiting, memo, 0);
        bytes = this.encode(nodes, root);
        result = (output: output, root: root, nodes: nodes.asArray, bytes: bytes);
        ^result
    }

    collectNodes { |nodes, visiting, memo, depth|
        var receiver = 0, first, cached;
        if(depth >= 64) { Error("SCHydra expression depth exceeds 64.").throw };
        if(visiting.includes(this)) { Error("SCHydra expression cycle.").throw };
        cached = memo[this];
        if(cached.notNil) { ^cached };
        visiting.add(this);
        operations.do { |operation|
            var name = operation[0], args = operation[1], spec = this.spec(name), encodedArgs;
            encodedArgs = args.collect { |value, index|
                value.isKindOf(SCHydraExpr).if({ [\expr, value.collectNodes(nodes, visiting, memo, depth + 1)] },
                    { value.isKindOf(SCHydraNumber).if({[\number,value]}, { [spec[1][index], (spec[1][index] == \output).if({ value }, { ShaderDef.validateValue(spec[1][index], value) })] }) });
            };
            if(nodes.size >= 256) { Error("SCHydra snapshot exceeds 256 nodes.").throw };
            first = (id: nodes.size + 1, name: name, category: spec[0], receiver: receiver,
                args: encodedArgs);
            nodes.add(first);
            receiver = first[\id];
        };
        visiting.remove(this);
        memo[this] = receiver;
        ^receiver
    }

    encode { |nodes, root|
        var bytes = List.new, result, params=List.new;
        nodes.do {|node|node[\args].do {|a|if(a[0]==\number and:{params.includes(a[1]).not}) {params.add(a[1])}}};
        [72, 89, 73, 82].do { |byte| bytes.add(byte) };
        this.writeInteger(bytes, 1, 2);
        this.writeInteger(bytes, params.isEmpty.if({0},{1}), 2);
        this.writeInteger(bytes, nodes.size, 4);
        this.writeInteger(bytes, params.size, 4);
        this.writeInteger(bytes, root, 4);
        params.do {|n|
            n.validate;this.writeInteger(bytes,n.id,4);this.writeInteger(bytes,n.kind,1);this.writeInteger(bytes,n.values.size,1);
            n.values.do {|v|ShaderDef.encodeValue(\float,v).do {|b|bytes.add(b.bitAnd(255))}};
        };
        nodes.do { |node|
            var name = node[\name].asString, categories = [\src, \coord, \color, \combine, \combineCoord];
            this.writeInteger(bytes, node[\id], 4);
            this.writeInteger(bytes, categories.indexOf(node[\category]), 1);
            this.writeInteger(bytes, node[\args].size, 1);
            this.writeInteger(bytes, name.size, 2);
            name.ascii.do { |character| this.writeInteger(bytes, character, 1) };
            this.writeInteger(bytes, node[\receiver], 4);
            node[\args].do { |argument|
                if(argument[0] == \number) {
                    this.writeInteger(bytes,1,1);this.writeInteger(bytes,argument[1].id,4);
                } {
                if(argument[0] == \expr) {
                    this.writeInteger(bytes, 2, 1);
                    this.writeInteger(bytes, argument[1], 4);
                } {
                    if(argument[0] == \output) {
                        this.writeInteger(bytes, 4, 1); this.writeInteger(bytes, argument[1], 1);
                    } {
                        this.writeInteger(bytes, (argument[0] == \vec4).if({ 5 }, { 0 }), 1);
                        ShaderDef.encodeValue(argument[0], argument[1]).do { |byte| bytes.add(byte.bitAnd(255)) };
                    };
                };
                };
            };
        };
        if(bytes.size > 65536) { Error("SCHydra IR exceeds 64 KiB.").throw };
        result = Int8Array.newClear(bytes.size);
        bytes.do { |byte, index| result[index] = byte };
        ^result
    }

    writeInteger { |bytes, value, count|
        if(value.isInteger.not or: { value < 0 or: { value > (count == 4).if({ 2147483647 }, { (1 << (count * 8)) - 1 }) } }) {
            Error("SCHydra IR integer is out of range.").throw;
        };
        count.do { |index| bytes.add((value >> ((count - index - 1) * 8)).bitAnd(255)) };
    }

    out { |output = \o0, action|
        hydra.checkLive;
        hydra.outputIndex(output);
        ^hydra.submit(this, output, action);
    }
}
