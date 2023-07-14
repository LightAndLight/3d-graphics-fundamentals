use crate::{material::MaterialId, objects::ObjectId, point::Point3, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Point3,
    pub object_id: ObjectId,
    pub normal: Vec3,
    pub material_id: MaterialId,
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // position: Point3
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            // object_id: ObjectId
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: std::mem::size_of::<Point3>() as u64,
                shader_location: 1,
            },
            // normal: Vec3
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: std::mem::size_of::<Point3>() as u64
                    + std::mem::size_of::<ObjectId>() as u64,
                shader_location: 2,
            },
            // material_id: MaterialId
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: std::mem::size_of::<Point3>() as u64
                    + std::mem::size_of::<ObjectId>() as u64
                    + std::mem::size_of::<Vec3>() as u64,
                shader_location: 3,
            },
        ],
    };
}
