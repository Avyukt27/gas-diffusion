use crate::colour::Colour;

pub struct Grid {
    screen_width: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: usize,
    pub concentrations: Vec<f64>,
    pub sources: Vec<f64>,
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
            sources: vec![0.0; grid_width * grid_height],
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

    pub fn update(
        &mut self,
        diffusion_coefficient: f64,
        advection_constants: (f64, f64),
        delta: f64,
    ) {
        let mut next = self.concentrations.clone();
        let advections = self.get_advections(advection_constants, delta);

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                let source_rate = self.sources[idx];
                let advection = advections[idx];

                let neighbors = self.get_neighbors(idx, &advections);
                let neighbor_sum: f64 = neighbors.into_iter().map(|v| v.unwrap_or(0.0)).sum();

                next[idx] = (advection
                    + diffusion_coefficient * delta * (neighbor_sum - 4.0 * advection)
                        / (self.cell_size * self.cell_size) as f64
                    + source_rate)
                    .max(0.0);
            }
        }

        self.concentrations = next;
    }

    fn get_value_change(
        &self,
        value: f64,
        neighbors: [Option<f64>; 4],
        advection_constants: (f64, f64),
    ) -> (f64, f64) {
        let concentration_change_x = if advection_constants.0 > 0.0
            && let Some(concentration_left) = neighbors[0]
        {
            (value - concentration_left) / self.cell_size as f64
        } else if advection_constants.0 < 0.0
            && let Some(concentration_right) = neighbors[1]
        {
            (concentration_right - value) / self.cell_size as f64
        } else {
            0.0
        };
        let concentration_change_y = if advection_constants.1 > 0.0
            && let Some(concentration_up) = neighbors[2]
        {
            (value - concentration_up) / self.cell_size as f64
        } else if advection_constants.1 < 0.0
            && let Some(concentration_down) = neighbors[3]
        {
            (concentration_down - value) / self.cell_size as f64
        } else {
            0.0
        };

        (concentration_change_x, concentration_change_y)
    }

    fn get_forward_advections(&self, advection_constants: (f64, f64), delta: f64) -> Vec<f64> {
        let mut forward_advections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;

                let concentration = self.concentrations[idx];
                let neighbors: [Option<f64>; 4] = self.get_neighbors(idx, &self.concentrations);

                let value_change =
                    self.get_value_change(concentration, neighbors, advection_constants);

                let forward_advection = concentration
                    - delta
                        * (advection_constants.0 * value_change.0
                            + advection_constants.1 * value_change.1);

                forward_advections[idx] = forward_advection;
            }
        }

        forward_advections
    }

    fn get_backward_advections(
        &self,
        forward_advections: &Vec<f64>,
        advection_constants: (f64, f64),
        delta: f64,
    ) -> Vec<f64> {
        let mut backward_advections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                let forward_advection = forward_advections[idx];
                let neighbors: [Option<f64>; 4] = self.get_neighbors(idx, &forward_advections);

                let value_change = self.get_value_change(
                    forward_advection,
                    neighbors,
                    (-advection_constants.0, -advection_constants.1),
                );

                let backward_advection = forward_advection
                    + delta
                        * (advection_constants.0 * value_change.0
                            + advection_constants.1 * value_change.1);

                backward_advections[idx] = backward_advection;
            }
        }

        backward_advections
    }

    fn get_corrections(&self, advection_constants: (f64, f64), delta: f64) -> Vec<f64> {
        let forward_advections = self.get_forward_advections(advection_constants, delta);
        let backward_advections =
            self.get_backward_advections(&forward_advections, advection_constants, delta);
        let mut corrections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                let concentration = self.concentrations[idx];
                let forward_advection = forward_advections[idx];
                let backward_advection = backward_advections[idx];
                corrections[idx] = forward_advection + 0.5 * (concentration - backward_advection);
            }
        }

        corrections
    }

    fn get_advections(&self, advection_constants: (f64, f64), delta: f64) -> Vec<f64> {
        let corrections = self.get_corrections(advection_constants, delta);
        let mut advections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                let concentration = self.concentrations[idx];
                let correction = corrections[idx];
                let neighbors = self.get_neighbors(idx, &corrections);

                let mut min_advection = concentration;
                let mut max_advection = concentration;
                for neighbor in neighbors {
                    match neighbor {
                        Some(n) => {
                            min_advection = min_advection.min(n);
                            max_advection = max_advection.max(n);
                        }
                        None => {}
                    }
                }
                advections[idx] = correction.clamp(min_advection, max_advection);
            }
        }

        advections
    }

    fn get_neighbors(&self, idx: usize, values_grid: &Vec<f64>) -> [Option<f64>; 4] {
        let x = idx % self.grid_width;
        let y = idx / self.grid_width;

        let left = if x > 0 {
            Some(values_grid[y * self.grid_width + (x - 1)])
        } else {
            None
        };
        let right = if x + 1 < self.grid_width {
            Some(values_grid[y * self.grid_width + (x + 1)])
        } else {
            None
        };
        let up = if y > 0 {
            Some(values_grid[(y - 1) * self.grid_width + x])
        } else {
            None
        };
        let down = if y + 1 < self.grid_height {
            Some(values_grid[(y + 1) * self.grid_width + x])
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
