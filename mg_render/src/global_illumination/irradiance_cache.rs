use crate::{graphics::Graphics, Vertex};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Entry {
    lifetime: u32,
    color: Vertex,
}

pub struct IrradianceCache {
    entries: Box<[Entry]>,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl IrradianceCache {
    pub fn bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
        graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,

                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("g bind group layout"),
            })
    }
    pub fn new(graphics: &Graphics) -> Self {
        let entries = vec![Entry::default(); 16 * 16 * 16 * 16].into_boxed_slice();
        let buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("prism buffer"),
                contents: bytemuck::cast_slice(&entries[..]),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &IrradianceCache::bind_group_layout(graphics),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                label: Some("geometry read bind group"),
            });
        Self {
            entries,
            buffer,
            bind_group,
        }
    }
}
