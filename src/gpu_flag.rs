use crate::gpu_variable::GpuVariable;

/// A boolean [`GpuVariable`]. `wgpu` doesn't support boolean uniforms, so we use [`u32`]s internally.
pub struct GpuFlag {
    buffer: GpuVariable<u32>,
}

impl GpuFlag {
    pub fn new(
        device: &wgpu::Device,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        value: bool,
    ) -> Self {
        Self {
            buffer: GpuVariable::new(device, label, usage, if value { 1 } else { 0 }),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, value: bool) {
        self.buffer.update(queue, if value { 1 } else { 0 });
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        self.buffer.as_raw_buffer()
    }
}
