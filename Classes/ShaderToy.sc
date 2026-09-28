ShaderToy : ShaderDef {
    *new { |name, path|
        ^super.new(name, path, (amount: \float), \shadertoy)
    }
}
