use winit::window::Window;

use crate::camera;

pub struct RenderEgui {
    egui_wgpu_renderer: egui_wgpu::Renderer,
}

impl RenderEgui {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let egui_wgpu_renderer =
            egui_wgpu::renderer::Renderer::new(device, surface_format, None, 1);

        Self { egui_wgpu_renderer }
    }

    pub fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &Window,
        screen_descriptor: &egui_wgpu::renderer::ScreenDescriptor,
        egui_winit_state: &mut egui_winit::State,
        egui_context: &egui::Context,
        mouse_look: &camera::MouseLook,
        command_encoder: &mut wgpu::CommandEncoder,
        surface_texture_view: &wgpu::TextureView,
        ui: &mut dyn FnMut(&mut egui::Ui),
    ) {
        egui_context.set_cursor_icon(if mouse_look.enabled() {
            egui::CursorIcon::None
        } else {
            egui::CursorIcon::default()
        });

        let raw_input = egui_winit_state.take_egui_input(window);
        let full_output = egui_context.run(raw_input, |context| {
            egui::Window::new("Debug").show(context, ui);
        });

        egui_winit_state.handle_platform_output(window, egui_context, full_output.platform_output);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_wgpu_renderer
                .update_texture(device, queue, *id, image_delta);
        }

        let paint_jobs = egui_context.tessellate(full_output.shapes);
        self.egui_wgpu_renderer.update_buffers(
            device,
            queue,
            command_encoder,
            &paint_jobs,
            screen_descriptor,
        );

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.egui_wgpu_renderer
            .render(&mut render_pass, &paint_jobs, screen_descriptor)
    }
}
