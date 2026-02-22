use minifb::{Key, Window, WindowOptions};

use crate::colour::Colour;

mod colour;
mod grid;
mod source;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const DIFFUSION: f64 = 0.5;

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
    window.set_target_fps(120);

    let mut grid = grid::Grid::new(WIDTH, HEIGHT, 10);
    create_cells(0, 0, 40, 20, 0.25, &mut grid);
    create_source(40, 30, 0.001, &mut grid);
    create_source(0, 0, -0.06, &mut grid);

    let mut mouse_intensity = 1.0;
    let mut mouse_size: usize = 1;

    let delta = 1.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.get_mouse_down(minifb::MouseButton::Right) {
            mouse_intensity -= 0.25;
            if mouse_intensity <= 0.0 {
                mouse_intensity = 1.0;
            }
        }

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
            create_cells(
                mouse_pos.0 as usize / grid.cell_size,
                mouse_pos.1 as usize / grid.cell_size,
                mouse_size,
                mouse_size,
                mouse_intensity,
                &mut grid,
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
