use crate::{
    buffer::Buffer, geometry, geometry::Geometry, graphics::Graphics, material, material::Material,
    mesh::Mesh, texture::Texture,
};
use gltf::Gltf;
use mg_core::*;
use wgpu::util::DeviceExt;

enum ImgSrc<'a> {
    Slice(&'a [u8]),
    Array(Box<[u8]>),
}

fn parse_meshes(
    graphics: &Graphics,
    defaults: &material::Defaults,
    mesh: &gltf::Mesh,
    buffer: Arc<Buffer>,
    materials: &Vec<Arc<Material>>,
) -> Vec<Mesh> {
    mesh.primitives()
        .map(|p| {
            let vertex_view = p.get(&gltf::Semantic::Positions).unwrap().view().unwrap();
            let mut ranges = geometry::Ranges {
                vertex: [
                    vertex_view.offset() as u32,
                    (vertex_view.offset() + vertex_view.length()) as u32,
                ],
                index: [0, 0],
                uv: [0, 0],
            };

            let elm_amt = {
                if let Some(view) = p.indices().unwrap().view() {
                    ranges.index[0] = view.offset() as u32;
                    ranges.index[1] = (view.offset() + view.length()) as u32;
                    view.length()
                } else {
                    vertex_view.length()
                }
            } as u32;
            if let Some(uvs) = p.get(&gltf::Semantic::TexCoords(0)) {
                let view = uvs.view().unwrap();
                ranges.uv[0] = view.offset() as u32;
                ranges.uv[1] = (view.offset() + view.length()) as u32;
            }
            let geometry = Arc::new(Geometry {
                elm_amt,
                ranges,
                buffer: buffer.clone(),
                g_pipeline: None,
                ray_pipeline: None,
            });
            let material = p
                .material()
                .index()
                .map_or(defaults.material.clone(), |i| materials[i].clone());

            Mesh {
                name: mesh.name().unwrap_or("").to_string(),
                geometry,
                material,
            }
        })
        .collect()
}

fn parse_materials(
    graphics: &Graphics,
    defaults: &material::Defaults,
    doc: &gltf::Document,
    buffer: Arc<Buffer>,
    path: &str,
) -> Vec<Arc<Material>> {
    let img_sources: Vec<_> = doc
        .images()
        .map(|i| match i.source() {
            gltf::image::Source::View { view, .. } => {
                ImgSrc::Slice(&buffer.bin[view.offset()..view.offset() + view.length()])
            }
            gltf::image::Source::Uri { uri, .. } => {
                let filename = path.to_string() + uri;
                let bytes = read_file_to_end(filename.as_str());
                ImgSrc::Array(bytes.into_boxed_slice())
            }
        })
        .collect();
    let textures: Vec<_> = doc
        .textures()
        .map(|t| {
            Arc::new(Texture::create_image_texture(
                graphics,
                (path.to_string() + " img").as_str(),
                match &img_sources[t.source().index()] {
                    ImgSrc::Slice(slice) => slice,
                    ImgSrc::Array(array) => &array[..],
                },
            ))
        })
        .collect();
    doc.materials()
        .map(|m| {
            let albedo_tx = match m.pbr_metallic_roughness().base_color_texture() {
                Some(t) => textures[t.texture().index()].clone(),
                None => defaults.material.bindings.albedo_tx.clone(),
            };
            let emission_tx = match m.emissive_texture() {
                Some(t) => textures[t.texture().index()].clone(),
                None => defaults.material.bindings.emission_tx.clone(),
            };
            Arc::new(Material::new(
                graphics,
                m.name(),
                material::Bindings {
                    albedo_tx,
                    emission_tx,
                    ..defaults.material.bindings.clone()
                },
            ))
        })
        .collect()
}

// pub fn meshes_from_embeded(graphics: &Graphics, name: &str) {
//     let gltf = Gltf::open(name).expect(format!("file not found \"{:?}\"", name).as_str());
// }

pub fn meshes_from_separated(
    graphics: &Graphics,
    defaults: &material::Defaults,
    path: &str,
    name: &str,
) -> Vec<Mesh> {
    // Create GLTF file paths
    let file_path = format!("{}{}", path, name);
    let gltf_path = format!("{}.gltf", file_path);
    let gltf_bin_path = format!("{}.bin", file_path);

    // Load in GLTF data
    // Attempt to load the GLTF file
    let gltf = match Gltf::open(&gltf_path) {
        Ok(gltf) => gltf,
        Err(err) => {
            println!("Error opening GLTF file {}: {}", &gltf_path, err);
            return Vec::new();
        }
    };

    // Assuming read_file_to_end() returns a Result<Vec<u8>, Error>
    let bytes = read_file_to_end(&gltf_bin_path);
    let bin = bytes.into_boxed_slice();
    meshes_from_gltf(graphics, defaults, &gltf, bin, path, name)
}

//let f = std::fs::File::open(name).unwrap();
//let reader = std::io::BufReader::new(f);
//let glb = Glb::from_reader(reader).unwrap();
//let gltf = Gltf::from_slice_without_validation(glb.json.as_ref()).unwrap();
//let bin = glb.bin.unwrap().into_owned().into_boxed_slice();

fn meshes_from_gltf(
    graphics: &Graphics,
    defaults: &material::Defaults,
    gltf: &Gltf,
    bin: Box<[u8]>,
    path: &str,
    name: &str,
) -> Vec<Mesh> {
    let gpu_buffer = graphics
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} buffer", name).as_str()),
            contents: bin.as_ref(),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE,
        });

    let texture_buffer = Arc::new(Buffer { bin, gpu_buffer });
    let mesh_buffer = texture_buffer.clone();
    let materials = parse_materials(graphics, defaults, &gltf.document, texture_buffer, path);
    gltf.scenes()
        .map(|scene| {
            scene
                .nodes()
                .filter_map(|node| match node.mesh() {
                    Some(mesh) => Some(
                        parse_meshes(graphics, defaults, &mesh, mesh_buffer.clone(), &materials)
                            .into_iter(),
                    ),
                    None => None,
                })
                .flatten()
        })
        .flatten()
        .collect()
}
