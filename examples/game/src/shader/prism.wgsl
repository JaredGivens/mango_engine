struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var albedo_tx: texture_3d<f32>;
@group(1) @binding(1) var albedo_s: sampler;

struct VertexInput {
  @location(0) pos: vec2f,
  @builtin(vertex_index) i: u32,
}

struct InstanceInput {
  @location(1) props: u32,
  @location(2) height: vec2u,
  @builtin(instance_index) i: u32,
}

struct VertexOutput {
  @builtin(position) pos: vec4f,
  @location(0) tx_coords: vec3f,
  @location(1) color: vec3f,
  @location(2) normal: vec3f,
}


@vertex
fn vs_main(
  vert: VertexInput,
  inst: InstanceInput,
) -> VertexOutput {
  var out: VertexOutput;

  let edge_len = 6.0;
  let edge_len_h = edge_len * 0.5;
  let depth = edge_len_h * sqrt(3.0);
  let height_fac = 0.1;

  let height_shift = (vert.i & 3u) * 8u; 
  let height = height_fac * f32(
      (inst.height[vert.i / 4u] >> height_shift) & 0xffu );

  let inv = 1.0 - f32(2u & inst.props);
  let xpos = f32(3u * (inst.i & 15u)) + edge_len_h 
    * f32(inst.i / 16u);
  let zpos = f32(depth) * f32(inst.i / 16u);
  let model = mat4x4<f32> (
      inv, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, inv, 0.0,
      xpos, 0.0, zpos, 1.0,
  );

  let pos_m = model * vec4f(vert.pos.x, height, vert.pos.y, 1.0); 

  out.pos = camera.view_proj * pos_m;

  out.tx_coords = vec3f(
      pos_m.x,
      pos_m.y,
      pos_m.z,
      ) / edge_len;

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
  let view_position = in.pos.xyz / in.pos.w;
  let albedo = textureSample(albedo_tx, albedo_s, in.tx_coords);
  var out: FragmentOutput;
  out.albedo = albedo;
  out.position = in.pos;
  return out;
}
