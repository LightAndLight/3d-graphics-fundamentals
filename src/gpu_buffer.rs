use wgpu::util::DeviceExt;

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

    pub fn init(
        device: &wgpu::Device,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        capacity: u32,
        contents: &[T],
    ) -> Self {
        let size = contents.len() as u32;
        assert!(size <= capacity);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            usage,
            contents: bytemuck::cast_slice(contents),
        });
        Self {
            buffer,
            capacity,
            size,
            _phantom_data: std::marker::PhantomData,
        }
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, data: T) -> u32 {
        assert!(self.size < self.capacity);

        let next_free_index = self.size;

        let next_size = self.size + 1;
        self.size = next_size;

        queue.write_buffer(
            &self.buffer,
            next_free_index as u64 * std::mem::size_of::<T>() as u64,
            bytemuck::cast_slice(&[data]),
        );

        next_free_index
    }

    pub fn update(&mut self, queue: &wgpu::Queue, index: u32, data: T) {
        assert!(index < self.size);

        queue.write_buffer(
            &self.buffer,
            index as u64 * std::mem::size_of::<T>() as u64,
            bytemuck::cast_slice(&[data]),
        );
    }

    pub fn update_slice(&mut self, queue: &wgpu::Queue, index: u32, data: &[T]) {
        assert!(index + (data.len() as u32) <= self.size);
        queue.write_buffer(&self.buffer, index as u64, bytemuck::cast_slice(data));
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
