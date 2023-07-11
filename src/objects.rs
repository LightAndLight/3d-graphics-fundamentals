use crate::matrix::Matrix4;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectId(pub u32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectData {
    pub transform: Matrix4,
}

pub struct Objects {
    buffer: wgpu::Buffer,
    capacity: u32,
    size: u32,
}

impl Objects {
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("objects"),
            size: capacity as u64 * std::mem::size_of::<ObjectData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            capacity,
            size: 0,
        }
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: ObjectData) -> ObjectId {
        let next_free_index = self.size;
        let object_id = ObjectId(next_free_index);

        let next_size = self.size + 1;
        assert!(next_size < self.capacity);
        self.size = next_size;

        queue.write_buffer(
            &self.buffer,
            next_free_index as u64 * std::mem::size_of::<ObjectData>() as u64,
            bytemuck::cast_slice(&[data]),
        );

        object_id
    }

    pub fn remove(&mut self, _object_id: ObjectId) {
        todo!()
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
