use crate::{wgpu_ctx::WgpuContext, texture::Texture, texture_bind_group_layout};
use mg_core::*;

#[derive(Clone)]
pub struct Bindings {
    pub albedo_tx: Arc<Texture>,
    pub albedo_sampler: Arc<wgpu::Sampler>,
    pub emission_tx: Arc<Texture>,
    pub emission_sampler: Arc<wgpu::Sampler>,
}

pub struct Material {
    pub bindings: Bindings,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(w_ctx: &WgpuContext, name: Option<&str>, bindings: Bindings) -> Material {
        let bind_group = w_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout(w_ctx),
                label: name,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&bindings.albedo_tx.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&bindings.albedo_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&bindings.emission_tx.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&bindings.emission_sampler),
                    },
                ],
            });
        Material {
            bindings,
            bind_group,
        }
    }
    //address_mode_u: wgpu::AddressMode::Repeat,
    //address_mode_v: wgpu::AddressMode::Repeat,
    //address_mode_w: wgpu::AddressMode::Repeat,
    //mag_filter: wgpu::FilterMode::Nearest,
    //min_filter: wgpu::FilterMode::Nearest,
    //mipmap_filter: wgpu::FilterMode::Nearest,
    //..Default::default()
    //},
}

pub struct Defaults {
    pub material: Arc<Material>,
}

impl Defaults {
    pub fn new(w_ctx: &WgpuContext) -> Self {
        let albedo_tx = Arc::new(Texture::create_image_texture(
            &w_ctx,
            "default albedo",
            include_bytes!("textures/defaultA.png"),
        ));
        let emission_tx = Arc::new(Texture::create_image_texture(
            &w_ctx,
            "default emission",
            include_bytes!("textures/defaultA.png"),
        ));
        let default_sampler = Arc::new(
            w_ctx
                .device
                .create_sampler(&wgpu::SamplerDescriptor::default()),
        );
        let bindings = Bindings {
            albedo_tx,
            albedo_sampler: default_sampler.clone(),
            emission_tx,
            emission_sampler: default_sampler.clone(),
        };
        let material = Arc::new(Material::new(w_ctx, Some("default"), bindings));
        Self { material }
    }
}
