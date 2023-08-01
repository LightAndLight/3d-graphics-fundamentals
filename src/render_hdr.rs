use crate::{
    camera::CameraUniform,
    gpu_buffer::GpuBuffer,
    light::{DirectionalLightGpu, PointLightGpu},
    material::Materials,
    objects::Objects,
    shadow_maps,
    vertex::Vertex,
    vertex_buffer::VertexBuffer,
};

pub struct RenderHdr {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl RenderHdr {
    pub fn new(
        device: &wgpu::Device,
        render_target_format: wgpu::TextureFormat,
        depth_texture_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_hdr_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("render_hdr.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_hdr_pipeline"),
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
                    // HDR render target format (Rgba32Float) doesn't support blending.
                    blend: None,
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
                // If this is disabled then depth testing won't happen.
                depth_write_enabled: true,
                /*
                WebGPU doesn't specify a Z direction for NDC:
                <https://www.reddit.com/r/wgpu/comments/tilvas/is_your_wgpu_world_left_or_right_handed/iykwrp0/>

                The Z direction is implied by the projection matrix, and the depth test needs to bet
                configured to match. If the projection matrix makes objects with high Z smaller (left-handed coordinates / "+Z in"),
                then the closest fragment is the one with the smallest Z, which means we need to
                clear to 1.0 (max Z / far plane) and use the `Less` comparison.

                Conversely, if the projection matrix made objects with low Z smaller (right-handed / "+Z out"),
                then we'd need to clear to 0.0 (min Z / far plane) and use the `Greater` comparison.
                */
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                // What's depth bias?
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
        hdr_render_target_view: &wgpu::TextureView,
        depth_texture_view: &wgpu::TextureView,
        vertex_buffer: &VertexBuffer,
    ) {
        /* What is an "attachment"?

        My current understanding is that a (render pass) attachment is a description of a memory region
        used in the render pass (either as input or output).

        So the `RenderPassColorAttachment`s describe color output locations for the render pass. I suppose
        the "depth" component of the `RenderPassDepthStencilAttachment` is for the [Z-buffer](https://en.wikipedia.org/wiki/Z-buffering).
        What's this "stencil" thing? I'm guessing it's for a [Stencil Buffer](https://en.wikipedia.org/wiki/Stencil_buffer). Don't know
        what that's for (yet!).

        Hypothesis: the `RenderPassColorAttachment`s set up outputs for the render pass' fragment shader.
        Answer: yes, that is the case. If I wanted to emit more information from the fragment shader, then
        I could add another color attachment and write to it at `@location(1)` in the fragment shader. I think
        I'd also need to add another entry to `FragmentState.targets` in the corresponding `RenderPipeline`.

        See also: <https://stackoverflow.com/questions/46384007/what-is-the-meaning-of-attachment-when-speaking-about-the-vulkan-api>
        */
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_hdr_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr_render_target_view,
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
                    /*
                    What effect does this have? Is it overwritten by `depth_write_enabled`?

                    Answer: this controls whether the values written to the depth texture are
                    visible to subsequent passes that use this depth texture.
                    */
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
    pub camera: &'a GpuBuffer<CameraUniform>,
    pub objects: &'a Objects,
    pub display_normals: &'a wgpu::Buffer,
    pub point_lights: &'a GpuBuffer<PointLightGpu>,
    pub directional_lights: &'a GpuBuffer<DirectionalLightGpu>,
    pub materials: &'a Materials,
    pub shadow_map_atlas: &'a wgpu::TextureView,
    pub shadow_map_atlas_sampler: &'a wgpu::Sampler,
    pub shadow_map_lights: &'a GpuBuffer<shadow_maps::Light>,
    pub sky_texture: &'a wgpu::TextureView,
    pub sky_texture_sampler: &'a wgpu::Sampler,
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
                    buffer: self.camera.as_raw_buffer(),
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
        // var<uniform> display_normals: u32; // Apparently booleans aren't host-mappable?
        let display_normals = (
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.display_normals,
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(3)
        // var<storage, read> point_lights: array<PointLight>;
        let point_lights = (
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.point_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(4)
        // var<storage, read> directional_lights: array<DirectionalLight>;
        let directional_lights = (
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.directional_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(5)
        // var<storage, read> materials: array<Material>;
        let materials = (
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::VERTEX,
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
                    buffer: self.materials.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(6)
        // var shadow_map_atlas: texture_depth_2d<f32>;
        let shadow_map_atlas = (
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(self.shadow_map_atlas),
            },
        );

        // @group(0) @binding(7)
        // var shadow_map_atlas_sampler: sampler_comparison;
        let shadow_map_atlas_sampler = (
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(self.shadow_map_atlas_sampler),
            },
        );

        // @group(0) @binding(8)
        // var<storage, read> shadow_map_lights: array<ShadowMapLight>;
        let shadow_map_lights = (
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.shadow_map_lights.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(9)
        // var sky_texture: texture_2d<f32>;
        let sky_texture = (
            wgpu::BindGroupLayoutEntry {
                binding: 9,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: wgpu::BindingResource::TextureView(self.sky_texture),
            },
        );

        // @group(0) @binding(10)
        // var sky_texture_sampler: sampler;
        let sky_texture_sampler = (
            wgpu::BindGroupLayoutEntry {
                binding: 10,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: wgpu::BindingResource::Sampler(self.sky_texture_sampler),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render_hdr_bind_group_layout_0"),
            entries: &[
                camera.0,
                objects.0,
                display_normals.0,
                point_lights.0,
                directional_lights.0,
                materials.0,
                shadow_map_atlas.0,
                shadow_map_atlas_sampler.0,
                shadow_map_lights.0,
                sky_texture.0,
                sky_texture_sampler.0,
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_hdr_bind_group_0"),
            layout: &layout,
            entries: &[
                camera.1,
                objects.1,
                display_normals.1,
                point_lights.1,
                directional_lights.1,
                materials.1,
                shadow_map_atlas.1,
                shadow_map_atlas_sampler.1,
                shadow_map_lights.1,
                sky_texture.1,
                sky_texture_sampler.1,
            ],
        });

        (layout, bind_group)
    }
}
