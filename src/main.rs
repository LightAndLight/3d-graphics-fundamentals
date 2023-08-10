use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use cgmath::Rotation3;
use image::codecs::hdr::HdrDecoder;
use it::{
    aabb::Aabb,
    camera::{self, Camera, CameraUniform},
    clip,
    color::Color,
    gpu_buffer::GpuBuffer,
    light::{
        DirectionalLight, DirectionalLightGpu, PointLight, PointLightGpu, PointLightShadowMapFace,
        ShadowMapLightIds,
    },
    load::load_model,
    luminance::{self, Luminance},
    material::{Material, Materials},
    matrix::Matrix4,
    model_matrices::ModelMatrices,
    point::Point3,
    render_egui::RenderEgui,
    render_hdr::{self, RenderHdr},
    render_sky::{self, RenderSky},
    render_wireframe::{self, RenderWireframe},
    shadow_map_atlas::ShadowMapAtlas,
    shadow_maps::{self, ShadowMaps},
    shape,
    tone_mapping::{self, ToneMapping},
    vector::{Vec2, Vec3},
    vertex_buffer::VertexBuffer,
};
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

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
                | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
                | wgpu::Features::DEPTH_CLIP_CONTROL,
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

    let mut model_matrices = ModelMatrices::new(&device, 1000);
    let mut shadow_caster_scene_bounds: Aabb = Aabb {
        min: Point3::ZERO,
        max: Point3::ZERO,
    };
    let mut materials = Materials::new(&device, 100);

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
    let grey_material = materials.insert(
        &queue,
        Material {
            color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            roughness: 0.8,
            metallic: 0.0,
            _padding: [0, 0],
        },
    );

    let mut vertex_buffer = VertexBuffer::new(&device, 100000);
    {
        let model_matrix_id = model_matrices.insert(
            &queue,
            cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            })
            .into(),
        );
        vertex_buffer.insert_many(&queue, &shape::triangle(model_matrix_id, blue_material));
    }

    {
        let model_matrix_id = model_matrices.insert(
            &queue,
            cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            })
            .into(),
        );
        vertex_buffer.insert_many(
            &queue,
            &shape::square(model_matrix_id, green_material, 0.25),
        );
    }

    {
        let model_matrix_id = model_matrices.insert(
            &queue,
            cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: 0.0,
                y: -2.5,
                z: 0.0,
            })
            .into(),
        );
        vertex_buffer.insert_many(&queue, &shape::floor(model_matrix_id, grey_material, 100.0));
    }

    for i in 0..10 {
        let matte_grey_material = materials.insert(
            &queue,
            Material {
                color: Color {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                    a: 1.0,
                },
                roughness: 0.1 + (i as f32 / 10.0) * 0.7,
                metallic: 1.0,
                _padding: [0, 0],
            },
        );
        let transform: Matrix4 = cgmath::Matrix4::from_translation(cgmath::Vector3 {
            x: 1.0 + i as f32,
            y: -2.0,
            z: -4.0 - i as f32,
        })
        .into();
        let model_matrix_id = model_matrices.insert(&queue, transform);
        let radius = 0.5;
        let vertices = shape::sphere(model_matrix_id, matte_grey_material, radius);
        vertex_buffer.insert_many(&queue, &vertices);
        let model_aabb = Aabb {
            min: Point3 {
                x: -radius,
                y: -radius,
                z: -radius,
            },
            max: Point3 {
                x: radius,
                y: radius,
                z: radius,
            },
        };

        shadow_caster_scene_bounds = shadow_caster_scene_bounds.union(transform * model_aabb);
    }

    let teapot_aabb = load_model(
        &queue,
        &mut model_matrices,
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
    shadow_caster_scene_bounds = shadow_caster_scene_bounds.union(teapot_aabb);

    let monkey_aabb = load_model(
        &queue,
        &mut model_matrices,
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
    shadow_caster_scene_bounds = shadow_caster_scene_bounds.union(monkey_aabb);

    let hdri = HdrDecoder::new(BufReader::new(
        File::open("hdris/rustig_koppie_puresky_4k.hdr").unwrap(),
    ))
    .unwrap();

    let hdri_metadata = hdri.metadata();
    let hdri_data = hdri.read_image_hdr().unwrap();

    log::debug!("hdri exposure: {:?}", hdri_metadata.exposure);

    let hdri_texture_size = wgpu::Extent3d {
        width: hdri_metadata.width,
        height: hdri_metadata.height,
        depth_or_array_layers: 1,
    };
    let sky_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sky_texture"),
        size: hdri_texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let mut sky_intensity_buffer = GpuBuffer::new(
        &device,
        Some("sky_intensity"),
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        1,
    );
    sky_intensity_buffer.insert(&queue, 80_000.0);

    queue.write_texture(
        wgpu::ImageCopyTextureBase {
            texture: &sky_texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(
            &hdri_data
                .into_iter()
                .map(|pixel| [pixel.0[0], pixel.0[1], pixel.0[2], 1.0])
                .collect::<Vec<[f32; 4]>>(),
        ),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * 4 * hdri_texture_size.width),
            rows_per_image: Some(hdri_texture_size.height),
        },
        hdri_texture_size,
    );
    let sky_texture_view = sky_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sky_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("sky_texture_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    });

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

    let mut camera = Camera {
        eye: Point3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        direction: cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
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
    let mut camera_buffer: GpuBuffer<CameraUniform> = GpuBuffer::new(
        &device,
        Some("camera"),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        1,
    );
    camera_buffer.insert(&queue, camera.to_uniform());
    let mut camera_updated = false;

    let mut display_normals = false;
    let display_normals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("display_normals"),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[if display_normals { 1 } else { 0 } as u32]),
    });
    let mut display_normals_updated = false;

    let mut shadow_map_atlas =
        ShadowMapAtlas::new(&device, wgpu::TextureFormat::Depth16Unorm, 4096);

    let mut shadow_map_lights_buffer: GpuBuffer<shadow_maps::Light> = GpuBuffer::new(
        &device,
        Some("shadow_map_lights"),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        10,
    );

    let mut point_lights_buffer: GpuBuffer<PointLightGpu> = GpuBuffer::new(
        &device,
        Some("point_lights"),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        10,
    );
    let mut point_lights: Vec<PointLight> = Vec::new();
    {
        let position = Point3 {
            x: 2.0,
            y: -2.0,
            z: -10.0,
        };
        let point_light_id = model_matrices.insert(
            &queue,
            cgmath::Matrix4::from_translation(cgmath::Vector3 {
                x: position.x,
                y: position.y,
                z: position.z,
            })
            .into(),
        );

        let shadow_projection = Matrix4::perspective(90.0, 1.0, 0.5, 15.0);

        let mut create_shadow_map_face = |up, face_direction| -> PointLightShadowMapFace {
            let shadow_map_atlas_entry = shadow_map_atlas.allocate();
            let shadow_map_light_gpu_id = shadow_map_lights_buffer.insert(
                &queue,
                shadow_maps::Light {
                    shadow_view: Matrix4::look_to(position, face_direction, up),
                    shadow_projection,
                    shadow_map_atlas_position: shadow_map_atlas_entry.position().into(),
                    shadow_map_atlas_size: [
                        shadow_map_atlas_entry.size(),
                        shadow_map_atlas_entry.size(),
                    ],
                    _padding: [0, 0, 0, 0, 0, 0, 0],
                },
            );
            PointLightShadowMapFace {
                shadow_map_light_gpu_id,
                shadow_map_atlas_entry,
            }
        };

        let x = create_shadow_map_face(Vec3::Y, Vec3::X);
        let neg_x = create_shadow_map_face(Vec3::Y, -Vec3::X);
        let y = create_shadow_map_face(Vec3::Z, Vec3::Y);
        let neg_y = create_shadow_map_face(Vec3::Z, -Vec3::Y);
        let z = create_shadow_map_face(Vec3::Y, Vec3::Z);
        let neg_z = create_shadow_map_face(Vec3::Y, -Vec3::Z);
        point_lights.push(PointLight {
            shadow_map_faces: it::light::PointLightShadowMapFaces {
                x,
                neg_x,
                y,
                neg_y,
                z,
                neg_z,
            },
        });

        point_lights_buffer.insert(
            &queue,
            PointLightGpu {
                model_matrix_id: point_light_id,
                _padding0: [0, 0, 0],
                color: Color {
                    r: 0.0,
                    g: 0.6,
                    b: 1.0,
                    a: 1.0,
                },
                luminous_power: 6e5,
                shadow_map_light_ids: ShadowMapLightIds {
                    x: x.shadow_map_light_gpu_id,
                    neg_x: neg_x.shadow_map_light_gpu_id,
                    y: y.shadow_map_light_gpu_id,
                    neg_y: neg_y.shadow_map_light_gpu_id,
                    z: z.shadow_map_light_gpu_id,
                    neg_z: neg_z.shadow_map_light_gpu_id,
                },
            },
        );
    }

    fn fit_orthographic_projection_to_camera(
        scene_bounds: &Aabb,
        camera: &Camera,
        shadow_view: Matrix4,
    ) -> Aabb {
        let camera_frustum_world_space = camera.frustum_world_space();
        let camera_frustum_light_space = shadow_view * camera_frustum_world_space;

        let camera_shadow_points = [
            camera_frustum_light_space.near_top_left,
            camera_frustum_light_space.near_top_right,
            camera_frustum_light_space.near_bottom_left,
            camera_frustum_light_space.near_bottom_right,
            camera_frustum_light_space.far_top_left,
            camera_frustum_light_space.far_top_right,
            camera_frustum_light_space.far_bottom_left,
            camera_frustum_light_space.far_bottom_right,
        ];

        let (left, right, bottom, top) = camera_shadow_points.iter().fold(
            (
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
            ),
            |(left, right, bottom, top), point| {
                (
                    left.min(point.x),
                    right.max(point.x),
                    bottom.min(point.y),
                    top.max(point.y),
                )
            },
        );

        let left_clipping_plane = clip::Plane::new(
            Vec3::X,
            Point3 {
                x: left,
                y: 0.0,
                z: 0.0,
            },
        );
        let right_clipping_plane = clip::Plane::new(
            -Vec3::X,
            Point3 {
                x: right,
                y: 0.0,
                z: 0.0,
            },
        );
        let bottom_clipping_plane = clip::Plane::new(
            Vec3::Y,
            Point3 {
                x: 0.0,
                y: bottom,
                z: 0.0,
            },
        );
        let top_clipping_plane = clip::Plane::new(
            -Vec3::Y,
            Point3 {
                x: 0.0,
                y: top,
                z: 0.0,
            },
        );

        fn clip_triangles_against_plane(
            plane: clip::Plane,
            triangles: Vec<clip::Triangle>,
        ) -> Vec<clip::Triangle> {
            triangles
                .into_iter()
                .map(move |triangle| (clip::clip_triangle(&plane, &triangle), triangle))
                .fold(Vec::new(), |mut triangles, (clip_result, triangle)| {
                    match clip_result {
                        clip::ClippedTriangle::Accept => {
                            triangles.push(triangle);
                        }
                        clip::ClippedTriangle::Reject => {}
                        clip::ClippedTriangle::Split1(triangle) => {
                            triangles.push(triangle);
                        }
                        clip::ClippedTriangle::Split2(triangle1, triangle2) => {
                            triangles.push(triangle1);
                            triangles.push(triangle2);
                        }
                    };
                    triangles
                })
        }

        let scene_bounds_light_space = shadow_view * scene_bounds;

        let triangles: Vec<clip::Triangle> = {
            let near_top_left = Point3 {
                x: scene_bounds_light_space.min.x,
                y: scene_bounds_light_space.max.y,
                z: scene_bounds_light_space.max.z,
            };
            let near_top_right = Point3 {
                x: scene_bounds_light_space.max.x,
                y: scene_bounds_light_space.max.y,
                z: scene_bounds_light_space.max.z,
            };
            let near_bottom_left = Point3 {
                x: scene_bounds_light_space.min.x,
                y: scene_bounds_light_space.min.y,
                z: scene_bounds_light_space.max.z,
            };
            let near_bottom_right = Point3 {
                x: scene_bounds_light_space.max.x,
                y: scene_bounds_light_space.min.y,
                z: scene_bounds_light_space.max.z,
            };

            let far_top_left = Point3 {
                x: scene_bounds_light_space.min.x,
                y: scene_bounds_light_space.max.y,
                z: scene_bounds_light_space.min.z,
            };
            let far_top_right = Point3 {
                x: scene_bounds_light_space.max.x,
                y: scene_bounds_light_space.max.y,
                z: scene_bounds_light_space.min.z,
            };
            let far_bottom_left = Point3 {
                x: scene_bounds_light_space.min.x,
                y: scene_bounds_light_space.min.y,
                z: scene_bounds_light_space.min.z,
            };
            let far_bottom_right = Point3 {
                x: scene_bounds_light_space.max.x,
                y: scene_bounds_light_space.min.y,
                z: scene_bounds_light_space.min.z,
            };

            vec![
                // Near face
                clip::Triangle(near_top_right, near_top_left, near_bottom_left),
                clip::Triangle(near_top_right, near_bottom_left, near_bottom_right),
                // Far face
                clip::Triangle(far_top_right, far_top_left, far_bottom_left),
                clip::Triangle(far_top_right, far_bottom_left, far_bottom_right),
                // Top face
                clip::Triangle(far_top_right, far_top_left, near_top_left),
                clip::Triangle(far_top_right, near_top_left, near_top_right),
                // Bottom face
                clip::Triangle(near_bottom_right, near_bottom_left, far_bottom_left),
                clip::Triangle(near_bottom_right, far_bottom_left, far_bottom_right),
                // Left face
                clip::Triangle(near_top_left, far_top_left, far_bottom_left),
                clip::Triangle(near_top_left, far_bottom_left, near_bottom_left),
                // Right face
                clip::Triangle(far_top_right, near_top_right, near_bottom_right),
                clip::Triangle(far_top_right, near_bottom_right, far_bottom_right),
            ]
        };
        let clipped_triangles: Vec<clip::Triangle> =
            clip_triangles_against_plane(left_clipping_plane, triangles);
        let clipped_triangles: Vec<clip::Triangle> =
            clip_triangles_against_plane(right_clipping_plane, clipped_triangles);
        let clipped_triangles: Vec<clip::Triangle> =
            clip_triangles_against_plane(bottom_clipping_plane, clipped_triangles);
        let clipped_triangles: Vec<clip::Triangle> =
            clip_triangles_against_plane(top_clipping_plane, clipped_triangles);

        let (near, far) = clipped_triangles.into_iter().fold(
            (f32::NEG_INFINITY, f32::INFINITY),
            |acc, triangle| {
                triangle.into_iter().fold(acc, |(near, far), point| {
                    (near.max(point.z), far.min(point.z))
                })
            },
        );

        dbg!(Aabb {
            min: Point3 {
                x: left,
                y: bottom,
                z: far,
            },
            max: Point3 {
                x: right,
                y: top,
                z: near,
            },
        })
    }

    let mut directional_lights_buffer: GpuBuffer<DirectionalLightGpu> = GpuBuffer::new(
        &device,
        Some("directional_lights"),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        10,
    );
    let mut directional_lights = Vec::new();

    struct DirectionalLightInfo {
        shadow_map_id: u32,
        shadow_view: Matrix4,
        shadow_view_inverse: Matrix4,
        shadow_map_atlas_position: Vec2,
        shadow_map_atlas_size: f32,
        aabb: Aabb,
    }
    let directional_light_info = {
        let direction = Vec3 {
            x: 1.0,
            y: -1.0,
            z: -0.2,
        };
        let shadow_map_atlas_entry = shadow_map_atlas.allocate();
        let position = shadow_map_atlas_entry.position();
        let size = shadow_map_atlas_entry.size();
        let shadow_view = Matrix4::look_to(Point3::ZERO, direction, Vec3::Y);

        let aabb = fit_orthographic_projection_to_camera(
            &shadow_caster_scene_bounds,
            &camera,
            shadow_view,
        );
        debug_assert!(aabb.valid(), "invalid aabb: {:?}", aabb);

        let id = shadow_map_lights_buffer.insert(
            &queue,
            shadow_maps::Light {
                shadow_view,
                shadow_projection: Matrix4::ortho(
                    aabb.min.x,
                    aabb.max.x,
                    aabb.min.y,
                    aabb.max.y,
                    // `ortho` takes positive near/far arguments but still assumes that far is
                    // towards -Z.
                    -aabb.max.z,
                    -aabb.min.z,
                ),
                shadow_map_atlas_position: position.into(),
                shadow_map_atlas_size: [size, size],
                _padding: [0, 0, 0, 0, 0, 0, 0],
            },
        );
        directional_lights_buffer.insert(
            &queue,
            DirectionalLightGpu {
                color: Color::WHITE,
                direction,
                illuminance: 110_000.0,
                shadow_map_light_id: id,
            },
        );
        directional_lights.push(DirectionalLight {
            shadow_map_light_gpu_id: id,
            shadow_map_atlas_entry,
        });

        DirectionalLightInfo {
            shadow_map_id: id,
            shadow_view,
            shadow_view_inverse: shadow_view.inverse(),
            shadow_map_atlas_position: position,
            shadow_map_atlas_size: size,
            aabb,
        }
    };

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

    let shadow_maps = ShadowMaps::new(
        &device,
        shadow_map_atlas.texture_format(),
        shadow_maps::BindGroup0 {
            lights: &shadow_map_lights_buffer,
            model_matrices: &model_matrices,
        },
    );

    let render_sky = RenderSky::new(
        &device,
        hdr_render_target_format,
        render_sky::BindGroup0 {
            camera: &camera_buffer,
            sky_texture: &sky_texture_view,
            sky_texture_sampler: &sky_texture_sampler,
            sky_intensity: &sky_intensity_buffer,
        },
    );

    let mut show_directional_shadow_map_coverage_buffer = GpuBuffer::new(
        &device,
        Some("show_directional_shadow_map_coverage_buffer"),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        1,
    );
    let mut show_directional_shadow_map_coverage = false;
    let mut show_directional_shadow_map_coverage_updated = false;
    show_directional_shadow_map_coverage_buffer.insert(
        &queue,
        if show_directional_shadow_map_coverage {
            1
        } else {
            0
        },
    );

    let render_hdr = RenderHdr::new(
        &device,
        hdr_render_target_format,
        depth_texture_format,
        render_hdr::BindGroup0 {
            camera: &camera_buffer,
            model_matrices: &model_matrices,
            display_normals: &display_normals_buffer,
            point_lights: &point_lights_buffer,
            directional_lights: &directional_lights_buffer,
            materials: &materials,
            shadow_map_atlas: shadow_map_atlas.texture_view(),
            shadow_map_atlas_sampler: shadow_map_atlas.sampler(),
            shadow_map_lights: &shadow_map_lights_buffer,
            sky_texture: &sky_texture_view,
            sky_texture_sampler: &sky_texture_sampler,
        },
        render_hdr::BindGroup1 {
            show_directional_shadow_map_coverage: &show_directional_shadow_map_coverage_buffer,
        },
    );

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

    let luminance = Luminance::new(
        &device,
        luminance::BindGroup0 {
            hdr_render_target: &hdr_render_target_view,
            hdr_render_target_sampler: &hdr_render_target_sampler,
            total_luminance_pixels_per_thread: &total_luminance_pixels_per_thread_buffer,
            total_luminance_intermediate: &total_luminance_intermediate_buffer,
            average_luminance: &average_luminance_buffer,
            auto_EV100: &auto_EV100_buffer,
            saturating_luminance: &saturating_luminance_buffer,
        },
    );

    let mut tone_mapping_enabled = true;
    let mut tone_mapping_enabled_updated = false;

    #[allow(clippy::unnecessary_cast)]
    let tone_mapping_enabled_buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tone_mapping_enabled"),
            contents: bytemuck::cast_slice(&[if tone_mapping_enabled { 1 } else { 0 } as u32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let tone_mapping = ToneMapping::new(
        &device,
        surface_format,
        tone_mapping::BindGroup0 {
            hdr_render_target: &hdr_render_target_view,
            hdr_render_target_sampler: &hdr_render_target_sampler,
            tone_mapping_enabled: &tone_mapping_enabled_buffer,
            saturating_luminance: &saturating_luminance_buffer,
        },
    );

    let mut render_wireframe_vertex_buffer: GpuBuffer<render_wireframe::VertexInput> =
        GpuBuffer::new(
            &device,
            Some("render_wireframe_vertex_buffer"),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            100,
        );

    {
        let model_matrix_id = model_matrices.insert(&queue, Matrix4::IDENTITY);
        for (from, to) in shadow_caster_scene_bounds.as_cuboid().wireframe_mesh() {
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: from,
                    model_matrix_id,
                },
            );
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: to,
                    model_matrix_id,
                },
            );
        }
    }

    let (
        directional_light_frustum_model_matrix_id,
        directional_light_frustum_wireframe_vertex_buffer_offset,
    ) = {
        let model_matrix_id =
            model_matrices.insert(&queue, directional_light_info.shadow_view_inverse);

        let offset = render_wireframe_vertex_buffer.len();

        for (from, to) in directional_light_info.aabb.as_cuboid().wireframe_mesh() {
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: from,
                    model_matrix_id,
                },
            );
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: to,
                    model_matrix_id,
                },
            );
        }
        (model_matrix_id, offset)
    };

    let mut render_camera_frustum_wireframe = true;
    let mut render_camera_frustum_wireframe_updated = false;
    let camera_frustum_model_matrix_id = {
        let camera_frustum = camera.frustum_camera_space();

        /* So the camera's "model" matrix is the inverse of its view matrix (<https://jsantell.com/model-view-projection/>).
        That's cool!

        A "model" matrix takes model-space coordinates to world-space coordinates. "Camera-space" is the camera's model-space,
        and the view matrix takes world-space coordinates to camera-space coordinates, which is the inverse of the "model" transformation.
        */
        let model_matrix_id = model_matrices.insert(&queue, camera.view_matrix().inverse());

        for (from, to) in camera_frustum.wireframe_mesh() {
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: from,
                    model_matrix_id,
                },
            );
            render_wireframe_vertex_buffer.insert(
                &queue,
                render_wireframe::VertexInput {
                    position: to,
                    model_matrix_id,
                },
            );
        }

        model_matrix_id
    };

    let render_wireframe = RenderWireframe::new(
        &device,
        surface_format,
        depth_texture_format,
        render_wireframe::BindGroup0 {
            camera: &camera_buffer,
            model_matrices: &model_matrices,
        },
    );

    let pixels_per_point = 2.0;
    let mut render_egui = RenderEgui::new(
        &device,
        surface_format,
        pixels_per_point,
        surface_config.width,
        surface_config.height,
    );
    let mut egui_winit_state = egui_winit::State::new(&window);
    egui_winit_state.set_pixels_per_point(pixels_per_point);
    let egui_context = egui::Context::default();

    let mut mouse_look = camera::MouseLook::new(&window, true);

    let mut w_held = false;
    let mut a_held = false;
    let mut s_held = false;
    let mut d_held = false;

    let mut propagate_camera_updates = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                let response = egui_winit_state.on_event(&egui_context, &event);

                if !response.consumed {
                    match event {
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
                            mouse_look.set(&window, false);
                        }
                        WindowEvent::Resized(physical_size) => {
                            surface_config.width = physical_size.width;
                            surface_config.height = physical_size.height;
                            surface.configure(&device, &surface_config);

                            camera.aspect =
                                surface_config.width as f32 / surface_config.height as f32;
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
                                    _ => {}
                                }
                            }
                        }
                        WindowEvent::MouseInput {
                            state: winit::event::ElementState::Pressed,
                            button: winit::event::MouseButton::Left,
                            ..
                        } if !mouse_look.enabled() => {
                            mouse_look.set(&window, true);
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                fps.start_frame();
                window.request_redraw();
                if mouse_look.enabled() {
                    window
                        .set_cursor_position(winit::dpi::PhysicalPosition {
                            x: surface_config.width as f32 / 2.0,
                            y: surface_config.height as f32 / 2.0,
                        })
                        .unwrap();
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if w_held {
                    let camera_movement = camera_move_speed * camera.direction;
                    camera.eye.x += camera_movement.x;
                    camera.eye.z += camera_movement.z;
                    camera_updated = true;
                }

                if s_held {
                    let camera_movement = camera_move_speed * camera.direction;
                    camera.eye.x -= camera_movement.x;
                    camera.eye.z -= camera_movement.z;
                    camera_updated = true;
                }

                if a_held {
                    let camera_movement = camera_move_speed * camera.up.cross(camera.direction);
                    camera.eye.x += camera_movement.x;
                    camera.eye.z += camera_movement.z;
                    camera_updated = true;
                }

                if d_held {
                    let camera_movement = camera_move_speed * camera.up.cross(camera.direction);
                    camera.eye.x -= camera_movement.x;
                    camera.eye.z -= camera_movement.z;
                    camera_updated = true;
                }

                if camera_updated {
                    camera_buffer.update(&queue, 0, camera.to_uniform());
                    camera_updated = false;

                    if propagate_camera_updates {
                        let aabb = fit_orthographic_projection_to_camera(
                            &shadow_caster_scene_bounds,
                            &camera,
                            directional_light_info.shadow_view,
                        );
                        shadow_map_lights_buffer.update(
                            &queue,
                            directional_light_info.shadow_map_id,
                            shadow_maps::Light {
                                shadow_view: directional_light_info.shadow_view,
                                shadow_projection: Matrix4::ortho(
                                    aabb.min.x,
                                    aabb.max.x,
                                    aabb.min.y,
                                    aabb.max.y,
                                    -aabb.max.z,
                                    -aabb.min.z,
                                ),
                                shadow_map_atlas_position: directional_light_info
                                    .shadow_map_atlas_position
                                    .into(),
                                shadow_map_atlas_size: [
                                    directional_light_info.shadow_map_atlas_size,
                                    directional_light_info.shadow_map_atlas_size,
                                ],
                                _padding: [0, 0, 0, 0, 0, 0, 0],
                            },
                        );

                        for (index, (from, to)) in
                            aabb.as_cuboid().wireframe_mesh().into_iter().enumerate()
                        {
                            render_wireframe_vertex_buffer.update(
                                &queue,
                                directional_light_frustum_wireframe_vertex_buffer_offset
                                    + 2 * index as u32,
                                render_wireframe::VertexInput {
                                    position: from,
                                    model_matrix_id: directional_light_frustum_model_matrix_id,
                                },
                            );
                            render_wireframe_vertex_buffer.update(
                                &queue,
                                directional_light_frustum_wireframe_vertex_buffer_offset
                                    + 2 * index as u32
                                    + 1,
                                render_wireframe::VertexInput {
                                    position: to,
                                    model_matrix_id: directional_light_frustum_model_matrix_id,
                                },
                            );
                        }

                        model_matrices.update(
                            &queue,
                            camera_frustum_model_matrix_id,
                            camera.view_matrix().inverse(),
                        );
                    }
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

                if show_directional_shadow_map_coverage_updated {
                    show_directional_shadow_map_coverage_buffer.update(
                        &queue,
                        0,
                        if show_directional_shadow_map_coverage {
                            1
                        } else {
                            0
                        },
                    );
                    show_directional_shadow_map_coverage_updated = false;
                }

                let surface_texture = surface.get_current_texture().unwrap();
                let surface_texture_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let commands = {
                    let mut command_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                    shadow_maps.record(
                        &mut command_encoder,
                        shadow_map_atlas.texture_view(),
                        &point_lights,
                        &directional_lights,
                        &vertex_buffer,
                    );

                    render_sky.record(&mut command_encoder, &hdr_render_target_view);

                    render_hdr.record(
                        &mut command_encoder,
                        &hdr_render_target_view,
                        &depth_texture_view,
                        &vertex_buffer,
                    );

                    if tone_mapping_enabled {
                        luminance.record(&mut command_encoder);
                    }

                    tone_mapping.record(&mut command_encoder, &surface_texture_view);

                    render_wireframe.record(
                        &mut command_encoder,
                        &surface_texture_view,
                        &depth_texture_view,
                        &render_wireframe_vertex_buffer,
                    );

                    render_egui.record(
                        &device,
                        &queue,
                        &window,
                        &mut egui_winit_state,
                        &egui_context,
                        &mouse_look,
                        &mut command_encoder,
                        &surface_texture_view,
                        &mut |ui| {
                            if ui
                                .checkbox(&mut display_normals, "Display normals")
                                .changed()
                            {
                                display_normals_updated = true;

                                // Disable tone mapping when displaying normals.
                                tone_mapping_enabled = !display_normals;
                                tone_mapping_enabled_updated = true;
                            }
                            ui.checkbox(&mut propagate_camera_updates, "Propagate camera updates");

                            show_directional_shadow_map_coverage_updated = ui
                                .checkbox(
                                    &mut show_directional_shadow_map_coverage,
                                    "Show directional shadow map coverage",
                                )
                                .changed();

                            render_camera_frustum_wireframe_updated = ui
                                .checkbox(
                                    &mut render_camera_frustum_wireframe,
                                    "Render camera frustum wireframe",
                                )
                                .changed();

                            if ui.button("Exit").clicked() {
                                *control_flow = ControlFlow::Exit;
                            }
                        },
                    );

                    command_encoder.finish()
                };

                queue.submit(std::iter::once(commands));
                surface_texture.present();

                fps.end_frame();
            }
            Event::DeviceEvent {
                event:
                    winit::event::DeviceEvent::MouseMotion {
                        delta: (delta_x, delta_y),
                    },
                ..
            } => {
                if mouse_look.enabled() {
                    camera.direction = cgmath::Quaternion::from_axis_angle(
                        camera.up,
                        cgmath::Deg(-delta_x as f32 / 10.0),
                    ) * cgmath::Quaternion::from_axis_angle(
                        camera.up.cross(camera.direction),
                        cgmath::Deg(delta_y as f32 / 10.0),
                    ) * camera.direction;
                    camera_updated = true;
                }
            }
            _ => {}
        }
    });
}
