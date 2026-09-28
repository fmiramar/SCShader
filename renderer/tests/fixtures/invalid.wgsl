// Deliberately invalid WGSL for recovery testing.
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return this_identifier_does_not_exist;
}

