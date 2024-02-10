use crate::{graphics::Graphics, Mesh};

fn parse_mesh(mesh: gltf::Mesh) -> Vec<Mesh> {
    mesh.primitives().map(|p| {
        let vertex_view = p.get(&gltf::Semantic::Positions).unwrap().view().unwrap();
        let mut ranges = Ranges {
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
    });
}

pub fn from_glb(graphics: &Graphics, name: &str) -> Geometry {
    let f = std::fs::File::open(name).unwrap();
    let reader = std::io::BufReader::new(f);
    let glb = Glb::from_reader(reader).unwrap();
    let gltf = Gltf::from_slice_without_validation(glb.json.as_ref()).unwrap();

    let bin = glb.bin.unwrap().into_owned().into_boxed_slice();
    let buffer = graphics
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} buffer", name).as_str()),
            contents: bin.as_ref(),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::STORAGE,
        });
    let mesh = gltf
        .scenes()
        .next()
        .unwrap()
        .nodes()
        .find(|n| n.mesh().is_some())
        .unwrap()
        .mesh()
        .unwrap();

    log::warn!("primitives {}", mesh.primitives().len());
    let primitive = mesh.primitives().next().unwrap();

    log::info!("{:?}", primitive);
    if let Some(view) = primitive.get(&gltf::Semantic::TexCoords(2)).unwrap().view() {
        ranges.uv[0] = view.offset() as u32;
        ranges.uv[1] = (view.offset() + view.length()) as u32;
    }

    Geometry {
        elm_amt,
        ranges,
        buffer,
        bin,
        g_pipeline: None,
        ray_pipeline: None,
    }
}
pub fn load_gltf(graphics: &Graphics, name: &str) {
    let gltf = Gltf::open(name).expect(format!("file not found \"{:?}\"", name).as_str());
    gltf.scenes().for_each(|scene| {
        scene.nodes().for_each(|node| {
            if let Some(mesh) = node.mesh() {
                parse_meshes(graphics, mesh);
            }
        })
    })
}
