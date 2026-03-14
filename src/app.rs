use std::sync::Arc;

use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use pixels::{Pixels, SurfaceTexture};
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
    pixels: Option<Pixels<'static>>,
    delta: f64,
    grid: Grid,

    ui_context: Context,
    ui_state: Option<State>,
    ui_renderer: Option<Renderer>,

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
            pixels: None,
            delta: 1.0,
            grid: Grid::new(WIDTH, HEIGHT, 10),

            ui_context: Context::default(),
            ui_state: None,
            ui_renderer: None,

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

        let window = Arc::new(window);

        let surface = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, window.clone());
        let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface).unwrap();

        let state = State::new(
            self.ui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
        );
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.ui_state = Some(state);
        self.ui_renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    pixels.resize_surface(size.width, size.height).unwrap();
                    pixels.resize_buffer(size.width, size.height).unwrap();
                }
            }
            WindowEvent::RedrawRequested => {
                let bg_colour: Colour = Colour::new(0, 0, 0, 255);

                self.grid.update(DIFFUSION, self.delta);

                let window = self.window.as_ref().unwrap();
                window.scale_factor();
                let raw_input = self.ui_state.as_mut().unwrap().take_egui_input(window);

                if let Some(pixels) = &mut self.pixels {
                    let frame = pixels.frame_mut();
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel[0] = bg_colour.red;
                        pixel[1] = bg_colour.green;
                        pixel[2] = bg_colour.blue;
                        pixel[3] = bg_colour.alpha;
                    }
                    self.grid.draw(frame);
                    pixels.render().unwrap();
                }

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
