use crate::{
    gpu_buffer::GpuBuffer, light::DirectionalLight, objects::Objects, shadow_maps, vertex::Vertex,
    vertex_buffer::VertexBuffer,
};

pub struct ShadowMapOverlay {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl ShadowMapOverlay {
    pub fn new(
        device: &wgpu::Device,
        render_target_format: wgpu::TextureFormat,
        depth_texture_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_map_overlay_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let shader_module =
            device.create_shader_module(wgpu::include_wgsl!("shadow_map_overlay.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_map_overlay_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[Vertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_texture_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            // What's a multiview render pass?
            multiview: None,
        });

        Self {
            bind_group_layout_0,
            bind_group_0,
            pipeline_layout,
            shader_module,
            render_pipeline,
        }
    }

    pub fn record(
        &self,
        command_encoder: &mut wgpu::CommandEncoder,
        render_target_view: &wgpu::TextureView,
        depth_texture_view: &wgpu::TextureView,
        vertex_buffer: &VertexBuffer,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shadow_map_overlay_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group_0, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.as_raw_slice());
        render_pass.draw(0..vertex_buffer.len() as u32, 0..1);
    }
}

pub struct BindGroup0<'a> {
    pub camera: &'a wgpu::Buffer,
    pub objects: &'a Objects,
    pub directional_lights: &'a GpuBuffer<DirectionalLight>,
    pub shadow_map_atlas: &'a wgpu::TextureView,
    pub shadow_map_atlas_sampler: &'a wgpu::Sampler,
    pub shadowing_directional_lights: &'a GpuBuffer<shadow_maps::DirectionalLight>,
}

impl<'a> BindGroup0<'a> {
    pub fn create(&self, device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        // @group(0) @binding(0)
        // var<uniform> camera: Camera;
        let camera = (
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.camera,
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
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        // @group(0) @binding(2)
        // var<storage, read> directional_lights: array<DirectionalLight>;
        let directional_lights = (
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.directional_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(3)
        // var shadow_map_atlas: texture_depth_2d<f32>;
        let shadow_map_atlas = (
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(self.shadow_map_atlas),
            },
        );

        // @group(0) @binding(4)
        // var shadow_map_atlas_sampler: sampler_comparison;
        let shadow_map_atlas_sampler = (
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(self.shadow_map_atlas_sampler),
            },
        );

        // @group(0) @binding(5)
        // var<storage, read> shadowing_directional_lights: array<ShadowingDirectionalLight>;
        let shadowing_directional_lights = (
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.shadowing_directional_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render_hdr_bind_group_layout_0"),
            entries: &[
                camera.0,
                objects.0,
                directional_lights.0,
                shadow_map_atlas.0,
                shadow_map_atlas_sampler.0,
                shadowing_directional_lights.0,
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_hdr_bind_group_0"),
            layout: &layout,
            entries: &[
                camera.1,
                objects.1,
                directional_lights.1,
                shadow_map_atlas.1,
                shadow_map_atlas_sampler.1,
                shadowing_directional_lights.1,
            ],
        });

        (layout, bind_group)
    }
}
