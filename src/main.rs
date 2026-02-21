use minifb::{Key, Window, WindowOptions};

use crate::colour::Colour;

mod colour;
mod grid;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const DIFFUSION: f64 = 2e-15;

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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = bg_colour.to_u32();
        }

        grid.draw(&mut buffer);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
