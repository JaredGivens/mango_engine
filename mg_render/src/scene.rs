use crate::{
    camera::Camera, graphics::Graphics, instance, mesh, mesh::Mesh, ray_buffer::RayBuffer, Vertex,
};
use mg_core::*;
use std::collections::VecDeque;
use std::mem::size_of;
use wgpu::util::DeviceExt;

struct LocalNode {
    pub child_count: usize,
    pub tsf: Mat4,
    pub mesh: usize,
    pub dirty: bool,
}

struct WorldNode {
    pub child_count: usize,
    pub tsf: Mat4,
    pub dirty: bool,
}

impl WorldNode {
    pub fn from_local(local: &LocalNode) -> WorldNode {
        WorldNode {
            child_count: local.child_count,
            tsf: local.tsf,
            dirty: local.dirty,
        }
    }
    pub fn add_local(&self, local: &LocalNode) -> WorldNode {
        WorldNode {
            child_count: local.child_count,
            tsf: self.tsf * local.tsf,
            dirty: self.dirty || local.dirty,
        }
    }
}

pub struct Scene {
    pub camera: Camera,
    //pub collections: Vec<geometry::Collection>,
    pub background: Vertex,
    pub root_count: usize,
    world_deque: VecDeque<WorldNode>,
    nodes: Vec<LocalNode>,
    pub meshes: Vec<Mesh>,
    pub inst_props: Vec<instance::Properties>,
    pub ray_buffer: RayBuffer,
    accel_struct: Box<[u32]>,
    pub accel_struct_buffer: wgpu::Buffer,
}

impl Scene {
    pub fn new(graphics: &Graphics) -> Scene {
        let camera = Camera::new(graphics);
        let background = Vertex([135.0 / 255.0, 206.0 / 255.0, 235.0 / 255.0]);
        let accel_struct = vec![0; 65536].into_boxed_slice();
        let accel_struct_buffer =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("accel struct buffer"),
                    contents: bytemuck::cast_slice(accel_struct.as_ref()),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        Scene {
            background,
            camera,
            root_count: 0,
            world_deque: VecDeque::with_capacity(128),
            nodes: Vec::with_capacity(1024),
            ray_buffer: RayBuffer::new(graphics, &accel_struct_buffer, 0),
            meshes: vec![],
            inst_props: vec![],
            accel_struct,
            accel_struct_buffer,
        }
    }
    pub fn update(&mut self) {
        self.world_deque.clear();
        for i in 0..self.root_count {
            let node = &self.nodes[i];
            if node.dirty {
                self.ray_buffer.world_tsfs[i].0 = node.tsf.into();
            }
            if node.child_count != 0 {
                self.world_deque.push_back(WorldNode::from_local(node));
            }
        }
        let mut offset = self.root_count;
        while let Some(parent) = self.world_deque.pop_front() {
            offset = self.update_children(offset, parent);
        }
    }

    fn update_children(&mut self, offset: usize, parent: WorldNode) -> usize {
        for i in offset..offset + parent.child_count {
            let node = &self.nodes[i];
            let world = parent.add_local(node);
            if world.dirty {
                self.ray_buffer.world_tsfs[i].0 = world.tsf.into();
            }
            if node.child_count != 0 {
                self.world_deque.push_back(world)
            }
        }
        offset + parent.child_count
    }
    pub fn instantiate_mesh(
        &mut self,
        graphics: &Graphics,
        mesh: Mesh,
        inst_param: instance::Params,
    ) -> usize {
        let ranges_uniform =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("geometry buffer"),
                    usage: wgpu::BufferUsages::UNIFORM,
                    contents: as_u8_slice(&mesh.geometry.ranges),
                });
        let range = match inst_param.range {
            Some(range) => range,
            None => {
                let size = size_of::<instance::Inst>() as u32;
                let start = self.ray_buffer.world_tsfs.len() as u32 * size;
                [start, start + inst_param.amt * size]
            }
        };
        let inst_range_uniform =
            graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("geometry buffer"),
                    usage: wgpu::BufferUsages::UNIFORM,
                    contents: as_u8_slice(&range),
                });
        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(format!("{} bind group", mesh.name).as_str()),
                layout: &mesh::bind_group_layout(&graphics),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: inst_range_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: ranges_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: mesh.geometry.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &mesh.material.emission_texture.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&mesh.material.emission_sampler),
                    },
                ],
            });
        self.inst_props.push(instance::Properties {
            amt: inst_param.amt,
            bin: inst_param.bin,
            buffer: inst_param.buffer,
            range,
            bind_group,
        });
        log::warn!("{}", range[1]);
        self.meshes.push(mesh);
        self.resize(graphics);
        self.meshes.len()
    }
    pub fn resize(&mut self, graphics: &Graphics) {
        self.camera.resize(graphics);
        self.ray_buffer = RayBuffer::new(
            graphics,
            &self.accel_struct_buffer,
            self.inst_props.last().map_or(0, |ip| ip.range[1]) as usize,
        );
    }
}
