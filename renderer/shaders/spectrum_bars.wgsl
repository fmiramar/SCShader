// Original SCShader example: the first 128 raw FFT magnitudes as visible bars.
struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    amount: f32,
};
@group(0) @binding(0) var<uniform> sc: Uniforms;
@group(0) @binding(3) var spectrum: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    var out: VertexOutput;
    out.position = vec4(positions[index], 0.0, 1.0);
    out.uv = positions[index] * 0.5 + vec2(0.5);
    return out;
}

@fragment fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let bins = min(textureDimensions(spectrum).x, 128u);
    let column = clamp(input.uv.x, 0.0, 0.99999) * f32(bins);
    let magnitude = max(textureLoad(spectrum, vec2<i32>(i32(column), 0), 0).r, 0.0);
    let height = clamp(sqrt(magnitude) * 2.0, 0.0, 0.95);
    let inside = input.uv.y < height && fract(column) < 0.8;
    let color = select(vec3(0.015, 0.02, 0.04), vec3(0.1, 0.55, 0.95) * sc.amount, inside);
    return vec4(color, 1.0);
}
