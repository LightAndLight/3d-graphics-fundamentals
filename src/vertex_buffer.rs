use crate::vertex::Vertex;

pub struct VertexBuffer {
    /// Handle to the underling GPU buffer.
    buffer: wgpu::Buffer,

    /// Maximum number of vertices that can be stored.
    capacity: u64,

    /// Current number of stored vertices
    size: u64,
}

impl VertexBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("VertexBuffer"),
            size: capacity * std::mem::size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity,
            size: 0,
        }
    }

    pub fn insert(&mut self, queue: &wgpu::Queue, vertex: Vertex) {
        assert!(self.size + 1 < self.capacity);
        queue.write_buffer(
            &self.buffer,
            self.size * std::mem::size_of::<Vertex>() as u64,
            bytemuck::cast_slice(&[vertex]),
        );
        self.size += 1;
    }

    pub fn insert_many(&mut self, queue: &wgpu::Queue, vertices: &[Vertex]) {
        let vertices_len = vertices.len() as u64;
        assert!(self.size + vertices_len < self.capacity);
        queue.write_buffer(
            &self.buffer,
            self.size * std::mem::size_of::<Vertex>() as u64,
            bytemuck::cast_slice(vertices),
        );
        self.size += vertices_len;
    }

    pub fn as_raw_slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..)
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
