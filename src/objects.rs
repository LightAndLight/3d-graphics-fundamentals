use crate::{gpu_buffer::GpuBuffer, matrix::Matrix4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectId(pub u32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectData {
    pub transform: Matrix4,
}

pub struct Objects(GpuBuffer<ObjectData>);

impl Objects {
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        Objects(GpuBuffer::new(device, Some("objects"), capacity))
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: ObjectData) -> ObjectId {
        let index = self.0.insert(queue, data);
        ObjectId(index)
    }

    pub fn remove(&mut self, _object_id: ObjectId) {
        todo!()
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        self.0.as_raw_buffer()
    }
}
