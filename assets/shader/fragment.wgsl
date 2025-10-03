@fragment
fn fs_main(@location(0) frag_position: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(frag_position.x, 0.0, frag_position.y, 1.0);
}