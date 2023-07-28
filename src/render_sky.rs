use wgpu::util::DeviceExt;

use crate::vector::Vec2;

pub struct RenderSky {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

impl RenderSky {
    pub fn new(
        device: &wgpu::Device,
        render_target_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_sky_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("render_sky.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_sky_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_target_format,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("render_sky_vertices"),
            contents: bytemuck::cast_slice(&[
                Vec2 { x: 1.0, y: 1.0 },
                Vec2 { x: -1.0, y: -1.0 },
                Vec2 { x: 1.0, y: -1.0 },
                Vec2 { x: 1.0, y: 1.0 },
                Vec2 { x: -1.0, y: 1.0 },
                Vec2 { x: -1.0, y: -1.0 },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            bind_group_layout_0,
            bind_group_0,
            pipeline_layout,
            shader_module,
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn record(
        &self,
        command_encoder: &mut wgpu::CommandEncoder,
        hdr_render_target_view: &wgpu::TextureView,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_sky_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr_render_target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group_0, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

pub struct BindGroup0<'a> {
    pub camera: &'a wgpu::Buffer,
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
                    buffer: self.camera,
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(1)
        // var sky_texture: texture_2d<f32>;
        let sky_texture = (
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(self.sky_texture),
            },
        );

        // @group(0) @binding(2)
        // var sky_texture_sampler: sampler;
        let sky_texture_sampler = (
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(self.sky_texture_sampler),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render_sky_bind_group_layout_0"),
            entries: &[camera.0, sky_texture.0, sky_texture_sampler.0],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_sky_bind_group_0"),
            layout: &layout,
            entries: &[camera.1, sky_texture.1, sky_texture_sampler.1],
        });

        (layout, bind_group)
    }
}
