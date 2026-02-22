use crate::{colour::Colour, source::Source};

pub struct Grid {
    pub screen_width: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: usize,
    pub concentrations: Vec<f64>,
    pub sources: Vec<Source>,
}

impl Grid {
    pub fn new(width: usize, height: usize, cell_size: usize) -> Self {
        let grid_width = width / cell_size;
        let grid_height = height / cell_size;

        Self {
            screen_width: width,
            grid_width,
            grid_height,
            cell_size,
            concentrations: vec![0.0; grid_width * grid_height],
            sources: vec![],
        }
    }

    pub fn draw(&self, buffer: &mut [u8]) {
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let colour = self.generate_heatmap(y * self.grid_width + x);

                for dy in 0..self.cell_size {
                    for dx in 0..self.cell_size {
                        let pixel_x = x * self.cell_size + dx;
                        let pixel_y = y * self.cell_size + dy;

                        let idx = (pixel_y * self.screen_width + pixel_x) * 4;

                        buffer[idx] = colour.red;
                        buffer[idx + 1] = colour.green;
                        buffer[idx + 2] = colour.blue;
                        buffer[idx + 3] = colour.alpha;
                    }
                }
            }
        }
    }

    pub fn update(&mut self, diffusion_coefficient: f64, delta: f64) {
        let mut next = self.concentrations.clone();

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;

                let concentration = self.concentrations[idx];
                let neighbor_sum: f64 = self
                    .get_neighbors(idx)
                    .into_iter()
                    .map(|v| v.unwrap_or(0.0))
                    .sum();

                let mut source_contribution = 0.0;

                for source in &self.sources {
                    if source.x == x && source.y == y {
                        source_contribution += source.rate * delta;
                    }
                }

                next[idx] = (concentration
                    + diffusion_coefficient * delta * (neighbor_sum - 4.0 * concentration)
                        / (self.cell_size * self.cell_size) as f64
                    + source_contribution)
                    .max(0.0);
            }
        }

        self.concentrations = next;
    }

    fn get_neighbors(&self, idx: usize) -> [Option<f64>; 4] {
        let x = idx % self.grid_width;
        let y = idx / self.grid_width;

        let left = if x > 0 {
            Some(self.concentrations[y * self.grid_width + (x - 1)])
        } else {
            None
        };
        let right = if x + 1 < self.grid_width {
            Some(self.concentrations[y * self.grid_width + (x + 1)])
        } else {
            None
        };
        let up = if y > 0 {
            Some(self.concentrations[(y - 1) * self.grid_width + x])
        } else {
            None
        };
        let down = if y + 1 < self.grid_height {
            Some(self.concentrations[(y + 1) * self.grid_width + x])
        } else {
            None
        };
        [left, right, up, down]
    }

    fn generate_heatmap(&self, idx: usize) -> Colour {
        let concentration = self.concentrations[idx].clamp(0.0, 1.0);

        let stops = [
            (0.01, Colour::new(0, 0, 75, 255)),
            (0.25, Colour::new(0, 204, 255, 255)),
            (0.5, Colour::new(0, 255, 0, 255)),
            (0.75, Colour::new(255, 255, 0, 255)),
            (1.0, Colour::new(255, 0, 0, 255)),
        ];

        for i in 0..(stops.len() - 1) {
            let (concentration0, colour0) = stops[i];
            let (concentration1, colour1) = stops[i + 1];

            if concentration >= concentration0 && concentration <= concentration1 {
                let a = (concentration - concentration0) / (concentration1 - concentration0);

                let r = self.lerp(colour0.red as f64, colour1.red as f64, a);
                let g = self.lerp(colour0.green as f64, colour1.green as f64, a);
                let b = self.lerp(colour0.blue as f64, colour1.blue as f64, a);

                return Colour::new(r as u8, g as u8, b as u8, 255);
            }
        }

        Colour::new(0, 0, 0, 0)
    }

    fn lerp(&self, a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
}
