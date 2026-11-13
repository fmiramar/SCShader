// Quiet startup image for Hydra examples before their first accepted output.
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let points = array<vec2<f32>,3>(vec2<f32>(-1.,-1.),vec2<f32>(3.,-1.),vec2<f32>(-1.,3.));
    return vec4<f32>(points[index],0.,1.);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.,0.,0.,1.); }
