@group(0) @binding(0) var t_albedo: texture_2d<f32>;
@group(0) @binding(1) var t_emissive: texture_2d<f32>;
@group(0) @binding(2) var t_position: texture_2d<f32>;
@group(0) @binding(3) var t_normal: texture_2d<f32>;

struct Entry {
  lifetime: u32,
  color: vec3f,
}
const CACHE_BUCKET_SIZE = 16;
@group(1) @binding(0) var<storage> irradiance_cache: array<Entry>;

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

@vertex
fn vs_main(
  @builtin(vertex_index) i: u32,
) -> @builtin(position) vec4f {
  var pos = array(
    vec2(-1.0, 1.0), vec2(-1.0, -1.0), 
    vec2(1.0, 1.0), vec2(1.0, -1.0)
  );
  return vec4f(pos[i], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
  let dim = vec2f(textureDimensions(t_albedo));
  let coord = vec2i(floor(pos.xy));

  // last param is mip level
  let albedo = textureLoad(t_albedo, coord, 0);
  let position = textureLoad(t_position, coord, 0);
  let normal = textureLoad(t_normal, coord, 0);
  return albedo;
}
