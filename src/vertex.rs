use crate::{color::Color, point::Point3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Point3,
    pub color: Color,
}
