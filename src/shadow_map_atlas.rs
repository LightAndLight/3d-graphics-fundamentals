use crate::vector::Vec2;

const ENTRY_SIZE: u32 = 1024;
const ENTRY_SIZE_F32: f32 = 1024.0;

pub struct ShadowMapAtlas {
    texture_format: wgpu::TextureFormat,
    capacity_x: u32,
    capacity_y: u32,
    size_x: u32,
    size_y: u32,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl ShadowMapAtlas {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat, size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_map_atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_map_atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            texture_format,
            capacity_x: size,
            capacity_y: size,
            size_x: 0,
            size_y: 0,
            texture,
            texture_view,
            sampler,
        }
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn allocate(&mut self) -> ShadowMapAtlasEntry {
        if self.capacity_x - self.size_x >= ENTRY_SIZE {
            self.size_x += ENTRY_SIZE;
        } else if self.capacity_y - self.size_y >= ENTRY_SIZE {
            self.size_x = 0;
            self.size_y += ENTRY_SIZE;
        } else {
            panic!(
                "failed to allocate {}x{} region in shadow map. only {}x{} remains.",
                ENTRY_SIZE,
                ENTRY_SIZE,
                self.capacity_x - self.size_x,
                self.capacity_y - self.size_y
            );
        }

        ShadowMapAtlasEntry {
            position: Vec2 {
                x: self.size_x as f32,
                y: self.size_y as f32,
            },
        }
    }
}

pub struct ShadowMapAtlasEntry {
    position: Vec2,
}

impl ShadowMapAtlasEntry {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn size(&self) -> f32 {
        ENTRY_SIZE_F32
    }
}
