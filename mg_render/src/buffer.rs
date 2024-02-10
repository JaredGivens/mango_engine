struct Buffer {
    pub bin: Box<[u8]>,
    pub buffer: wgpu::Buffer,
}
