use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

use crate::{colour::Colour, grid::Grid, source::Source};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const DIFFUSION: f64 = 2.0;
const DELTA: f64 = 1.0;

#[derive(PartialEq, Eq, Debug)]
enum DrawMode {
    GAS,
    SOURCE,
    SINK,
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    grid: Grid,
    draw_mode: DrawMode,
    draw_size: usize,
    draw_intensity: f64,
    mouse_position: PhysicalPosition<f64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            grid: Grid::new(WIDTH, HEIGHT, 10),
            draw_mode: DrawMode::GAS,
            draw_size: 1,
            draw_intensity: 1.0,
            mouse_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    fn apply_brush(&mut self, start_x: usize, start_y: usize, width: usize, height: usize) {
        for y in start_y..(start_y + height).min(self.grid.grid_height) {
            for x in start_x..(start_x + width).min(self.grid.grid_width) {
                let idx = y * self.grid.grid_width + x;
                match self.draw_mode {
                    DrawMode::GAS => {
                        self.grid.concentrations[idx] = self.draw_intensity.clamp(0.0, 1.0);
                    }
                    DrawMode::SOURCE => {
                        self.grid.sources.push(Source {
                            x,
                            y,
                            rate: self.draw_intensity.abs() / 100.0,
                        });
                    }
                    DrawMode::SINK => {
                        self.grid.sources.push(Source {
                            x,
                            y,
                            rate: -self.draw_intensity.abs() / 100.0,
                        });
                    }
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

        self.window = Some(window.clone());
        self.pixels = Some(pixels);

        self.apply_brush(0, 0, 40, 30);
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

                self.grid.update(DIFFUSION, DELTA);

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
                        Key::Named(NamedKey::Space) => {
                            if self.draw_mode == DrawMode::GAS {
                                self.draw_mode = DrawMode::SOURCE;
                            } else if self.draw_mode == DrawMode::SOURCE {
                                self.draw_mode = DrawMode::SINK;
                            } else if self.draw_mode == DrawMode::SINK {
                                self.draw_mode = DrawMode::SINK;
                            }
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            self.draw_intensity = (self.draw_intensity + 0.25).clamp(0.0, 1.0)
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            self.draw_intensity = (self.draw_intensity - 0.25).clamp(0.0, 1.0)
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => self.mouse_position = position,
            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() {
                    println!(
                        "position: {:?}\nbutton: {:?}\nsize: {:?}\nmode: {:?}",
                        self.mouse_position, button, self.draw_size, self.draw_mode
                    );
                    match button {
                        MouseButton::Left => self.apply_brush(
                            self.mouse_position.x as usize,
                            self.mouse_position.y as usize,
                            self.draw_size,
                            self.draw_size,
                        ),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
