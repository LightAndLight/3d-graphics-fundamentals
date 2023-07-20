pub struct GpuBuffer<T> {
    buffer: wgpu::Buffer,
    capacity: u32,
    size: u32,
    _phantom_data: std::marker::PhantomData<T>,
}

impl<T: Sized + bytemuck::Pod> GpuBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        capacity: u32,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: capacity as u64 * std::mem::size_of::<T>() as u64,
            usage,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            capacity,
            size: 0,
            _phantom_data: std::marker::PhantomData,
        }
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: T) -> u32 {
        let next_free_index = self.size;

        let next_size = self.size + 1;
        assert!(next_size < self.capacity);
        self.size = next_size;

        queue.write_buffer(
            &self.buffer,
            next_free_index as u64 * std::mem::size_of::<T>() as u64,
            bytemuck::cast_slice(&[data]),
        );

        next_free_index
    }

    pub fn remove(&mut self, _index: u32) {
        todo!()
    }

    pub fn as_raw_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> u32 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
