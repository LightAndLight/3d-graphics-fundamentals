use wgpu::include_wgsl;

use crate::{
    gpu_buffer::GpuBuffer, matrix::Matrix4, objects::Objects, vector::Vec2, vertex::Vertex,
    vertex_buffer::VertexBuffer,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLight {
    pub view: Matrix4,
    pub projection: Matrix4,
    pub shadow_map_atlas_position: [f32; 2],
    pub shadow_map_atlas_size: [f32; 2],
}

pub struct ShadowMapAtlasEntry {
    pub shadowing_directional_light_id: u32,
    pub position: Vec2,
    pub size: f32,
}

pub struct ShadowMaps {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub shader_module: wgpu::ShaderModule,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl ShadowMaps {
    pub fn new(
        device: &wgpu::Device,
        shadow_map_atlas_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let shader_module = device.create_shader_module(include_wgsl!("shadow_maps.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_maps_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_maps_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[Vertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: shadow_map_atlas_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            bind_group_0,
            bind_group_layout_0,
            shader_module,
            pipeline_layout,
            render_pipeline,
        }
    }

    pub fn record(
        &self,
        command_encoder: &mut wgpu::CommandEncoder,
        shadow_map_atlas: &wgpu::TextureView,
        // point_light_entries: &GpuBuffer<ShadowMapAtlasEntry>,
        directional_light_entries: &[ShadowMapAtlasEntry],
        vertex_buffer: &VertexBuffer,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow_maps_pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_map_atlas,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.as_raw_slice());

        for entry in directional_light_entries.iter() {
            render_pass.set_viewport(
                entry.position.x,
                entry.position.y,
                entry.size,
                entry.size,
                0.0,
                1.0,
            );
            render_pass.set_bind_group(
                0,
                &self.bind_group_0,
                &[entry.shadowing_directional_light_id],
            );
            render_pass.draw(0..vertex_buffer.len() as u32, 0..1);
        }
    }
}

pub struct BindGroup0<'a> {
    pub directional_lights: &'a GpuBuffer<DirectionalLight>,
    pub objects: &'a Objects,
}

impl<'a> BindGroup0<'a> {
    pub fn create(&self, device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        // @group(0) @binding(0)
        // var<uniform> directional_light: DirectionalLight;
        let directional_light = (
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.directional_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(1)
        // var<storage, read> objects: array<ObjectData>;
        let objects = (
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.objects.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_maps_bind_group_layout_0"),
            entries: &[directional_light.0, objects.0],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_maps_bind_group_0"),
            layout: &layout,
            entries: &[directional_light.1, objects.1],
        });

        (layout, bind_group)
    }
}
