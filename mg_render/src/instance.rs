use std::mem::size_of;
use std::ops::Range;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Inst(pub [[f32; 4]; 4]);

impl Inst {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Inst>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Params {
    pub amt: u32,
    pub bin: Option<Box<[u8]>>,
    pub buffer: Option<wgpu::Buffer>,
    pub range: Option<[u32; 2]>,
}

pub struct Properties {
    pub amt: u32,
    pub bin: Option<Box<[u8]>>,
    pub buffer: Option<wgpu::Buffer>,
    pub(super) range: [u32; 2],
    pub(super) bind_group: wgpu::BindGroup,
}

impl Properties {
    pub fn range(&self) -> Range<u64> {
        self.range[0] as u64..self.range[1] as u64
    }
}
