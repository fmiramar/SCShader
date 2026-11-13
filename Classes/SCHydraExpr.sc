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

    blend { |other, amount = 0.5| ^this.append(\blend, [other, amount]) }

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
                ShaderDef.validateValue(\float, value);
            };
        };
        if(operations.isEmpty and: { spec[0] != \src }) {
            Error("SCHydra expression must start with a source.").throw;
        };
        if(operations.isEmpty.not and: { spec[0] == \src }) {
            Error("SCHydra source operations start new expressions.").throw;
        };
        if(operations.size >= 256) { Error("SCHydra expression exceeds 256 operations.").throw };
        operations.add([name, args.copy]);
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
                    value.isKindOf(SCHydraExpr).if({ value.copyWithMemo(memo, visiting) }, { value })
                });
            };
        };
        visiting.remove(this);
        ^clone
    }

    spec { |name|
        ^switch(name,
            \osc, { [\src, [\float, \float, \float]] },
            \noise, { [\src, [\float, \float]] },
            \rotate, { [\coord, [\float, \float]] },
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
            encodedArgs = args.collect { |value|
                value.isKindOf(SCHydraExpr).if({ [\expr, value.collectNodes(nodes, visiting, memo, depth + 1)] },
                    { [\float, ShaderDef.validateValue(\float, value)] });
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
        var bytes = List.new, result;
        [72, 89, 73, 82].do { |byte| bytes.add(byte) };
        this.writeInteger(bytes, 1, 2);
        this.writeInteger(bytes, 0, 2);
        this.writeInteger(bytes, nodes.size, 4);
        this.writeInteger(bytes, 0, 4);
        this.writeInteger(bytes, root, 4);
        nodes.do { |node|
            var name = node[\name].asString, categories = [\src, \coord, \color, \combine, \combineCoord];
            this.writeInteger(bytes, node[\id], 4);
            this.writeInteger(bytes, categories.indexOf(node[\category]), 1);
            this.writeInteger(bytes, node[\args].size, 1);
            this.writeInteger(bytes, name.size, 2);
            name.ascii.do { |character| this.writeInteger(bytes, character, 1) };
            this.writeInteger(bytes, node[\receiver], 4);
            node[\args].do { |argument|
                if(argument[0] == \expr) {
                    this.writeInteger(bytes, 2, 1);
                    this.writeInteger(bytes, argument[1], 4);
                } {
                    this.writeInteger(bytes, 0, 1);
                    ShaderDef.encodeValue(\float, argument[1]).do { |byte| bytes.add(byte.bitAnd(255)) };
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
        Error("SCHydra renderer transport starts in Step 06; use snapshot for offline work.").throw;
    }
}
