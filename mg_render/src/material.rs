use crate::{graphics::Graphics, texture::Texture, texture_bind_group_layout};

pub struct Params {
    pub name: String,
    pub albedo_texture: Texture,
    pub albedo_sampler: Option<wgpu::Sampler>,
    pub emission_texture: Texture,
    pub emission_sampler: Option<wgpu::Sampler>,
}

pub struct Material {
    pub albedo_texture: Texture,
    pub albedo_sampler: wgpu::Sampler,
    pub emission_texture: Texture,
    pub emission_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(graphics: &Graphics, p: Params) -> Material {
        let albedo_texture = p.albedo_texture;
        let albedo_sampler =
            p.albedo_sampler
                .unwrap_or(graphics.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                }));

        let emission_texture = p.emission_texture;
        let emission_sampler = p.emission_sampler.unwrap_or(graphics.device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        ));
        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout(graphics),
                label: Some(format!("{} bind group", p.name).as_str()),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&albedo_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&emission_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&emission_sampler),
                    },
                ],
            });
        Material {
            albedo_texture,
            albedo_sampler,
            emission_texture,
            emission_sampler,
            bind_group,
        }
    }
}
