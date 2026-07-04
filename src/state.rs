use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

use crate::{grid::Grid, renderer::Renderer};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const CELL_SIZE: usize = 10;
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

    #[cfg(target_arch = "wasm32")]
    pub last_frame_time: f64,
    #[cfg(target_arch = "wasm32")]
    pub frame_count: u32,
    #[cfg(target_arch = "wasm32")]
    pub fps_timer: f64,

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

            #[cfg(target_arch = "wasm32")]
            last_frame_time: 0.0,
            #[cfg(target_arch = "wasm32")]
            frame_count: 0,
            #[cfg(target_arch = "wasm32")]
            fps_timer: 0.0,

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

    // pub fn create_canvas(&mut self) {
    //     let renderer_storage = self.renderer.clone();
    //
    //     #[cfg(not(target_arch = "wasm32"))]
    //     {
    //         let renderer = pollster::block_on(Renderer::new(&self.window, CELL_SIZE as u32));
    //         *renderer_storage.lock().unwrap() = Some(renderer);
    //     }
    //
    //     #[cfg(target_arch = "wasm32")]
    //     {
    //         let window_clone = self.window.clone();
    //         wasm_bindgen_futures::spawn_local(async move {
    //             let renderer = Renderer::new(&window_clone, CELL_SIZE as u32).await;
    //             let scale_factor = window_clone.scale_factor();
    //             let physical_size = winit::dpi::LogicalSize::new(WIDTH as f64, HEIGHT as f64)
    //                 .to_physical::<u32>(scale_factor);
    //             let mut renderer_lock = renderer_storage.lock().unwrap();
    //             *renderer_lock = Some(renderer);
    //             if let Some(ref mut r) = *renderer_lock {
    //                 r.resize(physical_size.width, physical_size.height);
    //             }
    //
    //             window_clone.request_redraw();
    //         })
    //     }
    // }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn render(&mut self) {
        //             if let Ok(mut guard) = self.renderer.lock() {
        //                 if let Some(ref mut renderer) = *guard {
        //                     self.grid.update(DIFFUSION, self.delta);
        //                     renderer.upload_texture(
        //                         &self.grid.concentrations,
        //                         &self.grid.walls,
        //                         self.grid.width as u32,
        //                         self.grid.height as u32,
        //                     );
        //                     renderer.render();
        //                     #[cfg(target_arch = "wasm32")]
        //                     self.window.request_redraw();
        //                 }
        //             }
        //             #[cfg(not(target_arch = "wasm32"))]
        self.window.request_redraw();
    }
}
