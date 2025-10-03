struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_position: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput  {
    var output: VertexOutput;

    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let pos = positions[vertex_index];
    output.position = vec4<f32>(pos.x, -pos.y, 0.0, 1.0);
    output.frag_position = (pos + vec2(1.0)) * 0.5;
    return output;
}
