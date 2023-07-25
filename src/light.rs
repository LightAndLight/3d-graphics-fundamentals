use crate::{color::Color, matrix::Matrix4, objects::ObjectId, shadow_map_atlas, vector::Vec3};

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
pub struct DirectionalLightGpu {
    pub color: Color,
    pub direction: Vec3,

    /// Measured in lux.
    pub illuminance: f32,

    pub shadow_view: Matrix4,
    pub shadow_projection: Matrix4,
    pub shadow_map_atlas_position: [f32; 2],
    pub shadow_map_atlas_size: [f32; 2],
}

pub struct DirectionalLight {
    pub directional_light_gpu_id: u32,
    pub shadow_map_atlas_entry: shadow_map_atlas::ShadowMapAtlasEntry,
}
