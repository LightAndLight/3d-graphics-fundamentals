use std::time::{Duration, Instant};

use it::{
    camera::Camera,
    color::Color,
    light::{DirectionalLight, PointLight},
    load::load_model,
    material::{Material, MaterialId, Materials},
    objects::{ObjectData, ObjectId, Objects},
    point::Point3,
    vector::{Vec2, Vec3},
    vertex::Vertex,
    vertex_buffer::VertexBuffer,
};
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn triangle_camera_space(object_id: ObjectId, material_id: MaterialId) -> Vec<Vertex> {
    vec![
        Vertex {
            position: Point3 {
                x: 0.5,
                y: -0.5,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -0.5,
                y: -0.5,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
    ]
}

fn square_camera_space(object_id: ObjectId, material_id: MaterialId, side: f32) -> Vec<Vertex> {
    let side_over_2 = side / 2.0;
    vec![
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: -side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
        Vertex {
            position: Point3 {
                x: side_over_2,
                y: -side_over_2,
                z: 0.0,
            },
            object_id,
            normal: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            material_id,
        },
    ]
}

struct Fps {
    frame_times: Vec<Duration>,
    next_frame_time_index: usize,
    instant: Instant,
}

impl Fps {
    fn new() -> Self {
        let next_frame_time_index = 0;
        let frame_times: Vec<Duration> = std::iter::repeat(Duration::new(1, 0)).take(10).collect();
        let instant = Instant::now();

        Self {
            frame_times,
            next_frame_time_index,
            instant,
        }
    }

    fn start_frame(&mut self) {
        self.instant = Instant::now();
    }

    fn end_frame(&mut self) {
        self.frame_times[self.next_frame_time_index] = self.instant.elapsed();
        if self.next_frame_time_index + 1 == self.frame_times.len() {
            self.next_frame_time_index = 0;

            let avg_millis_per_frame = self.frame_times.iter().sum::<Duration>().as_millis() as f32
                / self.frame_times.len() as f32;
            log::debug!("fps: {:?}", 1000.0 / avg_millis_per_frame)
        } else {
            self.next_frame_time_index += 1;
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let mut fps = Fps::new();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    // Safety: `window` must outlive `surface`.
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            // features: wgpu::Features::default(),
            features: wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let window_size = window.inner_size();

    let surface_capabilities = surface.get_capabilities(&adapter);

    // This texture formats have automatic conversion sRGB->linear conversion when reading
    // from the texture and linear->sRGB conversion when writing to the texture.
    //
    // See:
    // * <https://docs.rs/wgpu-types/latest/wgpu_types/enum.TextureFormat.html>
    // * <https://gpuweb.github.io/gpuweb/#texture-formats>
    let desired_surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let surface_format = surface_capabilities
        .formats
        .iter()
        .copied()
        .find(|format| *format == desired_surface_format)
        .unwrap_or_else(|| {
            panic!(
                "surface does not support {:?}. available: {:?}",
                desired_surface_format, surface_capabilities.formats
            )
        });
    log::debug!("surface texture format: {:?}", surface_format);

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window_size.width,
        height: window_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    };
    surface.configure(&device, &surface_config);

    let shader_module = device.create_shader_module(wgpu::include_wgsl!("render_hdr.wgsl"));

    let mut objects = Objects::new(&device, 1000);
    let mut materials = Materials::new(&device, 10);

    let matte_gold_material = materials.insert(
        &queue,
        Material {
            // color: Color::RED,
            color: Color {
                r: 1.0,
                g: 0.86,
                b: 0.57,
                a: 1.0,
            },
            roughness: 0.45,
            metallic: 1.0,
            _padding: [0, 0],
        },
    );
    let matte_red_material = materials.insert(
        &queue,
        Material {
            color: Color::RED,
            roughness: 0.5,
            metallic: 0.0,
            _padding: [0, 0],
        },
    );

    let green_material = materials.insert(
        &queue,
        Material {
            color: Color::GREEN,
            roughness: 0.5,
            metallic: 0.5,
            _padding: [0, 0],
        },
    );
    let blue_material = materials.insert(
        &queue,
        Material {
            color: Color::BLUE,
            roughness: 0.5,
            metallic: 0.5,
            _padding: [0, 0],
        },
    );

    let mut vertex_buffer = VertexBuffer::new(&device, 100000);
    {
        let object_id = objects.insert(
            &queue,
            ObjectData {
                transform: cgmath::Matrix4::from_translation(cgmath::Vector3 {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                })
                .into(),
            },
        );
        vertex_buffer.insert_many(&queue, &triangle_camera_space(object_id, blue_material));
    }

    {
        let object_id = objects.insert(
            &queue,
            ObjectData {
                transform: cgmath::Matrix4::from_translation(cgmath::Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                })
                .into(),
            },
        );
        vertex_buffer.insert_many(
            &queue,
            &square_camera_space(object_id, green_material, 0.25),
        );
    }

    load_model(
        &queue,
        &mut objects,
        &mut vertex_buffer,
        "models/teapot.obj",
        cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: -5.0,
            y: 0.0,
            z: -10.0,
        })
        .into(),
        matte_gold_material,
    );

    load_model(
        &queue,
        &mut objects,
        &mut vertex_buffer,
        "models/monkey.obj",
        cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: -10.0,
        })
        .into(),
        matte_red_material,
    );

    device.poll(wgpu::Maintain::WaitForSubmissionIndex(queue.submit([])));

    let hdr_render_target_format = wgpu::TextureFormat::Rgba32Float;
    let hdr_render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hdr_render_target"),
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: hdr_render_target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let hdr_render_target_view =
        hdr_render_target.create_view(&wgpu::TextureViewDescriptor::default());

    let hdr_render_target_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("hdr_render_target_sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    });

    let render_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // camera
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
                // objects
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
                // display_normals
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
                // point_lights
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
                // directional_lights
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
                // materials
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
            ],
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&render_bind_group_layout],
        push_constant_ranges: &[],
    });

    /*
    A floating point depth buffer is pretty important for working with a high `camera.near` to
    `camera.far` ratio.

    This fixed some weird popping in/out I was getting on certain not-close geometry.

    See:
    * <https://www.khronos.org/opengl/wiki/Depth_Buffer_Precision>
    * <http://www.humus.name/index.php?ID=255>
    * <https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html>
    */
    let depth_texture_format = wgpu::TextureFormat::Depth32Float;

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[Vertex::LAYOUT],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: hdr_render_target_format,
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

    let mut camera = Camera {
        eye: cgmath::Point3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        target: cgmath::Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        up: cgmath::Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        aspect: surface_config.width as f32 / surface_config.height as f32,
        fovy: 45.0,
        near: 0.1,
        far: 100.0,
    };
    let camera_move_speed: f32 = 0.05;
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("camera"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[camera.to_uniform()]),
    });
    let mut camera_updated = false;

    let mut display_normals = false;
    let display_normals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("display_normals"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[if display_normals { 1 } else { 0 } as u32]),
    });
    let mut display_normals_updated = false;

    let point_light_id = objects.insert(
        &queue,
        ObjectData {
            transform: cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: 2.0,
                y: -2.0,
                z: -10.0,
            })
            .into(),
        },
    );
    let point_lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("point_lights"),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[PointLight {
            object_id: point_light_id,
            _padding0: [0, 0, 0],
            color: Color {
                r: 0.0,
                g: 0.6,
                b: 1.0,
                a: 1.0,
            },
            luminous_power: 6e5,
            _padding1: [0, 0, 0],
        }]),
    });

    let directional_lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("directional_lights"),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[DirectionalLight {
            color: Color::WHITE,
            direction: Vec3 {
                x: 1.0,
                y: -1.0,
                z: -0.2,
            },
            illuminance: 110_000.0,
        }]),
    });

    let mut w_held = false;
    let mut a_held = false;
    let mut s_held = false;
    let mut d_held = false;

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        // TODO: recreate this texture when window is resized.
        size: wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: depth_texture_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("depth_texture_view"),
        format: None,
        dimension: None,
        aspect: wgpu::TextureAspect::DepthOnly,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    let luminance_pass_bind_group_0_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("luminance_pass_bind_group_0_layout"),
            entries: &[
                // @group(0) @binding(0)
                // var hdr_render_target: texture_2d<f32>;
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
                // @group(0) @binding(1)
                // var hdr_render_target_sampler: sampler;
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // @group(0) @binding(2)
                // var<uniform> total_luminance_pixels_per_thread: u32;
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
                // @group(0) @binding(3)
                // var<storage, read_write> total_luminance_intermediate: array<f32>;
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
                // @group(0) @binding(4)
                // var<storage, read_write> average_luminance: f32;
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
                // @group(0) @binding(5)
                // var<storage, read_write> auto_EV100: f32;
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
                // @group(0) @binding(6)
                // var<storage, read_write> saturating_luminance: f32;
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
            ],
        });

    let num_pixels = surface_config.width * surface_config.height;

    let total_luminance_threads: u32 = 256;
    let total_luminance_pixels_per_thread_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("total_luminance_pixels_per_thread"),
            contents: bytemuck::cast_slice(&[num_pixels / total_luminance_threads
                + if num_pixels % total_luminance_threads == 0 {
                    0
                } else {
                    1
                }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let total_luminance_intermediate_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("total_luminance_intermediate"),
            contents: bytemuck::cast_slice(
                &std::iter::repeat(0.0)
                    .take(total_luminance_threads as usize)
                    .collect::<Vec<f32>>(),
            ),
            usage: wgpu::BufferUsages::STORAGE,
        });

    let average_luminance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("average_luminance"),
        contents: bytemuck::cast_slice(&[0.0]),
        usage: wgpu::BufferUsages::STORAGE,
    });
    #[allow(non_snake_case)]
    let auto_EV100_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("auto_EV100"),
        contents: bytemuck::cast_slice(&[0.0]),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let saturating_luminance_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("saturating_luminance"),
            contents: bytemuck::cast_slice(&[0.0]),
            usage: wgpu::BufferUsages::STORAGE,
        });

    let luminance_pass_bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("luminance_pass_bind_group_0"),
        layout: &luminance_pass_bind_group_0_layout,
        entries: &[
            // @group(0) @binding(0)
            // var<storage, read> hdr_render_target: texture_storage_2d<vec4<f32>, read>;
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&hdr_render_target_view),
            },
            // @group(0) @binding(1)
            // var hdr_render_target_sampler: sampler;
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&hdr_render_target_sampler),
            },
            // @group(0) @binding(2)
            // var<uniform> total_luminance_pixels_per_thread: u32;
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &total_luminance_pixels_per_thread_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // @group(0) @binding(3)
            // var<storage, read_write> total_luminance_intermediate: array<f32>;
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &total_luminance_intermediate_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // @group(0) @binding(4)
            // var<storage, read_write> average_luminance: f32;
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &average_luminance_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // @group(0) @binding(5)
            // var<storage, read_write> auto_EV100: f32;
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &auto_EV100_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // @group(0) @binding(6)
            // var<storage, read_write> saturating_luminance: f32;
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &saturating_luminance_buffer,
                    offset: 0,
                    size: None,
                }),
            },
        ],
    });

    let luminance_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("luminance_pipeline_layout"),
            bind_group_layouts: &[&luminance_pass_bind_group_0_layout],
            push_constant_ranges: &[],
        });

    let luminance_shader_module =
        device.create_shader_module(wgpu::include_wgsl!("luminance.wgsl"));

    let calculate_total_luminance_intermediate_pipeline =
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("calculate_total_luminance_intermediate_pipeline"),
            layout: Some(&luminance_pipeline_layout),
            module: &luminance_shader_module,
            entry_point: "calculate_total_luminance_intermediate",
        });

    let calculate_average_luminance_pipeline =
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("calculate_average_luminance_pipeline"),
            layout: Some(&luminance_pipeline_layout),
            module: &luminance_shader_module,
            entry_point: "calculate_average_luminance",
        });

    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &render_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: objects.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &display_normals_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &point_lights_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &directional_lights_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: materials.as_raw_buffer(),
                    offset: 0,
                    size: None,
                }),
            },
        ],
    });

    let tone_mapping_pipeline_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tone_mapping_pipeline_bind_group_layout"),
            entries: &[
                // @group(0) @binding(0)
                // var hdr_render_target: texture_2d<f32>;
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
                // @group(0) @binding(1)
                // var hdr_render_target_sampler: sampler;
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // @group(0) @binding(2)
                // var<uniform> tone_mapping_enabled: u32;
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
                // @group(0) @binding(3)
                // var<storage, read> saturating_luminance: f32;
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
            ],
        });

    let mut tone_mapping_enabled = true;
    let mut tone_mapping_enabled_updated = false;

    #[allow(clippy::unnecessary_cast)]
    let tone_mapping_enabled_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tone_mapping_enabled"),
            contents: bytemuck::cast_slice(&[if tone_mapping_enabled { 1 } else { 0 } as u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let tone_mapping_pipeline_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("tone_mapping_pipeline_bind_group"),
        layout: &tone_mapping_pipeline_bind_group_layout,
        entries: &[
            // @group(0) @binding(0)
            // var hdr_render_target: texture_2d<f32>;
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&hdr_render_target_view),
            },
            // @group(0) @binding(1)
            // var hdr_render_target_sampler: sampler;
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&hdr_render_target_sampler),
            },
            // @group(0) @binding(2)
            // var<uniform> tone_mapping_enabled: u32;
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &tone_mapping_enabled_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // @group(0) @binding(3)
            // var<storage, read> saturating_luminance: f32;
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &saturating_luminance_buffer,
                    offset: 0,
                    size: None,
                }),
            },
        ],
    });

    let tone_mapping_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tone_mapping_pipeline_layout"),
            bind_group_layouts: &[&tone_mapping_pipeline_bind_group_layout],
            push_constant_ranges: &[],
        });

    let tone_mapping_shader_module =
        device.create_shader_module(wgpu::include_wgsl!("tone_mapping.wgsl"));
    let tone_mapping_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
    let tone_mapping_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("tone_mapping_pipeline"),
        layout: Some(&tone_mapping_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &tone_mapping_shader_module,
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
            module: &tone_mapping_shader_module,
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

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::Destroyed
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(physical_size) => {
                    surface_config.width = physical_size.width;
                    surface_config.height = physical_size.height;
                    surface.configure(&device, &surface_config);

                    camera.aspect = surface_config.width as f32 / surface_config.height as f32;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::W => match input.state {
                                ElementState::Pressed => {
                                    w_held = true;
                                }
                                ElementState::Released => {
                                    w_held = false;
                                }
                            },
                            VirtualKeyCode::A => match input.state {
                                ElementState::Pressed => {
                                    a_held = true;
                                }
                                ElementState::Released => {
                                    a_held = false;
                                }
                            },
                            VirtualKeyCode::S => match input.state {
                                ElementState::Pressed => {
                                    s_held = true;
                                }
                                ElementState::Released => {
                                    s_held = false;
                                }
                            },
                            VirtualKeyCode::D => match input.state {
                                ElementState::Pressed => {
                                    d_held = true;
                                }
                                ElementState::Released => {
                                    d_held = false;
                                }
                            },
                            VirtualKeyCode::N => {
                                if let ElementState::Pressed = input.state {
                                    display_normals = !display_normals;
                                    display_normals_updated = true;

                                    // Disable tone mapping when displaying normals.
                                    tone_mapping_enabled = !display_normals;
                                    tone_mapping_enabled_updated = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                fps.start_frame();
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if w_held {
                    let camera_direction = camera.target - camera.eye;
                    let camera_movement = camera_move_speed * camera_direction;
                    camera.eye += camera_movement;
                    camera.target += camera_movement;
                    camera_updated = true;
                }

                if s_held {
                    let camera_direction = camera.target - camera.eye;
                    let camera_movement = camera_move_speed * camera_direction;
                    camera.eye -= camera_movement;
                    camera.target -= camera_movement;
                    camera_updated = true;
                }

                if a_held {
                    let camera_direction = camera.target - camera.eye;
                    let camera_movement = camera_move_speed * camera.up.cross(camera_direction);
                    camera.eye += camera_movement;
                    camera.target += camera_movement;
                    camera_updated = true;
                }

                if d_held {
                    let camera_direction = camera.target - camera.eye;
                    let camera_movement = camera_move_speed * camera.up.cross(camera_direction);
                    camera.eye -= camera_movement;
                    camera.target -= camera_movement;
                    camera_updated = true;
                }

                if camera_updated {
                    queue.write_buffer(
                        &camera_buffer,
                        0,
                        bytemuck::cast_slice(&[camera.to_uniform()]),
                    );
                    camera_updated = false;
                }

                if display_normals_updated {
                    queue.write_buffer(
                        &display_normals_buffer,
                        0,
                        bytemuck::cast_slice(&[if display_normals { 1 } else { 0 } as u32]),
                    );
                    display_normals_updated = false;
                }

                if tone_mapping_enabled_updated {
                    queue.write_buffer(
                        &tone_mapping_enabled_buffer,
                        0,
                        bytemuck::cast_slice(&[if tone_mapping_enabled { 1 } else { 0 } as u32]),
                    );
                    tone_mapping_enabled_updated = false;
                }

                let surface_texture = surface.get_current_texture().unwrap();
                let surface_texture_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let commands = {
                    let mut command_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                    {
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
                        let mut render_pass =
                            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("render_pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &hdr_render_target_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 10_000.0 * (97.0_f64 / 255.0).powf(2.2),
                                            g: 10_000.0 * (180.0_f64 / 255.0).powf(2.2),
                                            b: 10_000.0 * (237.0_f64 / 255.0).powf(2.2),
                                            a: 1.0,
                                        }),
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &depth_texture_view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(1.0),
                                            // What effect does this have? Is it overwritten by `depth_write_enabled`?
                                            store: false,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                            });

                        render_pass.set_pipeline(&render_pipeline);
                        render_pass.set_vertex_buffer(0, vertex_buffer.as_raw_slice());
                        render_pass.set_bind_group(0, &render_bind_group, &[]);
                        render_pass.draw(0..vertex_buffer.len() as u32, 0..1);
                    }

                    if tone_mapping_enabled {
                        let mut compute_pass =
                            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                                label: Some("luminance_pass"),
                            });

                        compute_pass.set_bind_group(0, &luminance_pass_bind_group_0, &[]);

                        compute_pass.set_pipeline(&calculate_total_luminance_intermediate_pipeline);
                        // To dispatch a single workgroup, dispatch (1, 1, 1).
                        // If any of the dispatch dimensions are zero then the pipeline won't run.
                        compute_pass.dispatch_workgroups(1, 1, 1);

                        compute_pass.set_pipeline(&calculate_average_luminance_pipeline);
                        compute_pass.dispatch_workgroups(1, 1, 1);
                    }

                    {
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
                        let mut render_pass =
                            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("tone_mapping_pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &surface_texture_view,
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

                        render_pass.set_pipeline(&tone_mapping_pipeline);
                        render_pass.set_bind_group(0, &tone_mapping_pipeline_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, tone_mapping_vertices.slice(..));
                        render_pass.draw(0..6, 0..1);
                    }

                    command_encoder.finish()
                };

                queue.submit(std::iter::once(commands));
                surface_texture.present();

                fps.end_frame();
            }
            _ => {}
        }
    });
}
