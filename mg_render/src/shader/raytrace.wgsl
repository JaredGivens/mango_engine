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

const VOXEL_SIZE = 6.0;
const RAY_ITERATIONS = 16;

fn pcg3d(p: vec3u) -> vec3u {

  var v = p * 1664525u + 1013904223u;

  v.x += v.y*v.z;
  v.y += v.z*v.x;
  v.z += v.x*v.y;

  let s = vec3u(v.x >> 16u, v.y >> 16u, v.z >> 16u); 
  v ^= s;
  
  v.x += v.y*v.z;
  v.y += v.z*v.x;
  v.z += v.x*v.y;

  return v;
}

fn xxhash32(p: vec3u) -> u32 {
  let PRIME32_2 = 2246822519u; 
  let PRIME32_3 = 3266489917u;
  let PRIME32_4 = 668265263u;
  let PRIME32_5 = 374761393u;
  var h32 =  p.z + PRIME32_5 + p.x*PRIME32_3;
  h32 = PRIME32_4*((h32 << 17u) | (h32 >> (32u - 17u)));
  h32 += p.y * PRIME32_3;
  h32 = PRIME32_4*((h32 << 17u) | (h32 >> (32u - 17u)));
  h32 = PRIME32_2*(h32^(h32 >> 15u));
  h32 = PRIME32_3*(h32^(h32 >> 13u));
  return h32^(h32 >> 16u);
}

fn hash(pos: vec3f) -> u32 {
  let key = vec3u(pos / VOXEL_SIZE);
  let pcg = pcg3d(key);
  return pcg.x % 16u + (pcg.y % 16u) * 16u + (pcg.z % 16u) * 16u * 16u;
}

fn sphericalToNormal(p: vec2f) -> vec3f {
    let x = sin(p.y) * cos(p.x);
    let y = cos(p.y);
    let z = sin(p.y) * sin(p.x);

    return vec3f(x, y, z);
}

fn normalToSpherical(n: vec3f) -> vec2f {
    let azmuth = atan2(n.z, n.x);
    let inclination = asin(n.y);

    return vec2f(azmuth, inclination);
}
fn normalizeLargestComponent(v: vec3f) -> vec3f {
    let largestComponent = max(max(v.x, v.y), v.z);
    return v / largestComponent;
}

fn hi(n: u32) -> u32 { return n >> 16u; }
fn lo(n: u32) -> u32 { return n & ((1u << 16u) - 1u); }

fn evalInst(inst_i: u32) {
  let model_mat = inst_buf[inst_i];
  for (var i = g_ranges.ind_start; i < g_ranges.ind_end; i += 6u) {
    let odd = (i & 2u) >> 1u;
    var a_i = mesh_buf[i / 4u];
    var b_i = mesh_buf[i / 4u + odd];
    var c_i = mesh_buf[i / 4u + 1u];
    if odd == 1u {
      a_i = lo(a_i);
      b_i = hi(b_i);
      c_i = lo(c_i);
    } else {
      a_i = hi(a_i);
      b_i = lo(b_i);
      c_i = hi(c_i);
    }
    let av_i = g_ranges.vert_start / 4u + a_i * 3u;
    let bv_i = g_ranges.vert_start / 4u + b_i * 3u;
    let cv_i = g_ranges.vert_start / 4u + c_i * 3u;
    let av = model_mat * vec4f(
      bitcast<f32>(mesh_buf[av_i]),
      bitcast<f32>(mesh_buf[av_i + 1u]),
      bitcast<f32>(mesh_buf[av_i + 2u]),
      1.0f
    );
    let bv = model_mat * vec4f(
      bitcast<f32>(mesh_buf[bv_i]),
      bitcast<f32>(mesh_buf[bv_i + 1u]),
      bitcast<f32>(mesh_buf[bv_i + 2u]),
      1.0f
    );
    let cv = model_mat * vec4f(
      bitcast<f32>(mesh_buf[cv_i]),
      bitcast<f32>(mesh_buf[cv_i + 1u]),
      bitcast<f32>(mesh_buf[cv_i + 2u]),
      1.0f
    );

    let auv_i = g_ranges.uv_start / 4u + a_i * 2u;
    let buv_i = g_ranges.uv_start / 4u+ b_i * 2u;
    let cuv_i = g_ranges.uv_start / 4u+ c_i * 2u;
    let auv = vec2f(
      bitcast<f32>(mesh_buf[auv_i]),
      bitcast<f32>(mesh_buf[auv_i + 1u])
    );
    let buv = vec2f(
      bitcast<f32>(mesh_buf[buv_i]),
      bitcast<f32>(mesh_buf[buv_i + 1u])
    );
    let cuv = vec2f(
      bitcast<f32>(mesh_buf[cuv_i]),
      bitcast<f32>(mesh_buf[cuv_i + 1u])
    );
  }
}

fn evalVoxel(ray_pos: vec3f) {
  let vox_i = hash(ray_pos);
  for (var i = 0u; i < 8u; i++) {
    let inst_i = accel_struct_buf[vox_i + i];
    if (inst_i >= inst_range.start && inst_i < inst_range.end) {
      workgroupBarrier();
      evalInst(inst_i);
    }
  }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id : vec3u) {
  let gid = global_id.xy;
  var ray_pos = textureLoad(origin_tx, gid, 0).xyz;
  let spherical_ray_dir = textureLoad(direction_tx, gid, 0).xy;

  let ray_dir = sphericalToNormal(spherical_ray_dir);
  let ray_step = normalizeLargestComponent(ray_dir) * VOXEL_SIZE;

  for (var i = 0; i < RAY_ITERATIONS; i++) {
    ray_pos += ray_step;
    evalVoxel(ray_pos);
  }
}

