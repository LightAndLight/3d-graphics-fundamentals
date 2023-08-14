use wgpu::util::DeviceExt;

use crate::{gpu_buffer::GpuBuffer, gpu_flag::GpuFlag, vector::Vec2};

pub struct ToneMapping {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertices: wgpu::Buffer,
}

impl ToneMapping {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tone_mapping_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("tone_mapping.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tone_mapping_pipeline"),
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
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tone_mapping_vertices"),
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
            vertices,
        }
    }

    pub fn set_bind_group_0(&mut self, device: &wgpu::Device, bind_group_0: BindGroup0) {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);
        self.bind_group_layout_0 = bind_group_layout_0;
        self.bind_group_0 = bind_group_0;
    }

    pub fn record(&self, command_encoder: &mut wgpu::CommandEncoder, surface: &wgpu::TextureView) {
        /* I originally tried to do tone mapping + output to the frame buffer from a compute shader.
        It didn't work because I can't use the surface texture as a writeable storage texture.
        `winit` gives me a surface that supports `Bgra8unorm` and `Bgra8unormSrgb`, which aren't supported
        by `wgpu` as storage texture formats.

        I work around this by drawing a quad that covers the whole screen and using a fragment shader
        to perform tone mapping for each pixel of the frame buffer.

        See:
        * <https://github.com/gfx-rs/wgpu/issues/3359>
        * <https://github.com/gfx-rs/naga/issues/2195>
         */
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("tone_mapping_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group_0, &[]);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

pub struct BindGroup0<'a> {
    pub hdr_render_target: &'a wgpu::TextureView,
    pub hdr_render_target_sampler: &'a wgpu::Sampler,
    pub tone_mapping_enabled: &'a GpuFlag,
    pub saturating_luminance: &'a GpuBuffer<f32>,
}

impl<'a> BindGroup0<'a> {
    pub fn create(&self, device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        // @group(0) @binding(0)
        // var hdr_render_target: texture_2d<f32>;
        let hdr_render_target = (
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(self.hdr_render_target),
            },
        );

        // @group(0) @binding(1)
        // var hdr_render_target_sampler: sampler;
        let hdr_render_target_sampler = (
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(self.hdr_render_target_sampler),
            },
        );

        // @group(0) @binding(2)
        // var<uniform> tone_mapping_enabled: u32;
        let tone_mapping_enabled = (
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
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
                    buffer: self.tone_mapping_enabled.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(3)
        // var<storage, read> saturating_luminance: f32;
        let saturating_luminance = (
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
                    buffer: self.saturating_luminance.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tone_mapping_pipeline_bind_group_layout_0"),
            entries: &[
                hdr_render_target.0,
                hdr_render_target_sampler.0,
                tone_mapping_enabled.0,
                saturating_luminance.0,
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tone_mapping_pipeline_bind_group_0"),
            layout: &layout,
            entries: &[
                hdr_render_target.1,
                hdr_render_target_sampler.1,
                tone_mapping_enabled.1,
                saturating_luminance.1,
            ],
        });

        (layout, bind_group)
    }
}
