use crate::{color::Color, objects::ObjectId, shadow_map_atlas, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightGpu {
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
    pub shadow_map_light_ids: ShadowMapLightIds,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]

pub struct ShadowMapLightIds {
    pub x: u32,
    pub neg_x: u32,
    pub y: u32,
    pub neg_y: u32,
    pub z: u32,
    pub neg_z: u32,
}

pub struct PointLight {
    pub shadow_map_faces: PointLightShadowMapFaces,
}

pub struct PointLightShadowMapFaces {
    pub x: PointLightShadowMapFace,
    pub neg_x: PointLightShadowMapFace,
    pub y: PointLightShadowMapFace,
    pub neg_y: PointLightShadowMapFace,
    pub z: PointLightShadowMapFace,
    pub neg_z: PointLightShadowMapFace,
}

#[derive(Clone, Copy)]
pub struct PointLightShadowMapFace {
    pub shadow_map_light_gpu_id: u32,
    pub shadow_map_atlas_entry: shadow_map_atlas::ShadowMapAtlasEntry,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightGpu {
    pub color: Color,
    pub direction: Vec3,

    /// Measured in lux.
    pub illuminance: f32,

    pub shadow_map_light_id: u32,
}

pub struct DirectionalLight {
    pub shadow_map_light_gpu_id: u32,
    pub shadow_map_atlas_entry: shadow_map_atlas::ShadowMapAtlasEntry,
}
