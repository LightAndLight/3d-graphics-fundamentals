use crate::{color::Color, objects::ObjectId, point::Point3, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Point3,
    pub color: Color,
    pub object_id: ObjectId,
    pub normal: Vec3,
}
