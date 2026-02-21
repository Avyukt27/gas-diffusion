use minifb::{Key, Window, WindowOptions};

use crate::colour::Colour;

mod colour;
mod grid;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const DIFFUSION: f64 = 0.1;

fn create_cell_square(
    start_x: usize,
    start_y: usize,
    square_size: usize,
    intensity: f64,
    grid: &mut grid::Grid,
) {
    for y in start_y..(start_y + square_size) {
        for x in start_x..(start_x + square_size) {
            let idx = y * grid.grid_width + x;
            grid.concentrations[idx] = intensity.clamp(0.0, 1.0);
        }
    }
}

fn main() {
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
    window.set_target_fps(60);

    let mut grid = grid::Grid::new(WIDTH, HEIGHT, 1);
    create_cell_square(200, 300, 10, 0.75, &mut grid);
    create_cell_square(100, 90, 5, 1.0, &mut grid);
    create_cell_square(600, 420, 20, 0.5, &mut grid);

    let delta = 2.5;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        grid.update(DIFFUSION, delta);

        for i in buffer.iter_mut() {
            *i = bg_colour.to_u32();
        }

        grid.draw(&mut buffer);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
