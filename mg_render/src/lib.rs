#[macro_use]
// Render
pub mod camera;
pub mod buffer;
pub mod g_buffer;
pub mod geometry;
pub mod gltf_loader;
pub mod graphics;
pub mod instance;

use camera::Camera;
use g_buffer::GBuffer;
use graphics::Graphics;
use instance::Inst;

// Model data
pub mod mesh;
pub mod material;
pub mod scene;
pub mod texture;

// Global Illumination
pub mod global_illumination;
use global_illumination::irradiance_cache::IrradianceCache;
use crate::global_illumination::ray_buffer;

// Core
use mg_core::*;
use scene::Scene;
use std::mem::size_of;

pub const TX_FORMAT_COLOR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const TX_FORMAT_POSITION: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const TX_FORMAT_NORMAL: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const TX_FORMAT_DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex([f32; 3]);
impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UV([f32; 2]);

impl UV {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<UV>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![1 => Float32x2],
        }
    }
}

pub fn background_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
}
pub fn texture_bind_group_layout(graphics: &Graphics) -> wgpu::BindGroupLayout {
    graphics
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                // albedo texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // emission texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
}

pub struct Renderer {
    g_buffer: GBuffer,
    irradiance_cache: IrradianceCache,
    g_pipeline: wgpu::RenderPipeline,
    ray_pipeline: wgpu::ComputePipeline,
    comp_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(graphics: &Graphics) -> Renderer {
        let g_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("geometry pipeline layout"),
                    bind_group_layouts: &[
                        &Camera::bind_group_layout(graphics),
                        &texture_bind_group_layout(graphics),
                    ],
                    push_constant_ranges: &[],
                });

        let g_shader = graphics
            .device
            .create_shader_module(wgpu::include_wgsl!("shader/geometry.wgsl"));
        let g_pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("geometry pipeline"),
                layout: Some(&g_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &g_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout(), UV::layout(), Inst::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &g_shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: TX_FORMAT_COLOR,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: TX_FORMAT_COLOR,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: TX_FORMAT_POSITION,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: TX_FORMAT_NORMAL,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TX_FORMAT_DEPTH,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        let ray_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ray pipeline layout"),
                    bind_group_layouts: &[
                        &ray_buffer::read_bind_group_layout(graphics),
                        &mesh::bind_group_layout(graphics),
                        &g_buffer::read_bind_group_layout(graphics),
                        &g_buffer::write_bind_group_layout(graphics),
                    ],
                    push_constant_ranges: &[],
                });
        let ray_shader = graphics
            .device
            .create_shader_module(wgpu::include_wgsl!("shader/raytrace.wgsl"));
        let ray_pipeline =
            graphics
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ray pipeline"),
                    layout: Some(&ray_pipeline_layout),
                    module: &ray_shader,
                    entry_point: "main",
                });

        let comp_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("composition pipeline layout"),
                    bind_group_layouts: &[
                        &g_buffer::read_bind_group_layout(graphics),
                        &IrradianceCache::bind_group_layout(graphics),
                    ],
                    push_constant_ranges: &[],
                });
        let comp_shader = graphics
            .device
            .create_shader_module(wgpu::include_wgsl!("shader/composition.wgsl"));
        let comp_pipeline =
            graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("composition pipeline"),
                    layout: Some(&comp_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &comp_shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &comp_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: graphics.tx_format_surface,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });
        let irradiance_cache = IrradianceCache::new(graphics);

        Renderer {
            g_buffer: GBuffer::new(graphics),
            ray_pipeline,
            g_pipeline,
            irradiance_cache,
            comp_pipeline,
        }
    }

    pub fn resize(&mut self, graphics: &Graphics) {
        if graphics.width <= 0 && graphics.height <= 0 {
            return;
        }
        self.g_buffer = GBuffer::new(graphics);
    }

    pub fn ray_pass(&mut self, encoder: &mut wgpu::CommandEncoder, scene: &mut Scene) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("ray pass"),
            timestamp_writes: None,
        });
        scene
            .meshes
            .iter()
            .zip(scene.inst_props.iter())
            .for_each(|(mesh, inst_prop)| {
                match &mesh.geometry.ray_pipeline {
                    Some(pipeline) => compute_pass.set_pipeline(&pipeline),
                    None => compute_pass.set_pipeline(&self.ray_pipeline),
                }
                compute_pass.set_bind_group(0, &scene.ray_buffer.read_bind_group, &[]);
                compute_pass.set_bind_group(1, &inst_prop.bind_group, &[]);
                compute_pass.set_bind_group(
                    2,
                    &scene.ray_buffer.g_buffers[scene.ray_buffer.g_buffer_ind].read_bind_group,
                    &[],
                );
                compute_pass.set_bind_group(
                    3,
                    &scene.ray_buffer.g_buffers[scene.ray_buffer.g_buffer_ind ^ 1].write_bind_group,
                    &[],
                );
                scene.ray_buffer.g_buffer_ind ^= 1;

                compute_pass.dispatch_workgroups(1, 1, 1);
            });
    }
    pub fn geometry_pass(&mut self, encoder: &mut wgpu::CommandEncoder, scene: &Scene) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("g pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.g_buffer.albedo_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.g_buffer.emission_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.g_buffer.position_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.g_buffer.normal_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.g_buffer.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        scene
            .meshes
            .iter()
            .zip(scene.inst_props.iter())
            .for_each(|(mesh, inst_prop)| {
                match &mesh.geometry.g_pipeline {
                    Some(pipeline) => render_pass.set_pipeline(&pipeline),
                    None => render_pass.set_pipeline(&self.g_pipeline),
                }
                render_pass.set_bind_group(0, &scene.camera.bind_group, &[]);
                render_pass.set_bind_group(1, &mesh.material.bind_group, &[]);
                render_pass.set_vertex_buffer(
                    0,
                    mesh.geometry
                        .buffer
                        .gpu_buffer
                        .slice(mesh.geometry.ranges.vertex()),
                );
                if let Some(ib) = &inst_prop.buffer {
                    render_pass.set_vertex_buffer(1, ib.slice(inst_prop.range()));
                } else {
                    render_pass.set_vertex_buffer(
                        1,
                        scene.ray_buffer.world_tsfs_buffer.slice(inst_prop.range()),
                    );
                }
                render_pass.set_vertex_buffer(
                    2,
                    mesh.geometry
                        .buffer
                        .gpu_buffer
                        .slice(mesh.geometry.ranges.uv()),
                );

                //if mesh.geometry.ranges.index() {
                render_pass.set_index_buffer(
                    mesh.geometry
                        .buffer
                        .gpu_buffer
                        .slice(mesh.geometry.ranges.index()),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..mesh.geometry.elm_amt, 0, 0..inst_prop.amt);
                //} else {
                //render_pass.draw(0..mesh.geometry.elm_amt, 0..mesh.inst_amt);
                //}
            });
    }
    pub fn compose_pass<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.comp_pipeline);
        render_pass.set_bind_group(0, &self.g_buffer.read_bind_group, &[]);
        render_pass.set_bind_group(1, &self.irradiance_cache.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
