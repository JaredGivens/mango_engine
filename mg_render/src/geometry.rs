use crate::{buffer::Buffer, graphics::Graphics};
use gltf::{Glb, Gltf};
use mg_core::*;
use std::ops::Range;
use wgpu::util::DeviceExt;

#[repr(C)]
pub struct Ranges {
    pub index: [u32; 2],
    pub vertex: [u32; 2],
    pub uv: [u32; 2],
}

impl Ranges {
    pub fn index(&self) -> Range<u64> {
        self.index[0] as u64..self.index[1] as u64
    }
    pub fn vertex(&self) -> Range<u64> {
        self.vertex[0] as u64..self.vertex[1] as u64
    }
    pub fn uv(&self) -> Range<u64> {
        self.uv[0] as u64..self.uv[1] as u64
    }
}

pub struct Geometry {
    pub elm_amt: u32,
    pub ranges: Ranges,
    pub buffer: Arc<Buffer>,
}

impl Geometry {
    pub fn from_gltf(graphics: &Graphics, name: &str) {}

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
}
