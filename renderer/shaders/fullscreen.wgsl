struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    amount: f32,
    source_mix: f32,
    feedback: f32,
    padding0: f32,
    padding1: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var source_texture: texture_2d<f32>;

@group(0) @binding(2)
var source_sampler: sampler;

// ShaderBuffer data is a one-row R32Float texture. Use textureLoad rather
// than textureSample: the binding is deliberately non-filterable so the
// index-to-bin relationship stays exact for FFT and waveform data.
@group(0) @binding(3)
var spectrum_buffer: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let position = positions[vertex_index];
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = position * 0.5 + vec2<f32>(0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let safe_height = max(uniforms.resolution.y, 1.0);
    var point = input.uv * 2.0 - vec2<f32>(1.0);
    point.x *= uniforms.resolution.x / safe_height;

    let amount = uniforms.amount;
    let radius = length(point);
    let phase = atan2(point.y, point.x);
    let wave = 0.5 + 0.5 * sin(phase * 7.0 - uniforms.time * 1.8 + radius * 12.0);
    let glow = exp(-3.2 * radius) * (0.45 + 0.85 * amount);
    let pulse = 0.55 + 0.45 * sin(uniforms.time * 1.2 + amount * 3.14159265);
    let generated = vec3<f32>(
        glow * (0.3 + wave * amount),
        glow * (0.4 + pulse * 0.5),
        glow * (0.7 + (1.0 - wave) * 0.6),
    );
    let sampled = textureSample(source_texture, source_sampler, input.uv).rgb;
    let spectrum_energy = textureLoad(spectrum_buffer, vec2<i32>(0, 0), 0).r;
    let source_color = mix(generated, sampled, clamp(uniforms.source_mix, 0.0, 1.0));
    let color = mix(source_color, sampled, clamp(uniforms.feedback, 0.0, 1.0))
        + vec3<f32>(0.03 * max(spectrum_energy, 0.0));
    return vec4<f32>(color, 1.0);
}
