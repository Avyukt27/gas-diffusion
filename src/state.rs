use std::sync::Arc;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

use crate::{grid::Grid, renderer::Renderer};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;
pub const CELL_SIZE: usize = 10;
const DIFFUSION: f64 = 2.0;

#[derive(PartialEq, Eq, Debug)]
pub enum DrawMode {
    Gas,
    Source,
    Sink,
    Advection,
    Stopper,
}

pub struct State {
    pub window: Arc<Window>,
    pub delta: f64,
    pub renderer: Renderer,
    pub grid: Grid,

    pub draw_mode: DrawMode,
    pub draw_size: usize,
    pub draw_intensity: f64,
    pub mouse_down: bool,
    pub prev_mouse_position: PhysicalPosition<f64>,
    pub mouse_position: PhysicalPosition<f64>,
}

impl State {
    pub async fn new(window: Arc<Window>, renderer: Renderer) -> anyhow::Result<Self> {
        Ok(Self {
            window,
            delta: 1.0,
            renderer,
            grid: Grid::new(WIDTH, HEIGHT, CELL_SIZE),

            draw_mode: DrawMode::Gas,
            draw_size: 1,
            draw_intensity: 1.0,
            mouse_down: false,
            prev_mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
        })
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
        if start_x >= self.grid.width || start_y >= self.grid.height {
            return;
        }

        let max_x = (start_x + width).min(self.grid.width);
        let max_y = (start_y + height).min(self.grid.height);

        for y in start_y..max_y {
            for x in start_x..max_x {
                let idx = y * self.grid.width + x;
                if idx >= self.grid.concentrations.len() {
                    continue;
                }
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.grid.update(DIFFUSION, self.delta);
        self.renderer.upload_texture(
            &self.grid.concentrations,
            &self.grid.walls,
            self.grid.width as u32,
            self.grid.height as u32,
        );
        self.renderer.render()
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Space, true) => match self.draw_mode {
                DrawMode::Gas => self.draw_mode = DrawMode::Source,
                DrawMode::Source => self.draw_mode = DrawMode::Sink,
                DrawMode::Sink => self.draw_mode = DrawMode::Advection,
                DrawMode::Advection => self.draw_mode = DrawMode::Stopper,
                DrawMode::Stopper => self.draw_mode = DrawMode::Gas,
            },
            (KeyCode::ArrowUp, true) => {
                self.draw_intensity = (self.draw_intensity + 0.25).clamp(0.0, 1.0)
            }
            (KeyCode::ArrowDown, true) => {
                self.draw_intensity = (self.draw_intensity - 0.25).clamp(0.0, 1.0)
            }
            (KeyCode::Enter, true) => {
                if self.delta != 0.0 {
                    self.delta = 0.0;
                } else {
                    self.delta = 1.0;
                }
            }
            (KeyCode::KeyC, true) => {
                self.grid.concentrations.fill(0.0);
                self.grid.sources.fill(0.0);
                self.grid.advections.fill((0.0, 0.0));
                self.grid.walls.fill(false);
            }
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn handle_mouse_click(&mut self, state: ElementState, button: MouseButton) {
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                self.mouse_down = state.is_pressed();
                let cell_x = self.mouse_position.x as usize / self.grid.cell_size;
                let cell_y = self.mouse_position.y as usize / self.grid.cell_size;

                if cell_x < self.grid.width && cell_y < self.grid.height {
                    self.apply_brush(
                        cell_x,
                        cell_y,
                        cell_x,
                        cell_y,
                        self.draw_size,
                        self.draw_size,
                    );
                }
            }
            (MouseButton::Left, ElementState::Released) => self.mouse_down = state.is_pressed(),
            _ => {}
        }
    }

    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position = position;
        if self.mouse_down {
            let cell_x = self.mouse_position.x as usize / self.grid.cell_size;
            let cell_y = self.mouse_position.y as usize / self.grid.cell_size;

            let prev_cell_x = self.prev_mouse_position.x as usize / self.grid.cell_size;
            let prev_cell_y = self.prev_mouse_position.y as usize / self.grid.cell_size;

            if cell_x < self.grid.width
                && cell_y < self.grid.height
                && prev_cell_x < self.grid.width
                && prev_cell_y < self.grid.height
            {
                self.apply_brush(
                    cell_x,
                    cell_y,
                    prev_cell_x,
                    prev_cell_y,
                    self.draw_size,
                    self.draw_size,
                );
            }
        }
        self.prev_mouse_position = position;
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
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
}
