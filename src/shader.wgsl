// // Vertex Shader

// struct VertexOutput {
//   @builtin(position) clip_position: vec4<f32>, 
// };

// @stage(vertex)
// fn vs_main (@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
//   var out: VertexOutput;

//   let x = f32(1 - i32(in_vertex_index)) * .5;
//   let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * .5;

//   out.clip_position = vec4<f32>(x, y, .0, 1.); 

//   return out; 
// }

// // Fragment Shader
// @stage(fragment)
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//   return vec4<f32>(.3, .2, .1, 1.); 
// }


// Vertex shader

struct VertexInput {
  @location(0) position: vec3<f32>, 
  @location(1) color: vec3<f32>, 
  @location(2) texture_coords: vec2<f32>, 
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>, 
  @location(0)       color: vec3<f32>,
  @location(1)       texture_coords: vec2<f32>, 
};

@stage(vertex)
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.texture_coords = model.texture_coords;

    // Flip, WGPU texture coordinate are like dxd, 1,1 lower right
    out.texture_coords.y = 1. - out.texture_coords.y; 

    return out;
}

// Fragment shader
@group(0)
@binding(0)
var diffuse_texture : texture_2d<f32>;

@group(0)
@binding(1)
var diffuse_sampler : sampler; 

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(in.color, 1.0);
  return textureSample(diffuse_texture, diffuse_sampler, in.texture_coords); 
}
