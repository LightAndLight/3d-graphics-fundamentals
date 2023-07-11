#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4 {
    value: [[f32; 4]; 4],
}

impl From<cgmath::Matrix4<f32>> for Matrix4 {
    fn from(value: cgmath::Matrix4<f32>) -> Self {
        Matrix4 {
            value: value.into(),
        }
    }
}
