use winit::window::Window;

pub struct Hud {
    context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl Hud {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let context = egui::Context::default();
        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            texture_format,
            egui_wgpu::RendererOptions {
                dithering: false,
                ..Default::default()
            },
        );

        Self {
            context,
            state,
            renderer,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn render(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: 1.0,
        };

        let input = self.state.take_egui_input(window);
        let full_output = self.context.run(input, |ctx| {
            egui::Window::new("Controls")
                .collapsible(false)
                .resizable(false)
                .fixed_pos(egui::pos2(10.0, 10.0))
                .show(ctx, |ui| {
                    ui.label("test");
                });
        });
        self.state
            .handle_platform_output(window, full_output.platform_output);

        let primitives = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        self.renderer
            .update_buffers(device, queue, encoder, &primitives, &screen_descriptor);

        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();
        self.renderer
            .render(&mut pass, &primitives, &screen_descriptor);
    }
}
