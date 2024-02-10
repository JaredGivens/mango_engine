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
    pub g_pipeline: Option<wgpu::RenderPipeline>,
    pub ray_pipeline: Option<wgpu::ComputePipeline>,
}
