use crate::{color::Color, objects::ObjectId, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    pub object_id: ObjectId,

    /**
    See:

    * <https://sotrh.github.io/learn-wgpu/showcase/alignment/>
    * <https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-blinn-phong-model>
    */
    pub _padding0: [u32; 3],

    pub color: Color,

    /// Measured in lumens.
    pub luminous_power: f32,
    pub _padding1: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: Vec3,

    /// Measured in lux.
    pub illuminance: f32,
}
