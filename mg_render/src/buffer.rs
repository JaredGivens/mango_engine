pub struct Buffer {
    pub bin: Box<[u8]>,
    pub gpu_buffer: wgpu::Buffer,
}
