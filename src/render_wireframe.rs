use wgpu::include_wgsl;

use crate::{camera::CameraUniform, gpu_buffer::GpuBuffer, point::Point3, vector::Vec3};

pub struct RenderWireframe {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub shader_module: wgpu::ShaderModule,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl RenderWireframe {
    pub fn new(
        device: &wgpu::Device,
        render_target_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        bind_group_0: BindGroup0,
    ) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let shader_module = device.create_shader_module(include_wgsl!("render_wireframe.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_wireframe_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_wireframe_render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec3>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
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
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
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
        render_target_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        vertex_buffer: &GpuBuffer<Point3>,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_wireframe_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.as_raw_buffer().slice(..));
        render_pass.set_bind_group(0, &self.bind_group_0, &[]);
        render_pass.draw(0..vertex_buffer.len(), 0..1);
    }
}

pub struct BindGroup0<'a> {
    pub camera: &'a GpuBuffer<CameraUniform>,
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

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render_wireframe_bind_group_layout_0"),
            entries: &[camera.0],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_wireframe_bind_group_0"),
            layout: &layout,
            entries: &[camera.1],
        });

        (layout, bind_group)
    }
}
