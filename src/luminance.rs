use crate::gpu_buffer::GpuBuffer;

pub struct Luminance {
    pub bind_group_layout_0: wgpu::BindGroupLayout,
    pub bind_group_0: wgpu::BindGroup,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader_module: wgpu::ShaderModule,
    pub calculate_total_luminance_intermediate_pipeline: wgpu::ComputePipeline,
    pub calculate_average_luminance_pipeline: wgpu::ComputePipeline,
}

impl Luminance {
    pub fn new(device: &wgpu::Device, bind_group_0: BindGroup0) -> Self {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luminance_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout_0],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("luminance.wgsl"));

        let calculate_total_luminance_intermediate_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("calculate_total_luminance_intermediate_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "calculate_total_luminance_intermediate",
            });

        let calculate_average_luminance_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("calculate_average_luminance_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "calculate_average_luminance",
            });

        Self {
            bind_group_layout_0,
            bind_group_0,
            pipeline_layout,
            shader_module,
            calculate_total_luminance_intermediate_pipeline,
            calculate_average_luminance_pipeline,
        }
    }

    pub fn set_bind_group_0(&mut self, device: &wgpu::Device, bind_group_0: BindGroup0) {
        let (bind_group_layout_0, bind_group_0) = bind_group_0.create(device);
        self.bind_group_layout_0 = bind_group_layout_0;
        self.bind_group_0 = bind_group_0;
    }

    pub fn record(&self, command_encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("luminance_pass"),
        });

        compute_pass.set_bind_group(0, &self.bind_group_0, &[]);

        compute_pass.set_pipeline(&self.calculate_total_luminance_intermediate_pipeline);
        // To dispatch a single workgroup, dispatch (1, 1, 1).
        // If any of the dispatch dimensions are zero then the pipeline won't run.
        compute_pass.dispatch_workgroups(1, 1, 1);

        compute_pass.set_pipeline(&self.calculate_average_luminance_pipeline);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
}

#[allow(non_snake_case)] // for `auto_EV100`
pub struct BindGroup0<'a> {
    pub hdr_render_target: &'a wgpu::TextureView,
    pub hdr_render_target_sampler: &'a wgpu::Sampler,
    pub total_luminance_pixels_per_thread: &'a wgpu::Buffer,
    pub total_luminance_intermediate: &'a GpuBuffer<f32>,
    pub average_luminance: &'a GpuBuffer<f32>,
    pub auto_EV100: &'a GpuBuffer<f32>,
    pub saturating_luminance: &'a GpuBuffer<f32>,
}

impl<'a> BindGroup0<'a> {
    pub fn create(&self, device: &wgpu::Device) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        // @group(0) @binding(0)
        // var hdr_render_target: texture_2d<f32>;
        let hdr_render_target = (
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
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
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(self.hdr_render_target_sampler),
            },
        );

        // @group(0) @binding(2)
        // var<uniform> total_luminance_pixels_per_thread: u32;
        let total_luminance_pixels_per_thread = (
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
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
                    buffer: self.total_luminance_pixels_per_thread,
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(3)
        // var<storage, read_write> total_luminance_intermediate: array<f32>;
        let total_luminance_intermediate = (
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.total_luminance_intermediate.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(4)
        // var<storage, read_write> average_luminance: f32;
        let average_luminance = (
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.average_luminance.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(5)
        // var<storage, read_write> auto_EV100: f32;
        #[allow(non_snake_case)]
        let auto_EV100 = (
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.auto_EV100.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        // @group(0) @binding(6)
        // var<storage, read_write> saturating_luminance: f32;
        let saturating_luminance = (
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: self.saturating_luminance.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luminance_pass_bind_group_layout_0"),
            entries: &[
                hdr_render_target.0,
                hdr_render_target_sampler.0,
                total_luminance_pixels_per_thread.0,
                total_luminance_intermediate.0,
                average_luminance.0,
                auto_EV100.0,
                saturating_luminance.0,
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("luminance_pass_bind_group_0"),
            layout: &layout,
            entries: &[
                hdr_render_target.1,
                hdr_render_target_sampler.1,
                total_luminance_pixels_per_thread.1,
                total_luminance_intermediate.1,
                average_luminance.1,
                auto_EV100.1,
                saturating_luminance.1,
            ],
        });

        (layout, bind_group)
    }
}
