//@group(0) @binding(0) var t_prev_albedo: texture_2d<f32>;
//@group(0) @binding(1) var t_prev_position: texture_2d<f32>;
//@group(0) @binding(2) var t_prev_normal: texture_2d<f32>;

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;

struct VertexInput {
  @location(0) position: vec3f,
  @location(1) uv: vec2f,
  @builtin(vertex_index) index: u32,
}

struct InstanceInput {
  @location(2) m0: vec4f,
  @location(3) m1: vec4f,
  @location(4) m2: vec4f,
  @location(5) m3: vec4f,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4f,
  @location(0) tex_coords: vec2f,
  @location(1) normal: vec3f,
}

@vertex
fn vs_main(
  vert: VertexInput,
  instance: InstanceInput,
) -> VertexOutput {
  var out: VertexOutput;

  let model_matrix = mat4x4f(
      instance.m0,
      instance.m1,
      instance.m2,
      instance.m3,
  );

  out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vert.position, 1.0);
  out.tex_coords = vert.uv;

  return out;
}

struct FragmentOutput {
  @location(0) albedo: vec4f,
  @location(1) emission: vec4f,
  @location(2) position: vec4f,
  @location(3) normal: vec4f,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
  //let coord = vec2i(
//i32(round(in.clip_position.x / in.clip_position.w)),
//  i32(round(in.clip_position.y / in.clip_position.w)));
  let view_position = in.clip_position.xyz / in.clip_position.w;
  let color = textureSample(t_albedo, s_albedo, in.tex_coords);

  //textureStore(ts_albedo, coord, color);
  //textureStore(ts_position, coord, vec4f(view_position, 0.0));
  //textureStore(ts_normal, coord, vec4f(in.normal, 0.0));
  var out: FragmentOutput;
  out.albedo = color;
  return out;

}
