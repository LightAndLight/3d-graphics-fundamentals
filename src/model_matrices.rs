use crate::{gpu_buffer::GpuBuffer, matrix::Matrix4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelMatrixId(pub u32);

pub struct ModelMatrices(GpuBuffer<Matrix4>);

impl ModelMatrices {
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        ModelMatrices(GpuBuffer::new(
            device,
            Some("model_matrices"),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            capacity,
        ))
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: Matrix4) -> ModelMatrixId {
        let index = self.0.insert(queue, data);
        ModelMatrixId(index)
    }

    pub fn update(&mut self, queue: &wgpu::Queue, id: ModelMatrixId, value: Matrix4) {
        self.0.update(queue, id.0, value)
    }

    pub fn remove(&mut self, _object_id: ModelMatrixId) {
        todo!()
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        self.0.as_raw_buffer()
    }
}
