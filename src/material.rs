use crate::{color::Color, gpu_buffer::GpuBuffer};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialId(pub u32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    pub color: Color,
    pub roughness: f32,
    pub _padding: [u32; 3],
}

pub struct Materials(GpuBuffer<Material>);

impl Materials {
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        Materials(GpuBuffer::new(device, Some("materials"), capacity))
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: Material) -> MaterialId {
        let index = self.0.insert(queue, data);
        MaterialId(index)
    }

    pub fn remove(&mut self, _material_id: MaterialId) {
        todo!()
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        self.0.as_raw_buffer()
    }
}
