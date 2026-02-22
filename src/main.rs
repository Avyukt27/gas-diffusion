use minifb::{Key, Window, WindowOptions};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::colour::Colour;

mod colour;
mod grid;
mod source;

enum DrawMode {
    GAS,
    SOURCE,
    SINK,
}

fn create_cells(
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    intensity: f64,
    grid: &mut grid::Grid,
) {
    for y in start_y..(start_y + height).min(grid.grid_height) {
        for x in start_x..(start_x + width).min(grid.grid_width) {
            let idx = y * grid.grid_width + x;
            grid.concentrations[idx] = intensity.clamp(0.0, 1.0);
        }
    }
}

fn create_source(x: usize, y: usize, rate: f64, grid: &mut grid::Grid) {
    grid.sources.push(source::Source { x, y, rate });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
}
