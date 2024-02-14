use crate::wgpu_ctx::WgpuContext;
use mg_core::*;
use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform([[f32; 4]; 4]);

pub struct Camera {
    pub eye: Point3f,
    pub target: Point3f,
    pub up: Vec3f,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    proj: Mat4,
    uniform: Uniform,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(w_ctx: &WgpuContext) -> Camera {
        let uniform = Uniform(Mat4::identity().into());
        let buffer = w_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = w_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &Camera::bind_group_layout(w_ctx),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("camera bind group"),
            });
        let mut cam = Camera {
            // +z is out of the screen
            eye: Point3f::new(20.0, 20.0, 20.0),
            target: Point3f::new(0.0, 0.0, 0.0),
            up: Vec3f::new(0.0, 1.0, 0.0),
            aspect: w_ctx.width as f32 / w_ctx.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            proj: Mat4::identity(),
            uniform,
            buffer,
            bind_group,
        };
        cam.resize(w_ctx);
        cam
    }
    pub fn resize(&mut self, w_ctx: &WgpuContext) {
        self.aspect = w_ctx.width as f32 / w_ctx.height as f32;
        self.proj = OPENGL_TO_WGPU_MATRIX
            * Mat4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
    }
    pub fn update(&mut self, w_ctx: &WgpuContext) {
        let view = Mat4::look_at_rh(&self.eye, &self.target, &self.up);
        self.uniform.0 = (self.proj * view).into();
        w_ctx
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
    pub fn bind_group_layout(w_ctx: &WgpuContext) -> wgpu::BindGroupLayout {
        w_ctx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera bind group layout"),
            })
    }
}
