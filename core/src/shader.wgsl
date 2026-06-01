struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

struct ColourStop {
  concentration: f32,
  colour: vec4<f32>,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var sample: sampler;

@vertex
fn vtx_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
  var positions = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
    vec2(-1.0,  1.0), vec2( 1.0, -1.0), vec2( 1.0,  1.0),
  );

  var uvs = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
  );
  
  var out: VertexOutput;
  
  out.position = vec4(positions[vertex_index], 0.0, 1.0);
  out.uv = uvs[vertex_index];
  
  return out;
}

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let raw_val = textureSampleLevel(texture, sample, in.uv, 0.0).r;
    
  if (raw_val <= -1.0) {
      return vec4<f32>(0.3, 0.3, 0.3, 1.0);
  }

  let concentration = clamp(raw_val, 0.0, 1.0);
  
  let stops = array<ColourStop, 5>(
      ColourStop(0.01, vec4<f32>(0.0, 0.0, 0.294, 1.0)),
      ColourStop(0.25, vec4<f32>(0.0, 0.8,  1.0,   1.0)),
      ColourStop(0.5,  vec4<f32>(0.0, 1.0,  0.0,   1.0)),
      ColourStop(0.75, vec4<f32>(1.0, 1.0,  0.0,   1.0)),
      ColourStop(1.0,  vec4<f32>(1.0, 0.0,  0.0,   1.0)),
  );

  if (concentration < stops[0].concentration) {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }
  
  if (concentration >= stops[4].concentration) {
    return stops[4].colour;
  }

  for (var i = 0u; i < 4u; i = i + 1u) {
    let c0 = stops[i].concentration;
    let c1 = stops[i + 1].concentration;

    if (concentration >= c0 && concentration < c1) {
      let factor = (concentration - c0) / (c1 - c0);
      return mix(stops[i].colour, stops[i + 1].colour, factor);
    }
  }

  return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
