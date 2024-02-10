@group(0) @binding(0) var origin_tx: texture_2d<f32>;
@group(0) @binding(1) var direction_tx: texture_2d<f32>;
@group(0) @binding(2) var<storage> accel_struct_buf: array<u32>;
@group(0) @binding(3) var<storage> inst_buf: array<mat4x4f>;

struct InstRange {
  start: u32,
  end: u32,
}

struct GeometryRanges {
  ind_start: u32,
  ind_end: u32,

  vert_start: u32,
  vert_end: u32,

  uv_start: u32,
  uv_end: u32,
}

@group(1) @binding(0) var<uniform> inst_range: InstRange;
@group(1) @binding(1) var<uniform> g_ranges: GeometryRanges;
@group(1) @binding(2) var<storage> mesh_buf: array<u32>;
@group(1) @binding(3) var emissive_tx: texture_2d<f32>;
@group(1) @binding(4) var emissive_s: texture_2d<f32>;

@group(2) @binding(0) var g_albedo_tx: texture_2d<f32>;
@group(2) @binding(1) var g_emissive_tx: texture_2d<f32>;
@group(2) @binding(2) var g_position_tx: texture_2d<f32>;
@group(2) @binding(3) var g_normal_tx: texture_2d<f32>;

@group(3) @binding(0) var g_albedo_stx: texture_storage_2d<rgba8unorm, write>;
@group(3) @binding(1) var g_emissive_stx: texture_storage_2d<rgba8unorm, write>;
@group(3) @binding(2) var g_position_stx: texture_storage_2d<rgba16float, write>;
@group(3) @binding(3) var g_normal_stx: texture_storage_2d<rg8unorm, write>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id : vec3u) {
  let gid = global_id.xy;
  textureLoad(origin_tx, gid, 0);
  textureLoad(direction_tx, gid, 0);
}

