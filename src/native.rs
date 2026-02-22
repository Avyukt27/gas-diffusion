use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::{Window, WindowAttributes},
};

use crate::{colour::Colour, grid::Grid, source::Source};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const DIFFUSION: f64 = 2.0;

enum DrawMode {
    GAS,
    SOURCE,
    SINK,
}

fn apply_brush(
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    intensity: f64,
    grid: &mut Grid,
    mode: &DrawMode,
) {
    for y in start_y..(start_y + height).min(grid.grid_height) {
        for x in start_x..(start_x + width).min(grid.grid_width) {
            let idx = y * grid.grid_width + x;
            match mode {
                DrawMode::GAS => {
                    grid.concentrations[idx] = intensity.clamp(0.0, 1.0);
                }

                DrawMode::SOURCE => {
                    grid.sources.push(Source {
                        x,
                        y,
                        rate: intensity.abs() / 100.0,
                    });
                }

                DrawMode::SINK => {
                    grid.sources.push(Source {
                        x,
                        y,
                        rate: -intensity.abs() / 100.0,
                    });
                }
            }
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            pixels: None,
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

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

pub fn run() {
    let bg_colour: Colour = Colour::new(0, 0, 0);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Diffusion Simulation Window",
        WIDTH,
        HEIGHT,
        WindowOptions {
            borderless: true,
            topmost: true,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| panic!("{}", e));
    window.set_target_fps(120);

    let mut grid = Grid::new(WIDTH, HEIGHT, 10);

    let mut mouse_intensity = 1.0;
    let mut mouse_size: usize = 1;
    let mut mouse_mode = DrawMode::GAS;

    let delta = 1.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .get_keys_pressed(minifb::KeyRepeat::No)
            .iter()
            .for_each(|key| match key {
                Key::Space => {
                    mouse_mode = match mouse_mode {
                        DrawMode::GAS => DrawMode::SOURCE,
                        DrawMode::SOURCE => DrawMode::SINK,
                        DrawMode::SINK => DrawMode::GAS,
                    }
                }
                Key::Up => {
                    mouse_intensity += 0.25;
                    if mouse_intensity >= 1.0 {
                        mouse_intensity = 1.0;
                    }
                }
                Key::Down => {
                    mouse_intensity -= 0.25;
                    if mouse_intensity <= 0.0 {
                        mouse_intensity = 0.0;
                    }
                }
                Key::C => {
                    grid.sources.clear();
                    grid.concentrations.fill(0.0);
                }
                _ => (),
            });

        if let Some(mouse_scroll) = window.get_scroll_wheel() {
            if mouse_scroll.1 < 0.0 {
                mouse_size += 1
            } else if mouse_scroll.1 > 0.0 && mouse_size > 0 {
                mouse_size -= 1
            }
        }

        if window.get_mouse_down(minifb::MouseButton::Left)
            && let Some(mouse_pos) = window.get_mouse_pos(minifb::MouseMode::Discard)
        {
            apply_brush(
                mouse_pos.0 as usize / grid.cell_size,
                mouse_pos.1 as usize / grid.cell_size,
                mouse_size,
                mouse_size,
                mouse_intensity,
                &mut grid,
                &mouse_mode,
            );
        }

        grid.update(DIFFUSION, delta);

        for i in buffer.iter_mut() {
            *i = bg_colour.to_u32();
        }

        grid.draw(&mut buffer);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
