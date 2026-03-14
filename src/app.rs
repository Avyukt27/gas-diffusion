use std::sync::Arc;

use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

use crate::{colour::Colour, grid::Grid};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const DIFFUSION: f64 = 2.0;

#[derive(PartialEq, Eq, Debug)]
enum DrawMode {
    Gas,
    Source,
    Sink,
    Advection,
    Stopper,
}

pub struct App {
    window: Option<Arc<Window>>,

    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,

    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    sampler: Option<wgpu::Sampler>,
    render_pipeline: Option<wgpu::RenderPipeline>,

    ui_context: Context,
    ui_state: Option<State>,
    ui_renderer: Option<Renderer>,

    delta: f64,
    grid: Grid,

    draw_mode: DrawMode,
    draw_size: usize,
    draw_intensity: f64,
    mouse_down: bool,
    prev_mouse_position: PhysicalPosition<f64>,
    mouse_position: PhysicalPosition<f64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,

            surface: None,
            device: None,
            queue: None,
            config: None,

            texture: None,
            texture_view: None,
            sampler: None,
            render_pipeline: None,

            ui_context: Context::default(),
            ui_state: None,
            ui_renderer: None,

            delta: 1.0,
            grid: Grid::new(WIDTH, HEIGHT, 10),

            draw_mode: DrawMode::Gas,
            draw_size: 1,
            draw_intensity: 1.0,
            mouse_down: false,
            prev_mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    fn apply_brush(
        &mut self,
        start_x: usize,
        start_y: usize,
        prev_cell_x: usize,
        prev_cell_y: usize,
        width: usize,
        height: usize,
    ) {
        if start_x >= self.grid.grid_width || start_y >= self.grid.grid_height {
            return;
        }

        let max_x = (start_x + width).min(self.grid.grid_width);
        let max_y = (start_y + height).min(self.grid.grid_height);

        for y in start_y..max_y {
            for x in start_x..max_x {
                let idx = y * self.grid.grid_width + x;
                match self.draw_mode {
                    DrawMode::Gas => {
                        self.grid.concentrations[idx] = self.draw_intensity.clamp(0.0, 1.0);
                    }
                    DrawMode::Source | DrawMode::Sink => {
                        let rate = if matches!(self.draw_mode, DrawMode::Source) {
                            self.draw_intensity.abs() / 100.0
                        } else {
                            -self.draw_intensity.abs() / 100.0
                        };
                        self.grid.sources[idx] += rate;
                    }
                    DrawMode::Advection => {
                        let dx = start_x as f64 - prev_cell_x as f64;
                        let dy = start_y as f64 - prev_cell_y as f64;
                        let strength = 5.0;
                        let vel = (dx * strength, dy * strength);
                        let max_vel = self.grid.cell_size as f64 / self.delta * 0.5;

                        self.grid.advections[idx].0 =
                            (self.grid.advections[idx].0 + vel.0).clamp(-max_vel, max_vel);
                        self.grid.advections[idx].1 =
                            (self.grid.advections[idx].1 + vel.1).clamp(-max_vel, max_vel);
                    }
                    DrawMode::Stopper => self.grid.walls[idx] = true,
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("Diffusion Simulation Window")
                    .with_inner_size(winit::dpi::LogicalSize::new(WIDTH as f64, HEIGHT as f64)),
            )
            .unwrap();
        let size = window.inner_size();
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        self.surface = Some(surface);

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(self.surface.as_ref().unwrap()),
            force_fallback_adapter: false,
        }))
        .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        self.device = Some(device.clone());
        self.queue = Some(queue);

        let surface_capabilities = self.surface.as_ref().unwrap().get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface
            .as_ref()
            .unwrap()
            .configure(&device, &surface_config);
        self.config = Some(surface_config);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let bg_colour: Colour = Colour::new(0, 0, 0, 255);

                let window = self.window.as_ref().unwrap();
                window.scale_factor();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key, state, ..
                },
                ..
            } => {
                if state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::Space) => match self.draw_mode {
                            DrawMode::Gas => self.draw_mode = DrawMode::Source,
                            DrawMode::Source => self.draw_mode = DrawMode::Sink,
                            DrawMode::Sink => self.draw_mode = DrawMode::Advection,
                            DrawMode::Advection => self.draw_mode = DrawMode::Stopper,
                            DrawMode::Stopper => self.draw_mode = DrawMode::Gas,
                        },
                        Key::Named(NamedKey::ArrowUp) => {
                            self.draw_intensity = (self.draw_intensity + 0.25).clamp(0.0, 1.0)
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            self.draw_intensity = (self.draw_intensity - 0.25).clamp(0.0, 1.0)
                        }
                        Key::Named(NamedKey::Enter) => {
                            if self.delta != 0.0 {
                                self.delta = 0.0;
                            } else {
                                self.delta = 1.0;
                            }
                        }
                        Key::Character(ref c) if c == "c" => {
                            self.grid.concentrations.fill(0.0);
                            self.grid.sources.fill(0.0);
                            self.grid.advections.fill((0.0, 0.0));
                            self.grid.walls.fill(false);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = position;
                if self.mouse_down {
                    self.apply_brush(
                        self.mouse_position.x as usize / self.grid.cell_size,
                        self.mouse_position.y as usize / self.grid.cell_size,
                        self.prev_mouse_position.x as usize / self.grid.cell_size,
                        self.prev_mouse_position.y as usize / self.grid.cell_size,
                        self.draw_size,
                        self.draw_size,
                    );
                }
                self.prev_mouse_position = position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.mouse_down = state.is_pressed();
                    self.apply_brush(
                        self.mouse_position.x as usize / self.grid.cell_size,
                        self.mouse_position.y as usize / self.grid.cell_size,
                        self.prev_mouse_position.x as usize / self.grid.cell_size,
                        self.prev_mouse_position.y as usize / self.grid.cell_size,
                        self.draw_size,
                        self.draw_size,
                    );
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y as f64,
                    MouseScrollDelta::PixelDelta(pos) => pos.y / 50.0,
                };

                if scroll_y > 0.0 {
                    self.draw_size += 1;
                } else if scroll_y < 0.0 && self.draw_size > 1 {
                    self.draw_size -= 1;
                }
            }
            _ => {}
        }
    }
}
