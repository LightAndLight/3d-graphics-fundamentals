use crate::{material::MaterialId, model_matrices::ModelMatrixId, point::Point3, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Point3,
    pub model_matrix_id: ModelMatrixId,
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
            // model_matrix_id: ModelMatrixId
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: std::mem::size_of::<Point3>() as u64,
                shader_location: 1,
            },
            // normal: Vec3
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: std::mem::size_of::<Point3>() as u64
                    + std::mem::size_of::<ModelMatrixId>() as u64,
                shader_location: 2,
            },
            // material_id: MaterialId
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: std::mem::size_of::<Point3>() as u64
                    + std::mem::size_of::<ModelMatrixId>() as u64
                    + std::mem::size_of::<Vec3>() as u64,
                shader_location: 3,
            },
        ],
    };
}
