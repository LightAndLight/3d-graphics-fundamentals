use it::{color::Color, point::Point3, vertex::Vertex, vertex_buffer::VertexBuffer};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const TRIANGLE_CLIP_SPACE: [Vertex; 3] = [
    Vertex {
        position: Point3 {
            x: 0.5,
            y: -0.5,
            z: 0.0,
        },
        color: Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        },
    },
    Vertex {
        position: Point3 {
            x: 0.0,
            y: 0.5,
            z: 0.0,
        },
        color: Color {
            r: 0.3,
            g: 0.4,
            b: 0.5,
            a: 1.0,
        },
    },
    Vertex {
        position: Point3 {
            x: -0.5,
            y: -0.5,
            z: 0.0,
        },
        color: Color {
            r: 0.5,
            g: 0.6,
            b: 0.7,
            a: 1.0,
        },
    },
];

fn square_clip_space(origin: Point3, side: f32) -> Vec<Vertex> {
    let side_over_2 = side / 2.0;
    vec![
        Vertex {
            position: origin
                + Point3 {
                    x: side_over_2,
                    y: side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
        Vertex {
            position: origin
                + Point3 {
                    x: -side_over_2,
                    y: side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
        Vertex {
            position: origin
                + Point3 {
                    x: -side_over_2,
                    y: -side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
        Vertex {
            position: origin
                + Point3 {
                    x: side_over_2,
                    y: side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
        Vertex {
            position: origin
                + Point3 {
                    x: -side_over_2,
                    y: -side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
        Vertex {
            position: origin
                + Point3 {
                    x: side_over_2,
                    y: -side_over_2,
                    z: 0.0,
                },
            color: Color::RED,
        },
    ]
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

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
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let window_size = window.inner_size();

    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities
        .formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .expect("surface does not support sRGB");

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

    let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vertex_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: std::mem::size_of::<Point3>() as u64,
                        shader_location: 1,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fragment_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
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
        // What's a multiview render pass?
        multiview: None,
    });

    let mut vertex_buffer = VertexBuffer::new(&device, 1000);
    vertex_buffer.insert_many(&queue, &TRIANGLE_CLIP_SPACE);
    vertex_buffer.insert_many(
        &queue,
        &square_clip_space(
            Point3 {
                x: 0.5,
                y: 0.5,
                z: 0.0,
            },
            0.25,
        ),
    );
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(queue.submit([])));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

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
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
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

                        See also: <https://stackoverflow.com/questions/46384007/what-is-the-meaning-of-attachment-when-speaking-about-the-vulkan-api>
                        */
                        let mut render_pass =
                            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &surface_texture_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.01,
                                            g: 0.01,
                                            b: 0.01,
                                            a: 1.0,
                                        }),
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: None,
                            });

                        render_pass.set_pipeline(&render_pipeline);
                        render_pass.set_vertex_buffer(0, vertex_buffer.as_raw_slice());
                        render_pass.draw(0..vertex_buffer.len() as u32, 0..1);
                    }

                    command_encoder.finish()
                };

                queue.submit(std::iter::once(commands));
                surface_texture.present();
            }
            _ => {}
        }
    });
}
