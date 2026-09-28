ShaderDef : Object {
    classvar all;
    var <name, <path, <uniforms, <sourceType;

    *initClass {
        all = IdentityDictionary.new;
    }

    *new { |name, path, uniforms, sourceType = \wgsl|
        ^super.new.init(name, path, uniforms, sourceType)
    }

    *at { |name| ^all[name.asSymbol] }

    *all { ^all.copy }

    init { |argName, argPath, argUniforms, argSourceType|
        if(argName.isNil or: { argPath.isNil }) {
            Error("ShaderDef requires a name and a shader source path.").throw;
        };
        name = argName.asSymbol;
        path = argPath.asString.standardizePath;
        sourceType = argSourceType.asSymbol;
        if([\wgsl, \glsl, \shadertoy].includes(sourceType).not) {
            Error("ShaderDef sourceType must be \\wgsl, \\glsl, or \\shadertoy.").throw;
        };
        uniforms = (argUniforms ? (amount: \float)).copy;
        if(uniforms.isKindOf(Event).not) {
            Error("ShaderDef uniforms must be an Event such as (amount: \\float).").throw;
        };
        uniforms.keysValuesDo { |uniformName, type|
            if(this.class.valueSize(type).isNil) {
                Error("Unsupported ShaderDef uniform type: %".format(type)).throw;
            };
            if(uniformName.asSymbol == \amount and: { type != \float }) {
                Error("The built-in amount uniform must remain a float.").throw;
            };
        };
        uniforms[\amount] = \float;
        all[name] = this;
        ^this
    }

    uniformType { |uniformName| ^uniforms[uniformName.asSymbol] }

    *valueSize { |type|
        ^(float: 1, int: 1, uint: 1, bool: 1, vec2: 2, vec3: 3, vec4: 4,
            mat2: 4, mat3: 9, mat4: 16)[type]
    }

    *validateValue { |type, value|
        var size = this.valueSize(type), values;
        if(size.isNil) { Error("Unknown Shader uniform type: %".format(type)).throw };
        if(type == \bool) {
            if(value !== true and: { value !== false }) {
                Error("A bool Shader uniform requires true or false.").throw;
            };
            ^value
        };
        if(size == 1) { values = [value] } {
            if(value.isSequenceableCollection.not or: { value.isString or: { value.size != size } }) {
                Error("A % Shader uniform requires exactly % flat, column-major numbers.".format(type, size)).throw;
            };
            values = value.asArray;
        };
        values.do { |number|
            if(number.isNumber.not or: { number.isNaN or: { number.abs >= inf } }) {
                Error("Shader uniform components must be finite numbers.").throw;
            };
            if([\int, \uint].includes(type)) {
                // sclang Integers are signed 32-bit. A Float represents the upper
                // uint32 half exactly; a decimal integer literal would wrap before
                // this method could validate it.
                if((type == \int).if({ number.isInteger.not }, {
                    number < 0 or: { number > 4294967295.0 or: { number != number.floor } }
                })) { Error("Shader % requires an exact 32-bit integer value (use a Float for uint > 2147483647).".format(type)).throw };
            } {
                if(number.abs > 3.4028234663852886e38) {
                    Error("Shader uniform component exceeds the Float32 range.").throw;
                };
            };
        };
        if([\int, \uint].includes(type)) { ^value };
        ^(size == 1).if({ value.asFloat }, { values.collect(_.asFloat) })
    }

    *encodeValue { |type, value|
        var values = (type == \bool).if({ [value.binaryValue] }, { value.asArray });
        var bytes = Int8Array.newClear(values.size * 4);
        var integer = [\int, \uint, \bool].includes(type);
        values.do { |number, index|
            var bits = integer.if({
                (type == \uint and: { number > 2147483647 }).if({ (number - 4294967296.0).asInteger }, { number.asInteger })
            }, { number.asFloat.as32Bits });
            4.do { |byte| bytes[(index * 4) + byte] = (bits >> (24 - (byte * 8))).bitAnd(255) };
        };
        ^bytes
    }

    *decodeValue { |type, bytes|
        var size = this.valueSize(type), values;
        if(size.isNil or: { bytes.size != (size * 4) }) { Error("Malformed Shader uniform value reply.").throw };
        values = Array.fill(size, { |index|
            var bits = 0;
            4.do { |byte| bits = (bits << 8).bitOr(bytes[(index * 4) + byte].bitAnd(255)) };
            switch(type,
                \bool, { bits != 0 },
                \uint, { (bits < 0).if({ bits.asFloat + 4294967296.0 }, { bits }) },
                \int, { bits },
                { Float.from32Bits(bits) }
            )
        });
        ^(size == 1).if({ values[0] }, { values })
    }

    free {
        if(all[name] === this) { all.removeAt(name) };
        ^this
    }
}
