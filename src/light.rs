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

    /**
    The distance at which the light begins to attenuate.

    A perfectly diffuse white surface that is directly facing the point light at
    `intensity` units away appears `color`. Closer than `intensity` it appears brighter
    than this, and farther than `intensity` it appears darker.

    Non-physical based quantity.
    */
    pub intensity: f32,
    pub _padding1: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: Vec3,
    pub _padding: u32,
}
