use crate::gpu_buffer::GpuBuffer;

pub struct GpuVariable<T> {
    buffer: GpuBuffer<T>,
}

impl<T: Sized + bytemuck::Pod> GpuVariable<T> {
    pub fn new(
        device: &wgpu::Device,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        value: T,
    ) -> Self {
        Self {
            buffer: GpuBuffer::init(device, label, usage, 1, &[value]),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, value: T) {
        self.buffer.update(queue, 0, value);
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        self.buffer.as_raw_buffer()
    }
}
