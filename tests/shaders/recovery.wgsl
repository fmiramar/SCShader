// Original deterministic recovery/readback fixture: image + Float32 data + typed controls.
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(3) var data: texture_2d<f32>;
struct Controls { gain: f32, tint: vec3<f32> };
@group(0) @binding(4) var<uniform> controls: Controls;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let vertices = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
    return vec4(vertices[i], 0., 1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4(textureLoad(source, vec2<i32>(0), 0).rgb * controls.gain
        + controls.tint + vec3(textureLoad(data, vec2<i32>(0), 0).r, 0., 0.), 1.);
}
