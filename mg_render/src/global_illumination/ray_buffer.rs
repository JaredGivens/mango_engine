use crate::{
    g_buffer::GBuffer, graphics::Graphics, instance::Inst, scene::Scene, texture::Texture,
    TX_FORMAT_NORMAL, TX_FORMAT_POSITION,
};
use mg_core::*;
use wgpu::util::DeviceExt;

pub fn accel_struct_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("accel struct bind group layout"),
        })
}

pub fn read_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ray read bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        })
}

pub fn write_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ray write bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_POSITION,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_NORMAL,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        })
}

pub struct RayBuffer {
    pub g_buffer_ind: usize,
    pub g_buffers: [GBuffer; 2],
    pub read_bind_group: wgpu::BindGroup,
    pub write_bind_group: wgpu::BindGroup,
    origin_texture: Texture,
    direction_texture: Texture,

    pub world_tsfs: Box<[Inst]>,
    pub world_tsfs_buffer: wgpu::Buffer,
}

impl RayBuffer {
    pub fn new(graphics: &Graphics, accel_struct_buffer: &wgpu::Buffer, amt: usize) -> RayBuffer {
        let ray_usage = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
        let origin_texture = Texture::create_texture(
            &graphics,
            "ray origin testure",
            TX_FORMAT_POSITION,
            ray_usage,
        );
        let direction_texture = Texture::create_texture(
            &graphics,
            "ray direction testure",
            TX_FORMAT_NORMAL,
            ray_usage,
        );
        let write_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ray write bind group"),
                layout: &write_bind_group_layout(&graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&origin_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&direction_texture.view),
                    },
                ],
            });
        let world_tsfs =
            vec![Inst(Mat4::identity().into()); if amt != 0 { amt } else { 1 }].into_boxed_slice();
        let world_tsfs_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("world instances buffer"),
                    contents: bytemuck::cast_slice(&world_tsfs[..]),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                });
        let read_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ray read bind group"),
                layout: &read_bind_group_layout(&graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&origin_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&direction_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: accel_struct_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: world_tsfs_buffer.as_entire_binding(),
                    },
                ],
            });
        RayBuffer {
            g_buffer_ind: 0,
            g_buffers: [GBuffer::new(&graphics), GBuffer::new(&graphics)],
            origin_texture,
            direction_texture,
            read_bind_group,
            write_bind_group,
            world_tsfs,
            world_tsfs_buffer,
        }
    }
}
