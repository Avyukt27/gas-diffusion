#[derive(Copy, Clone)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

pub struct Grid {
    screen_width: usize,
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: usize,
    pub concentrations: Vec<f64>,
    pub sources: Vec<f64>,
    pub advections: Vec<(f64, f64)>,
    pub walls: Vec<bool>,
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
            advections: vec![(0.0, 0.0); grid_width * grid_height],
            walls: vec![false; grid_width * grid_height],
        }
    }

    pub fn draw(&self, buffer: &mut [u8]) {
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;

                if self.walls[idx] {
                    self.draw_cell(x, y, buffer, Colour::new(128, 128, 128, 255));
                    continue;
                }

                let colour = self.generate_heatmap(idx);
                self.draw_cell(x, y, buffer, colour);
            }
        }
    }

    pub fn update(&mut self, diffusion_coefficient: f64, delta: f64) {
        self.project();
        let mut next = self.concentrations.clone();
        let advections = self.get_advections(delta);

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                if self.walls[idx] {
                    next[idx] = 0.0;
                    continue;
                }
                let source_rate = self.sources[idx];
                let advection = advections[idx];

                let neighbors = self.get_neighbors(idx, &advections);
                let mut neighbor_sum = 0.0;
                let mut fluid_count = 0.0;

                for neighbor in neighbors.iter() {
                    if let Some((value, idx)) = neighbor {
                        if !self.walls[*idx] {
                            neighbor_sum += value;
                        } else {
                            neighbor_sum += advection;
                        }
                        fluid_count += 1.0;
                    }
                }

                next[idx] = advection
                    + diffusion_coefficient * delta * (neighbor_sum - fluid_count * advection)
                        / (self.cell_size * self.cell_size) as f64
                    + source_rate;
            }
        }

        self.concentrations = next;
    }

    fn get_value_change(
        &self,
        value: f64,
        neighbors: [Option<(f64, usize)>; 4],
        advection_values: (f64, f64),
    ) -> (f64, f64) {
        let concentration_change_x = if advection_values.0 > 0.0
            && let Some(concentration_left) = neighbors[0]
        {
            (value - concentration_left.0) / self.cell_size as f64
        } else if advection_values.0 < 0.0
            && let Some(concentration_right) = neighbors[1]
        {
            (concentration_right.0 - value) / self.cell_size as f64
        } else {
            0.0
        };
        let concentration_change_y = if advection_values.1 > 0.0
            && let Some(concentration_up) = neighbors[2]
        {
            (value - concentration_up.0) / self.cell_size as f64
        } else if advection_values.1 < 0.0
            && let Some(concentration_down) = neighbors[3]
        {
            (concentration_down.0 - value) / self.cell_size as f64
        } else {
            0.0
        };

        (concentration_change_x, concentration_change_y)
    }

    fn get_forward_advections(&self, delta: f64) -> Vec<f64> {
        let mut forward_advections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;

                if self.walls[idx] {
                    forward_advections[idx] = self.concentrations[idx];
                    continue;
                }

                let concentration = self.concentrations[idx];
                let advection_values = self.advections[idx];
                let neighbors = self.get_neighbors(idx, &self.concentrations);

                let value_change =
                    self.get_value_change(concentration, neighbors, advection_values);

                let forward_advection = concentration
                    - delta
                        * (advection_values.0 * value_change.0
                            + advection_values.1 * value_change.1);

                forward_advections[idx] = forward_advection;
            }
        }

        forward_advections
    }

    fn get_backward_advections(&self, forward_advections: &Vec<f64>, delta: f64) -> Vec<f64> {
        let mut backward_advections: Vec<f64> = vec![0.0; self.grid_width * self.grid_height];

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;

                if self.walls[idx] {
                    backward_advections[idx] = self.concentrations[idx];
                    continue;
                }

                let advection_values = self.advections[idx];
                let forward_advection = forward_advections[idx];
                let neighbors = self.get_neighbors(idx, &forward_advections);

                let value_change = self.get_value_change(
                    forward_advection,
                    neighbors,
                    (-advection_values.0, -advection_values.1),
                );

                let backward_advection = forward_advection
                    - delta
                        * (-advection_values.0 * value_change.0
                            + -advection_values.1 * value_change.1);

                backward_advections[idx] = backward_advection;
            }
        }

        backward_advections
    }

    fn get_corrections(&self, delta: f64) -> Vec<f64> {
        let forward_advections = self.get_forward_advections(delta);
        let backward_advections = self.get_backward_advections(&forward_advections, delta);
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

    fn get_advections(&self, delta: f64) -> Vec<f64> {
        let corrections = self.get_corrections(delta);
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
                            min_advection = min_advection.min(n.0);
                            max_advection = max_advection.max(n.0);
                        }
                        None => {}
                    }
                }
                advections[idx] = correction.clamp(min_advection, max_advection);
            }
        }

        advections
    }

    fn project(&mut self) {
        let mut divergences = vec![0.0; self.grid_width * self.grid_height];
        let mut pressures = vec![0.0; self.grid_width * self.grid_height];

        for y in 1..self.grid_height - 1 {
            for x in 1..self.grid_width - 1 {
                let idx = y * self.grid_width + x;
                let u = self.advections[idx].0;
                let v = self.advections[idx].1;

                let u_left = if !self.walls[y * self.grid_width + (x - 1)] {
                    self.advections[y * self.grid_width + (x - 1)].0
                } else {
                    0.0
                };
                let v_up = if !self.walls[(y - 1) * self.grid_width + x] {
                    self.advections[(y - 1) * self.grid_width + x].1
                } else {
                    0.0
                };

                let divergence =
                    (u - u_left) / self.cell_size as f64 + (v - v_up) / self.cell_size as f64;

                divergences[idx] = divergence;
            }
        }

        for _ in 0..5 {
            for y in 0..self.grid_height {
                let row = y * self.grid_width;
                for x in 0..self.grid_width {
                    let idx = row + x;
                    if self.walls[idx] {
                        continue;
                    }
                    let neighbors = self.get_neighbors(idx, &pressures);

                    let mut neighbor_sum = 0.0;
                    let mut fluid_count = 0.0;

                    for neighbor in neighbors.iter() {
                        if let Some((value, idx)) = neighbor {
                            if !self.walls[*idx] {
                                neighbor_sum += value;
                            }
                            fluid_count += 1.0;
                        }
                    }

                    pressures[idx] = (neighbor_sum
                        - self.cell_size as f64 * self.cell_size as f64 * divergences[idx])
                        / fluid_count;
                }
            }
        }

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                if x < self.grid_width - 1 {
                    let right = y * self.grid_width + (x + 1);
                    if !self.walls[right] {
                        self.advections[idx].0 -=
                            (pressures[right] - pressures[idx]) / self.cell_size as f64;
                    } else {
                        self.advections[idx].0 = 0.0
                    }
                }
                if y < self.grid_height - 1 {
                    let down = (y + 1) * self.grid_width + x;
                    if !self.walls[down] {
                        self.advections[idx].1 -=
                            (pressures[down] - pressures[idx]) / self.cell_size as f64;
                    } else {
                        self.advections[idx].1 = 0.0
                    }
                }
            }
        }
    }

    fn get_neighbors(&self, idx: usize, values_grid: &Vec<f64>) -> [Option<(f64, usize)>; 4] {
        let x = idx % self.grid_width;
        let y = idx / self.grid_width;

        let left = if x > 0 {
            let neighbor_idx = y * self.grid_width + (x - 1);
            if !self.walls[neighbor_idx] {
                Some((values_grid[neighbor_idx], idx - 1))
            } else {
                None
            }
        } else {
            None
        };
        let right = if x + 1 < self.grid_width {
            let neighbor_idx = y * self.grid_width + (x + 1);
            if !self.walls[neighbor_idx] {
                Some((values_grid[neighbor_idx], idx + 1))
            } else {
                None
            }
        } else {
            None
        };
        let up = if y > 0 {
            let neighbor_idx = (y - 1) * self.grid_width + x;
            if !self.walls[neighbor_idx] {
                Some((values_grid[neighbor_idx], idx - self.grid_width))
            } else {
                None
            }
        } else {
            None
        };
        let down = if y + 1 < self.grid_height {
            let neighbor_idx = (y + 1) * self.grid_width + x;
            if !self.walls[neighbor_idx] {
                Some((values_grid[neighbor_idx], idx + self.grid_width))
            } else {
                None
            }
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

                let r = self.lerp(colour0.r as f64, colour1.r as f64, a);
                let g = self.lerp(colour0.g as f64, colour1.g as f64, a);
                let b = self.lerp(colour0.b as f64, colour1.b as f64, a);

                return Colour::new(r as u8, g as u8, b as u8, 255);
            }
        }

        Colour::new(0, 0, 0, 255)
    }

    fn lerp(&self, a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    fn draw_cell(&self, x: usize, y: usize, buffer: &mut [u8], colour: Colour) {
        for dy in 0..self.cell_size {
            for dx in 0..self.cell_size {
                let pixel_x = x * self.cell_size + dx;
                let pixel_y = y * self.cell_size + dy;

                let idx = (pixel_y * self.screen_width + pixel_x) * 4;

                buffer[idx] = colour.r;
                buffer[idx + 1] = colour.g;
                buffer[idx + 2] = colour.b;
                buffer[idx + 3] = colour.a;
            }
        }
    }
}
