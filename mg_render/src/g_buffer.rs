use crate::{
    graphics::Graphics, texture::Texture, TX_FORMAT_COLOR, TX_FORMAT_DEPTH, TX_FORMAT_NORMAL,
    TX_FORMAT_POSITION,
};

pub fn write_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // rgba albedo
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_COLOR,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // emission
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_COLOR,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // position
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_POSITION,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // normal
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TX_FORMAT_NORMAL,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: Some("g bind group layout"),
        })
}
pub fn read_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // rgba albedo
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // emission
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // position
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // normal
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: Some("g bind group layout"),
        })
}

pub struct GBuffer {
    pub albedo_texture: Texture,
    pub emission_texture: Texture,
    pub position_texture: Texture,
    pub normal_texture: Texture,
    pub depth_texture: Texture,
    pub read_bind_group: wgpu::BindGroup,
    pub write_bind_group: wgpu::BindGroup,
}

impl GBuffer {
    pub fn new(graphics: &Graphics) -> Self {
        let g_usage = wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING;
        let albedo_texture =
            Texture::create_texture(graphics, "g albedo texture", TX_FORMAT_COLOR, g_usage);
        let emission_texture =
            Texture::create_texture(graphics, "g emission texture", TX_FORMAT_COLOR, g_usage);
        let position_texture =
            Texture::create_texture(graphics, "g position texture", TX_FORMAT_POSITION, g_usage);
        let normal_texture =
            Texture::create_texture(graphics, "g normal texture", TX_FORMAT_NORMAL, g_usage);
        let depth_texture = Texture::create_texture(
            graphics,
            "depth texture",
            TX_FORMAT_DEPTH,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let read_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &read_bind_group_layout(graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&emission_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&position_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                    },
                ],
                label: Some("g read bind group"),
            });
        let write_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &write_bind_group_layout(graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&emission_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&position_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                    },
                ],
                label: Some("g write bind group"),
            });
        Self {
            albedo_texture,
            emission_texture,
            position_texture,
            normal_texture,
            depth_texture,
            read_bind_group,
            write_bind_group,
        }
    }
}
