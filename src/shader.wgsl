struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

@vertex
fn vtx_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  var positions = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
    vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2( 1.0,  1.0),
  );

  var uvs = array<vec2<f32>, 6>(
    vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(0.0, 0.0),
    vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(1.0, 0.0),
  );
  
  var out: VertexOutput;
  
  out.position = vec4(positions[vertex_index], 0.0, 1.0);
  out.uv = uvs[vertex_index];
  
  return out;
}

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t, s, in.uv);
}
