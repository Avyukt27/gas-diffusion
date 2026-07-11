use std::sync::Arc;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton},
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::Window,
};

use crate::{cpu_grid::CpuGrid, grid::Grid, renderer::Renderer};

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;
pub const CELL_SIZE: usize = 10;
const DIFFUSION: f64 = 2.0;

pub struct State {
    pub window: Arc<Window>,
    pub delta: f64,
    pub renderer: Renderer,
    pub grid: CpuGrid,

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
            grid: CpuGrid::new(WIDTH, HEIGHT, CELL_SIZE),

            mouse_down: false,
            prev_mouse_position: PhysicalPosition::new(0.0, 0.0),
            mouse_position: PhysicalPosition::new(0.0, 0.0),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.grid.update(DIFFUSION, self.delta);
        self.renderer.upload_texture(
            self.grid.concentrations(),
            self.grid.walls(),
            self.grid.width() as u32,
            self.grid.height() as u32,
        );
        self.renderer.render()
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Space, true) => self.grid.set_draw_mode(),
            (KeyCode::ArrowUp, true) => self.grid.set_draw_intensity(0.25),
            (KeyCode::ArrowDown, true) => self.grid.set_draw_intensity(-0.25),
            (KeyCode::Enter, true) => {
                if self.delta != 0.0 {
                    self.delta = 0.0;
                } else {
                    self.delta = 1.0;
                }
            }
            (KeyCode::KeyC, true) => self.grid.clear(),
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn handle_mouse_click(&mut self, state: ElementState, button: MouseButton) {
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                self.mouse_down = state.is_pressed();
                let cell_x = self.mouse_position.x as usize / self.grid.cell_size();
                let cell_y = self.mouse_position.y as usize / self.grid.cell_size();

                if cell_x < self.grid.width() && cell_y < self.grid.height() {
                    self.grid.inject(cell_x, cell_y, cell_x, cell_y, self.delta);
                }
            }
            (MouseButton::Left, ElementState::Released) => self.mouse_down = state.is_pressed(),
            _ => {}
        }
    }

    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position = position;
        if self.mouse_down {
            let cell_x = self.mouse_position.x as usize / self.grid.cell_size();
            let cell_y = self.mouse_position.y as usize / self.grid.cell_size();

            let prev_cell_x = self.prev_mouse_position.x as usize / self.grid.cell_size();
            let prev_cell_y = self.prev_mouse_position.y as usize / self.grid.cell_size();

            if cell_x < self.grid.width()
                && cell_y < self.grid.height()
                && prev_cell_x < self.grid.width()
                && prev_cell_y < self.grid.height()
            {
                self.grid
                    .inject(cell_x, cell_y, prev_cell_x, prev_cell_y, self.delta);
            }
        }
        self.prev_mouse_position = position;
    }
}
