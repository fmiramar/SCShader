// Original SCShader fixture: all ten supported control types.
// @scshader bool enabled
struct Builtins {
    resolution: vec2<f32>, time: f32, amount: f32,
};
@group(0) @binding(0) var<uniform> sc: Builtins;

struct Controls {
    gain: f32,
    count: i32,
    seed: u32,
    enabled: u32,
    offset: vec2<f32>,
    color: vec3<f32>,
    tint: vec4<f32>,
    rotate: mat2x2<f32>,
    transform: mat3x3<f32>,
    projection: mat4x4<f32>,
};
@group(0) @binding(4) var<uniform> controls: Controls;

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
    let p = controls.rotate * (input.uv - vec2(0.5) - controls.offset);
    let q = controls.transform * vec3(p, 1.0);
    let projected = controls.projection * vec4(q, 1.0);
    let stripes = 0.5 + 0.5 * sin(projected.x * f32(max(controls.count, 1)) + sc.time);
    let seed_tone = f32(controls.seed % 17u) / 17.0;
    let color = mix(controls.color, controls.tint.rgb, stripes) * controls.gain * sc.amount;
    return vec4(select(vec3(0.0), color + vec3(0.02 * seed_tone), controls.enabled != 0u), 1.0);
}
